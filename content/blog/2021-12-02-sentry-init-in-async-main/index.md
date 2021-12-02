+++
title = "sentry::init() in anysc main()"
slug = "2021-12-02-sentry-init-in-async-main"

[taxonomies]
"tags" = ["tech", "rust"]
+++

この記事は [Rust Advent Calendar 2021](https://qiita.com/advent-calendar/2021/rust) 2日目の記事です.

## version など

<https://github.com/getsentry/sentry-rust>: [0.23.0](https://github.com/getsentry/sentry-rust/tree/0.23.0)

## Sentry の事前知識

[Sentry](https://sentry.io/) はエラーやトレースなどの情報を集約して管理できるサービスです.

Sentry に登録して, DSN (API token のようなもの) を取得して, 例えば

```rust
fn main() {
    let _guard = sentry::init(sentry::ClientOptions::default());

    sentry::capture_message("something wrong", sentry::Level::Error);

    panic!("explicit panic");
}
```

というプログラムを `SENTRY_DSN=<dsn> cargo run` として実行すると Sentry にエラーなどの情報が送られ, Sentry の UI 上でそれらを見ることができます.
べんりですね.

便利なのでよく web サーバーのエラーを把握したりするために使われたりします.

さて, 上記の `sentry::capture_message()` のところではエラーメッセージ以外の情報を渡していません.
エラーを送信するには DSN やらクライアントを使うはずです.

裏側を覗く前に登場人物を紹介しておきましょう.

- `Client`
  - https://docs.rs/sentry/0.23.0/sentry/struct.Client.html
  - DSN を保持しており, 実際の `capture_event()` の処理を行います.
- `Hub`
  - https://docs.rs/sentry/0.23.0/sentry/struct.Hub.html
  - スレッドに紐付いており, 文脈情報を保持します. その中には `Client` が含まれます. (後でもう少し詳しく説明します.)

`sentry::init()` から `sentry::capture_message()` までの流れは以下の様な感じです.

https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry/src/init.rs#L91

```rust
pub fn init<C>(opts: C) -> ClientInitGuard
where
    C: Into<ClientOptions>,
{
    let opts = apply_defaults(opts.into());
    let auto_session_tracking = opts.auto_session_tracking;
    let session_mode = opts.session_mode;
    let client = Arc::new(Client::from(opts));

    Hub::with(|hub| hub.bind_client(Some(client.clone())));
    if let Some(dsn) = client.dsn() {
        sentry_debug!("enabled sentry client for DSN {}", dsn);
    } else {
        sentry_debug!("initialized disabled sentry client due to disabled or invalid DSN");
    }
    if auto_session_tracking && session_mode == SessionMode::Application {
        crate::start_session()
    }
    ClientInitGuard(client)
}
```

`sentry::init()` はまず [`apply_defaults()`](https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry/src/defaults.rs#L84-L88) で環境変数 `SENTRY_DSN` から DSN を取得します.
それを用いて `Client` を作り, カレントスレッドの `Hub` に渡しています. (c.f. [`Hub::with()`](https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry-core/src/hub.rs#L151))

https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry-core/src/api.rs#L40

```rust
pub fn capture_event(event: Event<'static>) -> Uuid {
    Hub::with_active(|hub| hub.capture_event(event))
}
```

`sentry::capture_event()` はカレントスレッドの `Hub` の [`Hub::capture_event()`](https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry-core/src/hub.rs#L276) を呼び出しますが,
ここで `Hub` の中の `Client` の `Client::capture_event()` を呼びます. (c.f. [`Hub::with_active()`](https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry-core/src/hub.rs#L173))

というように, `sentry::capture_event()` をユーザーが手放しで呼び出せるのは sentry が裏側で **スレッドに紐づく `Hub`** を経由して色々やりくりしてくれているからというのがわかりますね.

## 問題設定

さて, ようやく本題です.

Q. 以下のプログラムは意図したとおりに動くでしょうか?

```
#[tokio::main]
async fn main() {
    let _guard = sentry::init(sentry::ClientOptions::default());

    ...do_something... // 色々処理をする中で sentry を呼び出す.
}
```

えー, まぁそりゃ動きますよね. そもそも何が問題なのかって?

[以前の記事](https://keno-ss.hatenadiary.org/entry/2019/12/01/235828) でも少し触れましたが, Rust の非同期の仕組みは

```
- async/await 構文を Future に変換.
- Future は asymmetric stackless coroutine. 状態機械として実装可能なので低コスト.
- Future は合成可能なので asymmetric stackful coroutine 概念を作れる.
- runtime がそれらを管理することで symmetric stackful coroutine のように見做せる.
```

でした. async fn は `Future` になり, それは runtime (ここでは `tokio`) から `poll()` されます.
ところで `poll()` が呼び出されるときのスレッドって, どのスレッドが使われるんでしたっけ?
残念ながらそれに対する「保証」などはありません.

じゃあスレッドに情報を紐付けてなんやかんや上手くやってくれる sentry は動くの? というのがこの記事の主題です.

## 壊してみる

さて, じゃあ一回壊してみましょうか.

Q. で「意図したとおりに動くの?」と書きましたが, 基本的には sentry の呼び出し自体でエラーが起きたりはしません.
ただ, `Client` がないためにエラーが送られない, という現象になります.
それも意図してない挙動ではあるので, ここでは「`Hub` にちゃんと `Client` が設定されているのか」を調べてみましょう.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!(
        "current thread id for `sentry::init()`: {:?}",
        std::thread::current().id()
    );
    let _guard = sentry::init(sentry::ClientOptions::default());
    assert!(Hub::current().client().is_some());
    assert!(Hub::main().client().is_some());

    let mut futs = vec![];
    for _ in 0..10 {
        futs.push(tokio::task::spawn(async {
            println!("current thread id: {:?}", std::thread::current().id());
            assert!(Hub::current().client().is_some());
            assert!(Hub::main().client().is_some());
        }));
    }
    futures::future::join_all(futs).await;

    println!("passed");

    Ok(())
}
```

最初に `sentry::init()` を呼び出した後, 各 `Future` の中で `Hub` の `Client` を調べています.
`Hub` が `Client` を持っていなければ `assert!()` で panic するはずです.

```console
$ SENTRY_DSN=<dsn> cargo run
   Compiling example-sentry-init-in-async-main v0.1.0 (/home/keno/src/github.com/kenoss/kenoss.github.io/content/blog/2021-12-02-sentry-init-in-async-main/code/example-sentry-init-in-async-main)
    Finished dev [unoptimized + debuginfo] target(s) in 10.43s
     Running `target/debug/example-sentry-init-in-async-main`
current thread id for `sentry::init()`: ThreadId(1)
current thread id: ThreadId(5)
current thread id: ThreadId(9)
current thread id: ThreadId(5)
current thread id: ThreadId(9)
current thread id: ThreadId(5)
current thread id: ThreadId(9)
current thread id: ThreadId(5)
current thread id: ThreadId(9)
current thread id: ThreadId(5)
current thread id: ThreadId(9)
passed
```

んー, panic しませんね?

ちょっと弄ってみます.

```rust
use sentry::Hub;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tokio::task::spawn(async {
        println!(
            "current thread id for `sentry::init()`: {:?}",
            std::thread::current().id()
        );
        let _guard = sentry::init(sentry::ClientOptions::default());
        assert!(Hub::current().client().is_some());
        assert!(Hub::main().client().is_some());

        let mut futs = vec![];
        for _ in 0..10 {
            futs.push(tokio::task::spawn(async {
                println!("current thread id: {:?}", std::thread::current().id());
                assert!(Hub::current().client().is_some());
                assert!(Hub::main().client().is_some());
            }));
        }
        futures::future::join_all(futs).await;

        println!("passed");
    })
    .await?;

    Ok(())
}
```

`tokio::task::spawn()` でくるんでみました.
これで `sentry::init()` は (プロセス開始時のスレッドとは違うかもしれない) ランダムなスレッドで実行されることになります.

```console
$ SENTRY_DSN=<dsn> cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.63s
     Running `target/debug/example-sentry-init-in-async-main`
current thread id for `sentry::init()`: ThreadId(9)
current thread id: ThreadId(7)
current thread id: ThreadId(8)
current thread id: ThreadId(3)
current thread id: ThreadId(8)
current thread id: ThreadId(7)
current thread id: ThreadId(9)
current thread id: ThreadId(7)
current thread id: ThreadId(7)
current thread id: ThreadId(8)
current thread id: ThreadId(3)
passed
```

あれ...? こんなはずでは...

```rust
use sentry::Hub;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    assert!(Hub::current().client().is_none());
    assert!(Hub::main().client().is_none());

    tokio::task::spawn(async {
        println!(
            "current thread id for `sentry::init()`: {:?}",
            std::thread::current().id()
        );
        let _guard = sentry::init(sentry::ClientOptions::default());
        assert!(Hub::current().client().is_some());
        assert!(Hub::main().client().is_some());

        let mut futs = vec![];
        for _ in 0..10 {
            futs.push(tokio::task::spawn(async {
                println!("current thread id: {:?}", std::thread::current().id());
                assert!(Hub::current().client().is_some());
                assert!(Hub::main().client().is_some());
            }));
        }
        futures::future::join_all(futs).await;

        println!("passed");
    })
    .await?;

    Ok(())
}
```

ここで塩をひとつまみ.
最初に謎の assert を入れてみます.

```
$  SENTRY_DSN=<dsn> cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 6.88s
     Running `target/debug/example-sentry-init-in-async-main`
current thread id for `sentry::init()`: ThreadId(7)
thread 'tokio-runtime-worker' panicked at 'assertion failed: Hub::main().client().is_some()', src/main.rs:15:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Error: panic
```

お!! 上手くいきましたね!!

やっと `sentry::init()` を呼んでいるのに `Hub` が `Client` を持っていない, という状況ができました.

## 解説編

https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry-core/src/hub.rs#L142

```rust
    pub fn main() -> Arc<Hub> {
        PROCESS_HUB.0.clone()
    }
```


`Hub::main()` は単に `Hub` を参照しているだけのように見えます. 何で `Hub::main()` の呼び出しなしでは上手くいき, 呼び出しを足すと意図しない状況が発生したのでしょうか?

これを理解するためには, `Hub` がいつどのように初期化されるのかを知る必要があります.

https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry-core/src/hub.rs#L17-L30

```rust
#[cfg(feature = "client")]
lazy_static::lazy_static! {
    static ref PROCESS_HUB: (Arc<Hub>, thread::ThreadId) = (
        Arc::new(Hub::new(None, Arc::new(Default::default()))),
        thread::current().id()
    );
}

#[cfg(feature = "client")]
thread_local! {
    static THREAD_HUB: UnsafeCell<Arc<Hub>> = UnsafeCell::new(
        Arc::new(Hub::new_from_top(&PROCESS_HUB.0)));
    static USE_PROCESS_HUB: Cell<bool> = Cell::new(PROCESS_HUB.1 == thread::current().id());
}
```

`hub.rs` の中ではふたつの変数が定義されています.

- `PROCESS_HUB`
  - プロセス内で unique なやつ.
  - `sentry::init()` で `Client` がセットされて **ほしい**.
- `THREAD_HUB`
  - スレッド毎に初期化される. 初期化時に `PROCESS_HUB` を参照して利用する.

対応する `lazy_static!` と `thread_local!` の初期化タイミングはそれぞれ以下です.

- `lazy_static!`
  - 最初にアクセスしたときに初期化される.
- `thread_local!`
  - https://doc.rust-lang.org/std/thread/struct.LocalKey.html#initialization-and-destruction
  - 最初に `with()` が呼ばれたときに初期化される.

ということは `PROCESS_HUB` は最初にどこかの `THREAD_HUB` が参照されたときに初期化されます.

一方, `sentry::init()` は `Hub::with` で初期化対象の `Hub` を選んでいます.

https://github.com/getsentry/sentry-rust/blob/0.23.0/sentry-core/src/hub.rs#L150

```rust
    #[cfg(feature = "client")]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Arc<Hub>) -> R,
    {
        if USE_PROCESS_HUB.with(Cell::get) {
            f(&PROCESS_HUB.0)
        } else {
            // note on safety: this is safe because even though we change the Arc
            // by temporary binding we guarantee that the original Arc stays alive.
            // For more information see: run
            THREAD_HUB.with(|stack| unsafe {
                let ptr = stack.get();
                f(&*ptr)
            })
        }
    }
```

`Hub::with()` はカレントスレッドが `PROCESS_HUB` を初期化したスレッドならそれを使い, そうでなければ `THREAD_HUB` を使います.

なので, 壊してみるのところで何が起こっていたかというと,

- `sentry::init()` の呼び出し前に何もなければ, `PROCESS_HUB` に `Client` がセットされ, 他のスレッドでは `Hub::with()` などで `Hub` を使おうとした瞬間に `PROCESS_HUB` を参照して `Client` を設定する.
- `sentry::init()` の呼び出し前に `Hub::main()` などの呼び出しがあると, そのスレッドで `PROCESS_HUB` が初期化される. 他のスレッドで `sentry::init()` するとそのスレッドの `THREAD_HUB` に `Client` がセットされる. その他のスレッドでは `THREAD_HUB` に `Client` はいない.

となっていたのでした. なるほどなぁ.

## 解答編

元の疑問

```
Q. 以下のプログラムは意図したとおりに動くでしょうか?
```

に戻ると, 答えは

A. async main() の先頭で `sentry::init()` を呼び出している限りは `Client` はセットされる.

ということがわかりました.

ふぅ, これでやっと安心して production のサーバーに sentry を入れられますね.

ただ, ここでは `Client` が意図通りにセットされるのかどうかしか見ていません.
例えば [breadcrumb](https://docs.rs/sentry/0.23.0/sentry/struct.Hub.html#method.add_breadcrumb) のようなものはスレッドに紐付きます.
スレッドに紐づくものは async 下では基本的には上手く動かないです.

じゃあそういう「文脈情報」まで取得したいとなったらどうすればいいのかというと, それは [`tracing` crate](https://github.com/tokio-rs/tracing) の話になるので, また気が向いたらそれについて書くかもしれません.
