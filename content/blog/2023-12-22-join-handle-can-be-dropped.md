+++
title = "スレッドは join しなくてもよい"
slug = "2023-12-22-join-handle-can-be-dropped"

[taxonomies]
"tags" = ["tech", "rust"]
+++

この記事は [Rust Advent Calendar 2023](https://qiita.com/advent-calendar/2023/rust) 22日目の記事です.

## TL;DR

`std::thread::JoinHanlde<T>` は `join()` しないとメモリリークすると思い込んでいたがそんなことはなかったな！

## 問題

Rust はメモリ安全・スレッド安全な言語であることを標榜しており, 多くの場合は安全でないことをしようとするとコンパイルが通りません.
例えば複数のスレッドからあるオブジェクトに対して可変参照 `&mut T` を通じて変更をすることはできず, 例えば `Arc<Mutex<T>>` などを利用する必要があります.

一方で安全な (`unsafe` でない) 操作のみを使っていても防げないものもあります. 典型的なものはメモリリークです.
例えば `Rc` を使って循環参照を作ってその部分を切り離すと, 切り離された側は永遠に drop されません.
(参照カウントが0になったときに drop されるが, 循環参照以外にどこからも参照されていないならカウントは変化しないため.)

長いこと勘違いしていたのですが, `std::thread::JoinHandle` にも同様の問題があると思い込んでました.

```rust
// join するならメモリリークしない
let handle = std::thread::spawn(|| {
    ...
});
handle.join()

// これは?
let _ = std::thread::spawn(|| {
    ...
});
```

ここでリークし得る (と思い込んでいた) メモリは以下の様なものです:

- スレッドに割り当てられたスタック領域
- スレッドの exit status を保存している (恐らくカーネル内の領域)
- スレッドの戻り値を保管している領域

## Joinable thread と detached thread

ところで POSIX スレッドには joinable thread と detached thread という概念があります.

[https://man7.org/linux/man-pages/man3/pthread_create.3.html](https://man7.org/linux/man-pages/man3/pthread_create.3.html)

その名の通り joinable thread は join できるスレッドです. join しなければリソースは解放されません.

detached thread は join はできませんが, スレッドが終了した時点で自動的にリソースは解放されます.

では Rust の実装はどうなっているのかというと

[https://github.com/rust-lang/rust/blob/1.74.1/library/std/src/thread/mod.rs#L1510-L1514](https://github.com/rust-lang/rust/blob/1.74.1/library/std/src/thread/mod.rs#L1510-L1514)

```rust
struct JoinInner<'scope, T> {
    native: imp::Thread,
    thread: Thread,
    packet: Arc<Packet<'scope, T>>,
}
```

[https://github.com/rust-lang/rust/blob/1.74.1/library/std/src/sys/unix/thread.rs#L284-L289](https://github.com/rust-lang/rust/blob/1.74.1/library/std/src/sys/unix/thread.rs#L284-L289)

```rust
impl Drop for Thread {
    fn drop(&mut self) {
        let ret = unsafe { libc::pthread_detach(self.id) };
        debug_assert_eq!(ret, 0);
    }
}
```

`JoinHandle` の本体である `JoinInner` が環境毎に実装されている `imp::Thread` を持ち, 特に unix 環境では drop 時に `libc::pthread_detach` が呼ばれ, joinable thread を detached thread に変更します.
これでカーネルが保持しているリソースはスレッド終了時に解放されます.

戻り値は `packet` で保持されているので drop されていればこれもスレッド終了時に解放されます.

スレッドが終了しているのに `JoinHandle` を drop も join もしないというのは普通書かないコードなのでこれでよさそうです.

ちなみに [公式ドキュメント](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html) には detach する旨はちゃんと書いてあります.
POSIX スレッドの知識が足らず, 見逃していました.

## まとめ

Rust では RAII を使ってスレッド終了後に上手いことリソースが解放されるようになっています. RAII ってべんりだなぁ.
