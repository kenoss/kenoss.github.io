+++
title = "一歩踏み込んだ tracing の設定 (Rust)"
slug = "2025-12-13-advanced-tracing-config"

[taxonomies]
"tags" = ["rust", "tech", "tracing"]
+++

これは [Rust Advent Calendar 2025](https://qiita.com/advent-calendar/2025/rust) 13日目の記事です [^1].

## Intro

みなさん tracing してますか? してますよね. 私はほぼ常にしています.

Rust での [`tracing`](https://docs.rs/tracing/latest/tracing/) は, ざっくり言うと

- ([`tracing`](https://docs.rs/tracing/latest/tracing/); ライブラリやアプリ側で)
  log や span を仕込んでおき,
- ([`tracing_subscriber`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/); アプリ側で)
  それらを表示したりのアクションを起こす.

仕組みです.

この記事では基本的な概念, どうやったら使えるのか, 内部の仕組みについては解説せず, 仮定します. 以下の記事がおすすめです:

- そもそも tracing って何? どうやったら使えるの?
  - 公式ドキュメント
    [`tracing`](https://docs.rs/tracing/latest/tracing/),
    [`tracing_subscriber`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/)
  - [Rustでオブザーバビリティを実現するには (@ymgyt)](https://techblog.paild.co.jp/entry/2024/04/02/144212)
- 内部の仕組み
  - [tracing/tracing-subscriberでログが出力される仕組みを理解する (@helloyuki)](https://blog.ymgyt.io/entry/how-tracing-and-tracing-subscriber-write-events/)
- otel で使ってみたい
  - [Rust での tracing (@sadnessOjisan)](https://blog.ojisan.io/rust-tracing/)

さて, `tracing` crate の使い方として最もよくあるのは
[`log`](https://docs.rs/log/latest/log/)/[`env_logger`](https://docs.rs/env_logger/latest/env_logger/) crate
的な挙動のために以下の様な設定を使うやつでしょう:

```rust
#[allow(unused_imports)]
#[macro_use]
extern crate tracing;

use anyhow;

fn tracing_init() {
    use time::UtcOffset;
    use time::macros::format_description;
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt::time::OffsetTime;

    let offset = UtcOffset::current_local_offset().unwrap();
    let timer = OffsetTime::new(
        offset,
        format_description!("[hour]:[minute]:[second].[subsecond digits:3]"),
    );

    tracing_subscriber::fmt()
        // `RUST_LOG` 環境変数に応じてログを出力するかどうかを制御
        .with_env_filter(EnvFilter::from_default_env())
        // 時刻のフォーマット変更
        .with_timer(timer)
        // trace のファイル中の行数を表示
        .with_line_number(true)
        // 色を付ける
        .with_ansi(true)
        .init();
}

fn main() -> anyhow::Result<()> {
    tracing_init();

    hoge()?;

    Ok(())
}

fn hoge() -> anyhow::Result<()> {
    info!("log in hoge");

    f(2);

    Ok(())
}

#[tracing::instrument]
fn f(n: usize) {
    debug!("f({n})");

    if n == 0 {
        return;
    }

    f(n - 1);
}
```

出力サンプルはこんな感じです:

```
$ RUST_LOG=debug cargo run
   Compiling hoge v0.1.0 (/home/kenoss/work/hoge)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.45s
     Running `target/debug/hoge`
23:33:39.858  INFO hoge: 40: log in hoge
23:33:39.858 DEBUG f{n=2}: hoge: 49: f(2)
23:33:39.858 DEBUG f{n=2}:f{n=1}: hoge: 49: f(1)
23:33:39.858 DEBUG f{n=2}:f{n=1}:f{n=0}: hoge: 49: f(0)
```

この記事ではここから一歩進んで, 以下の機能を足していきます:

- stdout とファイルの片方, あるいは両方に出力する.
- log に span context を表示しない.
- [Perfetto](https://perfetto.dev/)/[Perfetto UI](https://ui.perfetto.dev/)
  で読める trace を記録する.
- フィルタリングを動的に on/off する.

初手からアレなんですが, 逆に言うとこういう †変なこと† をする必要がない場合はこの記事を読む必要はないです.
必要になってから読めばよいです.

## Versions

- `tracing`/`tracing_subscriber`: tag [`tracing-0.1.43`](https://github.com/tokio-rs/tracing/tree/tracing-0.1.43)
- sabiniwm: commit [37b905f](https://github.com/kenoss/sabiniwm/tree/37b905f)

## `tracing_subscriber::fmt()` を紐解く

`tracing`/`tracing_subscriber` は基本的に simple な概念の積み重ねで作られています. しかし
`tracing_subscriber::fmt()` は違います. こいつはよくあるユースケースに特化した simple な
インターフェースです. その範囲に収まっていれば短かく書けて便利なのですが,
一歩踏み込もうとしたときには邪魔 [^2] でもあります.

というわけで, まずはこれを `tracing_subscriber` の基本概念である
[`Registry`](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/registry/struct.Registry.html)
[`Subscriber`](https://docs.rs/tracing-core/0.1.35/tracing_core/subscriber/trait.Subscriber.html),
[`Layer`](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/layer/trait.Layer.html),
[`Filter`](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/layer/trait.Filter.html)
で書き直しましょう.

```rust
tracing_subscriber::fmt().init();
```

最小の設定はこれですが, これは次と等価です
[[cs](https://github.com/tokio-rs/tracing/blob/64e1c8d3ae5cf5deab40ad3d376c8595d4e4db7f/tracing-subscriber/src/fmt/mod.rs#L486-L491)].

```rust
let subscriber = Registry::default()
    .with(fmt::Layer::default());
subscriber.init();
```

`EnvFilter` を足すと

```rust
let subscriber = Registry::default()
    .with(EnvFilter::from_default_env())
    .with(fmt::Layer::default());
subscriber.init();
```

になります. (`with_line_number()` などは暫く無視します.)

さて, これは何をやっているのでしょうか?

下から逆順に見ていくと, `subscriber.init()` は
[`SubscriberInitExt::init()`](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/util/trait.SubscriberInitExt.html#method.init)
であり, これは trait `Subscriber` を global に set します. 変数 `subscriber` の型は (`tracing_subscriber::` は省略して)

```
layer::Layered<fmt_layer::Layer<???, N, E, W>,
               layer::Layered<EnvFilter,
                              Registry
                              >
               >
```

です. [`Layered`](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/layer/struct.Layered.html)
`layer::Layered<L, I, S = I>` は `L: Layer` と `S: Subscriber` を取って `Subscriber` を返す様なものです.
その核になっているのは `Registry` で, これは `Subscriber` を impl しています.

`Registry` を使う理由は log と span はちょっと異なり span の情報を生存期間中保持しておくため...ですが,
まぁあんまり深く考えずに取り敢えず `Registry` を使うと思っておけばよいです.
`EnvFilter` はそこから降りてくる log/span を `RUST_LOG` 環境変数に従って filtering します [^3].
`fmt_layer::Layer` はそれらを文字列化し, stdout に書き出します.

というわけで, これで `tracing_subscriber::fmt().init()` をバラせました.

## Per-layer filtering

さて, ここで log や span を **全て** 記録したいとなったときはどうすればいいのでしょう?

```rust
let subscriber = Registry::default()
    .with(ここで log and/or span を記録する何か)
    .with(EnvFilter::from_default_env())
    .with(fmt::Layer::default());
subscriber.init();
```

filter の前に置いているから filtering はされずに全て記録できるでしょうか?
残念ながらこれは `EnvFilter` の影響を受けます [^4]. (何故か?という話は控えます [^5].)

そこで使われるのが
[Per-Layer Filtering](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/layer/index.html#per-layer-filtering)
です. 名前の通り, layer 毎に filtering を個別に持たせます:

```rust
let subscriber = Registry::default()
    .with(ここで log and/or span を記録する何か)
    .with(fmt::Layer::default()
              .with_filter(EnvFilter::from_default_env()));
subscriber.init();
```

**`tracing_subscriber::fmt().<...>.init()` で済まないときは per-layer filtering を使う.**
今日はこれだけ覚えて帰ってください.

以降は全てこれを使った具体例になります.

## stdout and/or file に書く

プログラム開始時点での条件によって以下を切り替えたりしたいです [^6].

- stdout にのみ出力する.
- ファイルに出力する.
- stdout とファイルの両方に出力する.

これは以下の様にすればよいです.

```rust
fn tracing_init() -> anyhow::Result<()> {
    macro_rules! fmt_layer {
        () => {{
            let offset = UtcOffset::current_local_offset().unwrap();
            let timer = OffsetTime::new(
                offset,
                format_description!("[hour]:[minute]:[second].[subsecond digits:3]"),
            );

            tracing_subscriber::fmt::Layer::default()
                .compact()
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
                .with_ansi(true)
                .with_timer(timer)
                .with_level(true)
                .with_target(true)
                .with_file(false)
                .with_line_number(true)
        }};
    }
    let (stdout_logging, file_logging) = if use_file_for_logging() {
        const LOG_FILE: &str = "/tmp/hoge.log";
        let log_file = std::io::LineWriter::new(std::fs::File::create(LOG_FILE)?);
        let writer = std::sync::Mutex::new(log_file);
        let file_logging = fmt_layer!().with_writer(writer);
        (None, Some(file_logging))
    } else {
        let stdout_logging = fmt_layer!().with_writer(std::io::stdout);
        (Some(stdout_logging), None)
    };
    let env_filter = EnvFilter::from_default_env();

    let subscriber = Registry::default()
        .with(
            stdout_logging
                .with_filter(env_filter.clone()),
        )
        .with(
            file_logging.with_filter(env_filter),
        );
    subscriber.init();

    Ok(())
}
```

解説. これは
[`L: Layer<S>` のとき `Option<L>: Layer<S>`](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/layer/trait.Layer.html#impl-Layer%3CS%3E-for-Option%3CL%3E)
というのを利用しています. `None` のときは **挙動としては** `.with(None)` を書かなかった場合と同じになります.
(返す型は異なります.) ファイル出力用の layer と stdout 出力用の layer をそれぞれ `Option` で包んで設定しています.
その中では先程の per-layer filtering を使っています.

(両方 `Some` のケースがないのであれば) こんな面倒なことをしなくてもいいのでは?と思われるかもしれません.
`let logging = if ... else ... ;` で layer を切り替える感じですね. しかしこれはコンパイルできません.
(`EnvFilter` の `Layered` を無視すると) 実は `tracing_subscriber::fmt::Layer` は
`Layer<Registry, N, E, W>` で,
[`.with_writer()`](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/fmt/struct.Layer.html#method.with_writer)
は型パラメータ `W` を変更します. stdout とファイルでこの部分が異なるので同じ変数には入れられません.

型というと, (関数ではなく) `fmt_layer!()` というマクロを使っているのもポイントです.

```rust
let subscriber = Registry::default()
    .with(EnvFilter::from_default_env())
    .with(fmt::Layer::default());
subscriber.init();
```

の `subscriber` の型は

```
layer::Layered<fmt_layer::Layer<???, N, E, W>,
               layer::Layered<EnvFilter,
                              Registry
                              >
               >
```

と書きましたが, この `???` は `layer::Layered<EnvFilter, Registry>` です
[[`fmt::Layer::with_subscriber()`](https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/fmt/struct.Layer.html#method.with_subscriber)]
[[`SubscriberExt::with()`](https://github.com/tokio-rs/tracing/blob/64e1c8d3ae5cf5deab40ad3d376c8595d4e4db7f/tracing-subscriber/src/layer/mod.rs#L1500-L1509)].
ここには「`.with()` を付ける前の subscriber の型」が入ります. こんなの手書きしたくないですし,
`.with(stdout_logging...)` と `.with(file_logging...)` の箇所で要求される型が異なります.
なのでマクロにしています.

## log に span context を表示しない

上記 `fmt_layer!()` の部分,

```rust
tracing_subscriber::fmt::Layer::default()
    .compact()
    .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
    .with_ansi(true)
    .with_timer(timer)
    .with_level(true)
    .with_target(true)
    .with_file(false)
    .with_line_number(true)
```

としていますが, 出力は例えば

```
...
23:39:47.066  INFO smithay::wayland::socket:72: Created new socket name=Some("wayland-2")
23:39:47.068  INFO sabiniwm::state:234: Start listening on Wayland socket: WAYLAND_DISPLAY = wayland-2
23:39:47.068  INFO input_seat:add_keyboard:input_keyboard: smithay::input::keyboard:672: Initializing a xkbcommon handler with keymap query name="winit" xkb_config=XkbConfig { rules: "", model: "", layout: "custom", variant: "", options: None } repeat_delay=200 repeat_rate=60
23:39:47.073  INFO input_seat:add_keyboard:input_keyboard: smithay::input::keyboard:680: Loaded Keymap name="Modifiers for MacBookAir" name="winit" xkb_config=XkbConfig { rules: "", model: "", layout: "custom", variant: "", options: None } repeat_delay=200 repeat_rate=60
23:39:47.078  INFO smithay::xwayland::xserver:187: spawning XWayland instance
...
```

などとなります. 3行目と4行目はそれ以外と異なり `input_seat:add_keyboard:input_keyboard:` と表示されていますが,
これは log されたときの道中の span を表しています.

しかしこれ, 結構邪魔です. span は必要なときは別途 perfetto で見るつもりだし, log はコンパクトに
表示したい. ではどうやって消すか?

実はこれ簡単には消せません. `tracing_subscriber::fmt::Layer` は色々オプションを持っていますが,
[FmtCtx](https://github.com/tokio-rs/tracing/blob/64e1c8d3ae5cf5deab40ad3d376c8595d4e4db7f/tracing-subscriber/src/fmt/format/mod.rs#L1094-L1104)
には表示を弄るオプションがありません. 困りましたね... 自作 formatter を作るしかないのでしょうか...?

いや, この場合はできます. そう, filter ならね!

```rust
/// Filters out span context for event logging
pub struct NoSpanContextFilter;

impl<S> tracing_subscriber::layer::Filter<S> for NoSpanContextFilter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn enabled(
        &self,
        metadata: &tracing_core::Metadata<'_>,
        _cx: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        !metadata.is_span()
    }
}

...

    let subscriber = Registry::default()
        .with(
            stdout_logging
                .with_filter(env_filter.clone())
                .with_filter(NoSpanContextFilter),
        )
        .with(
            file_logging
                .with_filter(env_filter)
                .with_filter(NoSpanContextFilter),
        );

...
```

`Filter::enabled()` で span のときに false を返しています. こうすることで
`tracing_subscriber::fmt::Layer` に一切 span を渡さず, span 由来の表示を消せます.

## Perfetto の trace file を吐く

Perfetto というのは, span/log を可視化してくれる君です. 詳しくは公式ドキュメントで.

使い始めるのは簡単です. layer を追加するだけですね.

```rust
...
    use tracing_perfetto::PerfettoLayer;

    let perfetto_tracing = match std::env::var("HOGE_PFTRACE_PATH") {
        Err(std::env::VarError::NotPresent) | Err(std::env::VarError::NotUnicode(_)) => None,
        Ok(path) => {
            let file = std::fs::File::create(path)?;
            Some(PerfettoLayer::new(std::sync::Mutex::new(file)))
        }
    };

...

    let subscriber = Registry::default()
        .with(perfetto_tracing)
        .with(
            stdout_logging
                .with_filter(env_filter.clone())
                .with_filter(NoSpanContextFilter),
        )
        .with(
            file_logging
                .with_filter(env_filter)
                .with_filter(NoSpanContextFilter),
        );

...
```

簡単とは言いましたが, 自分はハマりました. 前述の

```rust
let subscriber = Registry::default()
    .with(ここで log and/or span を記録する何か)
    .with(EnvFilter::from_default_env())
    .with(fmt::Layer::default());
subscriber.init();
```

に引っ掛かって `EnvFilter` で filtering されてしまったのです. 上で単に追加するだけで済んだのは,
既に per-filter filtering を使っていたからです.

**`tracing_subscriber::fmt().<...>.init()` で済まないときは per-layer filtering を使う.**
でないとこういう罠にハマることになります. これだけははっきりと真実を伝えたかった [^7].

## Rust での perfetto について

Perfetto は Rust の tracing とあんまり相性が良くない [^8].

Perfetto には category とか track とか flow とかの独自概念がある
[[doc](https://perfetto.dev/docs/instrumentation/track-events)].
category はまぁなくても env filter とかで代用できる.
しかし track/flow は代替手段がないし, 是非使いたい.

では Rust でのサポート状況はというと...

- [perfetto-sdk](https://github.com/google/perfetto)
  - Google 公式.
  - C++ と同様のマクロで記録.
  - [flow サポートなし.](https://github.com/google/perfetto/blob/v53.0/contrib/rust-sdk/perfetto/src/track_event.rs#L1441-L1452)
    Google 氏〜〜〜 がんばってくれ〜〜〜
- [tracing-perfetto](https://github.com/csmoe/tracing-perfetto)
  - flow サポートはできない/してないはず [^9].
- [tracing-perfetto-sdk-layer](https://github.com/modal-labs/tracing-perfetto-sdk)
  - flow サポートはできない/してないはず.
- [perfetto-recorder](https://github.com/davidlattimore/perfetto-recorder)
  - `tracing` ではない, 独自マクロで記録.
  - [tracing 系のやつらが遅いから](https://github.com/davidlattimore/perfetto-recorder?tab=readme-ov-file#performance) らしい.
  - [flow サポートなし.](https://github.com/davidlattimore/perfetto-recorder?tab=readme-ov-file#unsupported-features)

そう! 誰もサポートしていないのである! [^10]

う〜ん, 困りました. 困ったので今年はもう寝ます.

## Perfetto での trace 取得を動的に切り替える

さて, Chromium/Chrome では chrome://traring を開いて起動後に動的に recording を開始/停止することができます
[[doc](https://chromium.googlesource.com/chromium/src/+/HEAD/services/tracing/perfetto/README.md)].
当然この機能も欲しいですよね.

on/off などのインターフェースはアプリケーション毎に大きく変わるでしょうから, sabiniwm での例を見ましょう
[[cs](https://github.com/kenoss/sabiniwm/blob/37b905f1be292355dd12194a50f6ab64a1975cae/crates/sabiniwm_pistachio/src/main.rs#L32)].

```rust
fn tracing_init() -> eyre::Result<Option<ToggleFilterHandle<Registry>>> {

...

    use sabiniwm_tracing_helper::debug::ToggleFilter;

    let (perfetto_toggle_filter, perfetto_toggle_handle) = ToggleFilter::new(false);

...

    let subscriber = Registry::default()
        .with(perfetto_tracing.with_filter(perfetto_toggle_filter))
        .with(
            stdout_logging
                .with_filter(env_filter.clone())
                .with_filter(NoSpanContextFilter),
        )
        .with(
            file_logging
                .with_filter(env_filter)
                .with_filter(NoSpanContextFilter),
        );
    subscriber.init();

    Ok(Some(perfetto_toggle_handle))
}
```

もう詳しい説明は要らないでしょう. on/off 可能な filter `ToggleFilter` とその handle のペアを作り,
必要なときだけ on にして event を流すようにしています. sabiniwm ではこの handle を `Action` にして
キーバインドから実行可能にしています.

## まとめ

- `tracing_subscriber` では `Registry`, `Subscriber`, `Layer`, `Filter` が肝要.
- **`tracing_subscriber::fmt().<...>.init()` で済まないときは per-layer filtering を使う.**
- あとは `Layer` とか `Filter` を見ていけば色々やりたい拡張を実現できる.

年末ですし皆さんもご自身の tracing 設定を確認されてみてはいかがでしょうか?

Happy holidays!


[^1]: なお Seriese 2 の方の 13 日目の記事は
      [Ray Tracing from Scratch in Rust ~ 簡易CPUレンダラの自作と仕組みの勉強 ~](https://zenn.dev/oyatomo/articles/9cebd37807cd4c)
      です. tracing 違いですね.
[^2]: 実際, `tracing_subscriber` では `Registry`, `Subscriber`, `Layer`, `Filter` が肝要ですし
      公式ドキュメントもこのあたりの概念に沿って書かれていますが, `tracing_subscriber::fmt()`
      はこれら全てを覆い隠しているため「うーんドキュメントに書いてあるこれはどこ?」となります.
      なった.
[^3]: これは嘘.
[^4]: 「じゃあ『Layer』って何だよ! なんで Layer って名前なんだよ!」と思いますが, 歴史の話を追うのは
      めんどくさそうなのでやめておきます. 調べてる時間ないし. 詳しい方がいたら教えてください.
      (まぁ短絡評価したりするところとかは Layer って感じはするし, それ言ったら
      `tracing_subscriber::fmt::Layer` は Layer という名前が適切なんか?という話には, なる.)
[^5]: <https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/layer/index.html#global-filtering>
[^6]: 具体的には, sabiniwm では udev backend (TTY から起動) のときは file に書き,
      winit backend のときは stdout に出しています.
[^7]: 実際, これがこの記事を書こうと思った決め手です.
[^8]: 概念的にもはまらないし, インターフェース的にも無理な気がする?
      ライブラリで `tracing` 使ってるの自体は変更できないので, `tracing` を基本にしつつアプリ側で
      必要なところは `trace_event/trace_event_begin/trace_event_end` を使うとかが理想ではあります.
[^9]: 一応 src/perfetto.protos.rs までは来ているが, これは自動生成ってだけだろうし.
[^10]: 誰かサポート済みだったら, スマン.
