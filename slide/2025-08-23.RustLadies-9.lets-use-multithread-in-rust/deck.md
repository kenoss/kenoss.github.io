<!-- https://docs.google.com/presentation/d/1uToGGW8HytOrEpAtExvnj7QudG82nrmQxY8spsOK-6s/edit?slide=id.SLIDES_API1386870661_0 -->

<!-- {"freeze": true} -->

<!-- {"layout": "cover"} -->

# Rust で始めるマルチスレッド

## keno (@kenoss/@keno_ss) 2025-08-23 [RustLadies #9](https://rustladies.connpass.com/event/362014/) (LT 10分)

---

<!-- (1.5 min) -->

<!-- {"layout": "center-big"} -->

Q. マルチスレッドって何?

A. なんか並列にやってくれるやつ

---

## モチベーション

- 速くしたい (並列にしたい)
  - ripgrep: たくさんのファイルを並列に読み込みんで grep した結果を全部表示する.
- 待ちたくない (block されたくない)
  - starship: (ちょろっと読んだ感じ) `git status` が遲ければタイマーで中断.

---

<!-- {"freeze": true} -->

## どう動くの? (oversimplified Linux)

- スレッド
  - 動かしたいスレッドを OS にお願いすると
  - OS が一定時間毎などプリエンプティブにスレッドを切り替えて
  - (複数の) CPU を使って複数のスレッドを動かす
- async
  - 動かしたいタスクをランタイムにお願いすると
  - ランタイムがイベント起因で協調的にタスクを切り替えて
  - (複数の) スレッドを使って複数のタスクを動かす

---

<!-- {"layout": "center-big"} -->

似てる!!

実は全然違う

---

<!-- {"layout": "center-big"} -->

Q. どっちが易しい/難しい?

A. 場合による.

---

<!-- {"freeze": true} -->

<!-- {"layout": "center-big"} -->

Q. どっちが適切?

A. 場合による.

(個人的にはスレッドが「使える」なら

スレッドを選ぶ.)

---

<!-- (1 min) -->

<!-- {"layout": "center-big"} -->

比較するためにもまずは知るところから

---

<!-- {"layout": "center-big"} -->

なんか並列にやってくれるやつ?

---

<!-- {"freeze": true} -->

<!-- {"layout": "center-big"} -->

- 動かしたいスレッドを OS にお願いすると
- OS がスレッドを切り替えて
- (複数の) CPU を使って複数のスレッドを動かす

---

<!-- {"layout": "blank"} -->

```
use std::time::Duration;

fn main() {
    let t0 = std::thread::spawn(|| {
        for _ in 0..3 {
            std::thread::sleep(Duration::from_secs(1));
            println!("マルチスレッドは普通");
        }
    });
    let t1 = std::thread::spawn(|| {
        for _ in 0..3 {
            std::thread::sleep(Duration::from_secs(1));
            println!("マルチスレッドさんありがとう");
        }
    });
    let t2 = std::thread::spawn(|| {
        for _ in 0..3 {
            std::thread::sleep(Duration::from_secs(1));
            println!("マルチスレッドが使えると大喜び");
        }
    });

    t0.join().unwrap();
    t1.join().unwrap();
    t2.join().unwrap();
}
```

---

<!-- {"layout": "blank"} -->

---

---

## どういうときに使えるの?

- ひとつの動作, たくさんの対象
- 複数の操作, パイプライン
- blocking 処理と割り込み (例: シグナルハンドリング)

---

<!-- (2 min) -->

## ひとつの動作, たくさんの対象

- ripgrep: たくさんのファイルを並列に読み込みんで grep した結果を全部表示する.
- Chromium (> 55万 files) で20秒くらい

---

<!-- {"layout": "blank"} -->

```
fn search_parallel(args: &HiArgs, mode: SearchMode) -> anyhow::Result<bool> {
    ...
    // 複数 worker でディレクトリを舐めて (directory walk)
    args.walk_builder()?.build_parallel().run(|| {
        ...
        // それぞれの worker で, ディレクトリが与えられる毎に
        Box::new(move |result| {
            ...
            // grep して
            let search_result = match searcher.search(&haystack) {
                Ok(search_result) => search_result,
                Err(err) => {...}
            };
            ...
            // grep 結果を出力
            if let Err(err) = bufwtr.print(searcher.printer().get_mut()) {
                ...
            }
            ...
        })
    });
}
```

---

<!-- (2 min) -->

## 複数の操作, パイプライン

- `cargo watch -c -s 'cargo build && just check-strict'`
  - エディタでファイルを弄ったら `cargo build` と clippy などのチェックを走らせる
- Rust 以外でも使いたい
  - `watchexec`
  - でも大量のファイルがあると遅い... (忘れたけど140秒くらい?)
- ほな自作 (`watchdo`)

---

## ボトルネック

- directory walk は遲くない. (ripgrep は速いので)
- inotify の登録が遅い.
- ほな監視ファイルを減らそう
  - `git status` はまぁ待てるくらいには速い (3秒)
  - まず `git status` で出てきたファイルを watch し, それとは並列して walk

---

<!-- {"layout": "blank"} -->

```
GitProducer -> GitWatcher, WalkProducer -> WalkWatcher
↓
Switcher
↓
Delayer
↓
Runner
```

---

<!-- (1 min) -->

## `crossbeam`

- `crossbeam`
  - 並行プログラミングのツール集
- `crossbeam::channel`
  - Golang のチャンネルみたいなやつ
- `crossbeam::select!`
  - 複数チャンネルを待てる.

---

<!-- {"layout": "blank"} -->

```
impl Switcher {
    pub fn event_loop(mut self) -> eyre::Result<()> {
        let mut receivers = if let Some(git_watcher_receiver) = self.git_watcher_receiver {
            // GitWatcher と WalkWatcher を見る.
            vec![git_watcher_receiver, self.file_watcher_receiver.unwrap()]
        } else {
            vec![self.file_watcher_receiver.unwrap()]
        };
        ...
        loop {
            select! {
                // 終了シグナルが来たら終了
                recv(self.tlink.receiver()) -> _ => {
                    return Ok(());
                }
                // 切り替えシグナルが走ったら GitWatcher を捨てる.
                recv(walk_initialized) -> _ => {
                    if receivers.len() > 1 {
                        receivers.remove(0);
                    }
                }
                // 今見てる方を後続に素通し
                recv(receivers[0]) -> message => {
                    let message = message?;
                    let _ = sender.send(message);
                }
            }
        }
    }
}
```

---

<!-- {"layout": "blank"} -->

---

<!-- {"layout": "blank"} -->

---

<!-- {"layout": "blank"} -->

---

<!-- {"layout": "blank"} -->

---

<!-- {"layout": "blank"} -->

---

<!-- (3 min) -->

<!-- {"layout": "center-big"} -->

「自分のはただの CLI プログラムだから

スレッド関係ないよ」

そうかな...?

---

## 例: シグナルハンドリング

- 時間がかかる処理をする CLI プログラム
- ユーザーが Ctrl-C を打ったら終了処理をして終了.
  - デフォルトではメインスレッドが即終了されてしまう.

---

## 例: (Oversimplified) `wl-screenrec`

(勝手に例にして申し訳ない. でも皆一度は踏む良い例だったから...)

- Linux (Wayland) の画面を録画する
- ユーザーが Ctrl-C を打ったらエンコード結果を flush して終了.

---

<!-- {"layout": "blank"} -->

```
fn execute<S: CaptureSource + 'static>(args: Args, conn: Connection) {
    // ::MAX means still running, otherwise it's an exit value
    let quit_flag = Arc::new(AtomicUsize::new(usize::MAX));

    // Ctrl-C が来たらフラグが立つ.
    signal_hook::flag::register_usize(SIGINT, Arc::clone(&quit_flag), 0).unwrap();

    // フラグが立つまでは処理を回し続ける.
    while quit_flag.load(Ordering::SeqCst) == usize::MAX {
        // 画像が送られてくるのを待って処理.
        queue.blocking_dispatch(&mut state).unwrap();
    }

    // エンコード結果を flush して終了.
    if let EncConstructionStage::Complete(c) = &mut state.enc {
        c.enc.flush();
    }

    exit(quit_flag.load(Ordering::SeqCst) as i32)
}

// これはバグがあります. さてどういうときに何が起こるでしょう?
```

---

<!-- {"layout": "blank"} -->

---

A. 0 FPS (画像が送られてこないので録画できない) のときに Ctrl-C で終了できない ([#122](https://github.com/russelltg/wl-screenrec/issues/122))

---

<!-- {"layout": "blank"} -->

```
fn execute<S: CaptureSource + 'static>(args: Args, conn: Connection) {
    // ::MAX means still running, otherwise it's an exit value
    let quit_flag = Arc::new(AtomicUsize::new(usize::MAX));

    // Ctrl-C が来たらフラグが立つ.
    signal_hook::flag::register_usize(SIGINT, Arc::clone(&quit_flag), 0).unwrap();

    // フラグが立つまでは処理を回し続ける.
    while quit_flag.load(Ordering::SeqCst) == usize::MAX {
        // ここで無限に待つ... ので ↑ の判定に入れない.
        queue.blocking_dispatch(&mut state).unwrap();
    }

    // エンコード結果を flush して終了.
    if let EncConstructionStage::Complete(c) = &mut state.enc {
        c.enc.flush();
    }

    exit(quit_flag.load(Ordering::SeqCst) as i32)
}
```

---

## 解法1

- 両方ともイベントを待っている.
  - シグナルが上がった
  - 画像が送られてきた
- これはさっきやった `crossbeam::select!`
- あるいは (panic 時の処理もするなら) blocking 処理を別スレッドに切り出して, 以下を待つ:
  - シグナルが上がった
  - blocking 処理が終わった (正常/異常含む)
  - blocking 処理が panic した

教訓: 複数のイベントや blocking 処理が絡むとスレッドが出てくる.

---

## 解法2

- 割り込みハンドラと `mio::Poll` を使う. (issue はこれで解決されたらしい.)
- (と, このケースではスレッド使わなくても行けるはず. まぁ `wl-screenrec` は元々スレッド使ってるからどっちでもいいけど.)

`mio::Poll` とは: ファイルディスクリプタ版 `crossbeam::select!` (oversimplified)

---

<!-- (1 min) -->

## スレッドか async どちらを使うべき?

個人的な判定法

- HTTP サーバーなら asnyc
- 依存ライブラリが async なら (部分的に) async
- スレッド数が無限に増え得るなら async
- それ以外ならスレッド

スレッドの方が楽なので.

---

## まとめ

- スレッドって何?
  - OS が複数スレッドを複数 CPU で実行する
- どういうときに使えるの?
  - ひとつの動作, たくさんの対象 (例: ripgrep)
  - 複数の操作, パイプライン (例: watchdo)
  - blocking 処理と割り込み (例: シグナルハンドリング)
- Good parts
  - `crossbeam::channel`/`crossbeam::select!`

---

## Thanks!

- 次のおすすめ
  - [Linuxのプロセススケジューラの歴史 v2.6.23~v4.18](https://speakerdeck.com/sat/linuxfalsepurosesusukeziyurafalseli-shi-v2-dot-6-23-v4-dot-18)
  - [プログラマーのためのCPU入門 ― CPUは如何にしてソフトウェアを高速に実行するか](https://www.lambdanote.com/products/cpu)
  - `Send`, `Sync`, `Mutex`, condition variable, ...
- async は?
  - たぶん maguro (@yusuktan) さんの方が詳しい

Powered by: [k1LoW/deck](https://github.com/k1LoW/deck), [Aloxaf/silicon](https://github.com/Aloxaf/silicon)
