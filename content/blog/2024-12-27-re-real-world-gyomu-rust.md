+++
title = "Re: RealWorld 業務 Rust (業務以外編)"
slug = "2024-12-27-re-real-world-gyomu-rust"

[taxonomies]
"tags" = ["rust"]
+++

## これは何?

[RealWorld 業務 Rust](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94)
に乗っかって普段から考えているなんやかんやを参照可能にしておく. 主に情報の補足と極論の補正 [^1].
但し業務というコンテキストは外してだいたいいつでも適用できるようにする.

legokichi さんの考えには少なからず影響されていることに注意.

皆も自分なりの考えを書こう!

([これ](https://x.com/keno_ss/status/1775142778777039141) とかもどこかにまとめたいわね...)

## Re: [docker でビルドできるようにしとけ](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#docker-%E3%81%A7%E3%83%93%E3%83%AB%E3%83%89%E3%81%A7%E3%81%8D%E3%82%8B%E3%82%88%E3%81%86%E3%81%AB%E3%81%97%E3%81%A8%E3%81%91)

docker でビルドできるようにしておくこと自体は大事なんだけど, 最近は Asahi Linux (aarch64) があり
docker が絶対とは言えなくなってきた. そんなマイナー環境使う方が悪い? それはそう... (なので x86_64 環境も持っている.)

なお今年の ISUCON14 では [`cross`](https://github.com/cross-rs/cross) を使った.
メンバーが Linux (x86_64), MacOS (Apple Silicon), Linux (aarch64+x86_64) だったので [^2].

## Re: [cargo fmt](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#ci-%E3%81%A7-cargo-fmt-%E3%82%92%E3%83%81%E3%82%A7%E3%83%83%E3%82%AF%E3%81%97%E3%82%8D) and [clippy](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#ci-%E3%81%A7-cargo-clippy---tests---examples-----dclippyall-%E3%81%97%E3%82%8D)

割と有名所が使ってなくて辛い.

補足として, 単に `cargo fmt` や `cargo clippy` ではなく何らかのフラグを付けるなら [`just`](https://github.com/casey/just)
か何かを使って CI と手動での結果を一致させるべき.
([過去にそういうリポジトリがあった](https://x.com/keno_ss/status/1796144317502759082).)

`RUSTFLAGS='-D warnings'` を付けると warning を絶許にできるのでおすすめ.
但し `RUSTFLAGS` を使ったり弄ったりすると再ビルドすることになる. なので `CARGO_TARGET_DIR` を併用するとべんり:

```
export CARGO_TARGET_DIR=target/check-strict RUSTFLAGS='-D warnings'; cargo build && cargo clippy --tests --examples && cargo fmt -- --check
```

僕はこれを [justfile に check-strict で登録](https://github.com/kenoss/thin_delegate/blob/v0.0.3/justfile#L1-L5) [^3] しておいて
`cargo watch -c -s 'cargo build && cargo nextest run --status-level=fail && just check-strict'` してる.
(具体的なコマンドは場合によって変える.)

これは `cargo fmt` だけではなく build/test などについても言える.
関る全員が常にシンプルなコマンド群の下で作業できるようにするべきだ.

ところでオレオレフォーマットを主張したくないから
[rustfmt にバグ修正の PR を投げている](https://github.com/rust-lang/rustfmt/pull/6165)
んだけど review bandwidth がないと言われています. わりと困り, 困る.
(いやまぁ向こうの事情も汲むが, このサイズの PR でそう言われてもな...)

## Re: [コンパイルエラーは一番上のエラーから順番に直せ](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#%E3%82%B3%E3%83%B3%E3%83%91%E3%82%A4%E3%83%AB%E3%82%A8%E3%83%A9%E3%83%BC%E3%81%AF%E4%B8%80%E7%95%AA%E4%B8%8A%E3%81%AE%E3%82%A8%E3%83%A9%E3%83%BC%E3%81%8B%E3%82%89%E9%A0%86%E7%95%AA%E3%81%AB%E7%9B%B4%E3%81%9B)

`head` については `just check-warn` で見えるようにしたけどあんまり使ってないな...
安定期に入るとそこまで長いエラーが出なくなって, `cargo watch` で `clear` と `tmux` 上連打で十分になってしまった.
(`head` だと肝心の部分が途切れたりして二度手間.)

## Re: [rust-analyzer は頼りにならない](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#rust-analyzer-%E3%81%AF%E9%A0%BC%E3%82%8A%E3%81%AB%E3%81%AA%E3%82%89%E3%81%AA%E3%81%84)

前述の通り `cargo watch` を使っている.
`alacritty/tmux (git 操作とか手動でテストとか)`, `emacs`, `alacritty/tmux (cargo watch とか)` と並べている.

このやり方の利点はどこでもどんなプロジェクトでも応用が効くことだ. LSP を使えない荒野はたくさんある.
(いや, 使えるときは使った方が良いとは思う. 僕は設定するのが面倒で先延ばしにしているだけ...)

## Re: [`Result<Result<T, CustomError>, anyhow::Error>`](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#%E3%82%A8%E3%83%A9%E3%83%BC%E3%81%AE%E5%9E%8B%E3%81%AF-resultresultt-customerror-anyhowerror-%E3%81%A7fa), [backtrace](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#%E6%9C%AC%E7%95%AA%E7%92%B0%E5%A2%83%E3%81%A7%E3%82%82%E5%B8%B8%E3%81%AB-rust_backtrace1-%E3%81%A7%E5%AE%9F%E8%A1%8C%E3%81%97%E3%82%8D)

Rust は `Result` や `Option` があるが, 現実的に panic をなくせるわけではない.
例えば index access `xs[i]` で panic するケースはプログラムのバグだから回復不可能である. なので (普通) panic するべきだ.
偉人たちも panic すべきときは panic せよと言っている.
[[BurntSushi](https://burntsushi.net/unwrap/)]
[[lo48576](https://blog.cardina1.red/2019/12/19/dont-fear-the-panic/)]
しなければ逆に全てが `Result` になる.

もちろん `Result<Result<T, CustomError>, anyhow::Error>` の様にハンドリングすべきときも確かにある.
HTTP サーバーとか. しかしそれが全てではない.

僕は panic が許容できるアプリケーションを書くときは
[こんな風に `std::panic::set_hook()` を使って `tracing::error!()` する](https://github.com/kenoss/sabiniwm/blob/ed5d2c9cc199dd3358b33e8ef700e31fc39ea6ed/crates/sabiniwm/src/util/panic.rs#L7)
ようにしている. 例えば sabiniwm では
[udev backend で動作しているときは `/tmp/sabiniwm.log` に向けて `tracing_subscriber` を設定しておき](https://github.com/kenoss/sabiniwm/blob/ed5d2c9cc199dd3358b33e8ef700e31fc39ea6ed/crates/sabiniwm-pistachio/src/main.rs#L45-L47),
ビルドマシンで実行してそれを Asahi Linux 側から `ssh` して `tail -F /tmp/sabiniwm.log` して見るようにしている.
(TTY から起動していると console がぶっこわれて読めないことがあるし, tmux 噛ますと Wayland compositor を起動できないため.
Asahi Linux で実働させているときもデバッグ用に使える.)
マルチスレッドプログラムの場合はここで全イベントループを停止させるのもやっている [^4].

ISUCON14 でも (otel と合わせて) これで効率的にデバッグできたケースがあった. (絶対に成功する `HashMap::get().unwrap()` を書いたつもりだったが仮定が偽だった.)

まぁでも業務 HTTP サーバーだと結局 Sentry とかを使えという話になるのかも.

## Re: [`?`](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#huga-%E3%81%BF%E3%81%9F%E3%81%84%E3%81%AA-the-question-mark-operator-%E3%82%92%E3%81%9D%E3%81%AE%E3%81%BE%E3%81%BE%E4%BD%BF%E3%81%86%E3%81%AA)

なお最近は `anyhow` も `eyre` も backtrace を capture する.
[[`anyhow`](https://github.com/dtolnay/anyhow/blob/1.0.95/README.md?plain=1#L80)]
[[`color-eyre`](https://github.com/eyre-rs/eyre/blob/v0.6.8/src/lib.rs#L20)]
`.context()`/`.wrap_err()` はあくまで情報を付け足すべきときに使うで良い.
(でも僕はまだ (上述の `set_hook()` を書いた 2023 時点では) `std::backtrace` に満足できていなかったので `backtrace` crate を使っているよ...)

thiserror は 
[`#[backtrace]` attribute でマークされている](https://github.com/dtolnay/thiserror/blob/master/README.md?plain=1#L140)
場合に capture してくれる.
しかしこれは nightly で `#![feature(error_generic_member_access)]` を指定する必要がある.
仕方がないので ISUCON14 では以下の様にした:

```rust
// lib.rs

// Cargo.toml で feature "debug" を追加しておく.

#![cfg_attr(feature = "debug", feature(error_generic_member_access))]

#[cfg(not(feature = "debug"))]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
...
}
#[cfg(feature = "debug")]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLx error: {0}, backtrace = \n{1}")]
    Sqlx(#[from] sqlx::Error, std::backtrace::Backtrace),
...
}
```

```justfile
# justfile

deploy:
  cd webapp/rust && cross build --release
  just deploy-aux

deploy-debug:
  cd webapp/rust && cross +nightly build --features debug
  rm webapp/rust/target/x86_64-unknown-linux-gnu/release/isuride
  mv webapp/rust/target/x86_64-unknown-linux-gnu/{debug,release}/isuride
  just deploy-aux
```

早く安定化してくれ〜〜〜

## 軽率に全てを `use` するな派 それがその file/module で unique で複数回出てきてかつ pub trait 定義の中でなければまぁいいんじゃね穏健派

Re: `use` について [[1](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#alias-%E3%81%AF%E4%BD%BF%E3%81%86%E3%81%AA)][[2](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%85%88%E9%A0%AD%E3%81%A7-use-%E3%81%AF%E4%BD%BF%E3%81%86%E3%81%AA)][[3](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB%E5%85%88%E9%A0%AD%E3%81%A7-use-%E3%81%AF%E4%BD%BF%E3%81%86%E3%81%AA)]

ところで、[この `use`](https://github.com/Smithay/smithay/blob/2eddf12d296b8930fd882d30366b8b9a683aae75/anvil/src/udev.rs) を見てくれ。こいつをどう思う？  
すごく・・・大きいです・・・ [^5]

この中で `Device` 概念は少なくとも

- [`smithay::backend::input::Device`](https://docs.rs/smithay/latest/smithay/backend/input/trait.Device.html)
- [`smithay::reexports::drm::Device`](https://docs.rs/drm/latest/drm/trait.Device.html)
- [`smithay::reexports::drm::control::Device`](https://docs.rs/drm/latest/drm/control/trait.Device.html)
- [`smithay::reexports::gbm::Device`](https://docs.rs/gbm/latest/gbm/struct.Device.html)
- [`smithay::reexports::input::Device`](https://docs.rs/input/latest/input/struct.Device.html)
- [`smithay::reexports::libseat::Device`](https://docs.rs/libseat/latest/libseat/struct.Device.html)
- [`smithay::reexports::udev::Device`](https://docs.rs/udev/latest/udev/struct.Device.html)

がある. マジわけわかんねぇ!!

だからデフォルトでは full path で書いてくれ [^6].

短い名前で `use` するための僕の判断基準は概ね

- その概念が file/module で unique である; and
- それが複数回使われる; and
- not (pub trait の中である).

`Mutex` とか `crossbeam::channel` とかは別にいいんじゃないですかね? async/await を使っているか否かというのはそのアプリケーションの方針から
unique に判定可能で頭にキャッシュされていると期待できるから. (違う方を使ったときだけ full path で書けばよい.)
でも [tokio の channel は複数ある](https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html?search=channel) ので許されない.
`Result` はケースバイケースかな.

`log`/`traicng` は
[{lib,main}.rs の先頭で `#[macro_use]` している](https://github.com/kenoss/sabiniwm/blob/ed5d2c9cc199dd3358b33e8ef700e31fc39ea6ed/crates/sabiniwm/src/lib.rs#L5-L7).
`log`/`tracing` を使わない奴はいない.
デバッグ中に「ここに `info!(...)` を書きたいなぁ〜」と思ったときに `use しろ` と言われるとささくれにひっかかった気分になる.
ISUCON の参考実装でもこうしてくれ〜〜〜

最後のやつは trait をコピペするためである.
[このへん](https://github.com/kenoss/sabiniwm/blob/ed5d2c9cc199dd3358b33e8ef700e31fc39ea6ed/crates/sabiniwm/src/config.rs#L10).
でも他人 (のリポジトリ) は制御できないから
[こういう対応](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_external_trait_def_with_uses.rs#L28-L55)
をせざるを得ない.

関数/block の中ならそこそこゆるゆるで OK.

## Re: [macro](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#%E3%83%9E%E3%82%AF%E3%83%AD%E3%81%AF%E4%BD%BF%E3%81%86%E3%81%AA)/[trait](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#%E3%82%AA%E3%83%AC%E3%82%AA%E3%83%AC-trait-%E3%81%AF%E4%BD%BF%E3%81%86%E3%81%AA) は使うな

ところで [こいつを](https://github.com/Smithay/smithay/blob/2eddf12d296b8930fd882d30366b8b9a683aae75/src/backend/renderer/element/mod.rs#L1348) (ry

とても人間には読めないとは思うのだけれど, これを書いた人にはそれなりの事情があるのだと思う.
(特に業務では) 理想的ではなくともパワーで解決すべきときというのは確かにあり, そういうのを否定する気持ちにはなれない.

いつ YAGNI/KISS に倒すか/パワーで解決するか/抽象化で解決するかというのは個別の例を見ないとなんとも言えない.

legokichi さんには legokichi さんの文脈があるんだけれども, 業務だとうかつにコード出せないから残念...

チームで対話しろ. style guide を書け. Pros/cons を明確に言語化して「決め」の合意を取れ.
[[example 1](https://google.github.io/styleguide/cppguide.html)]
[[example 2](https://chromium.googlesource.com/chromium/src/+/main/styleguide/c++/c++.md)]
最初は完璧でなくてもよい. 誰かが明確化したいと思ったときの障壁をあらかじめ取り除いておけ.

(というわけで皆も自分の判断基準を書こう! 例を出せる人は是非例を挙げて.)

## Re: [Builder Pattern はクソ](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#builder-pattern-%E3%81%AF%E3%82%AF%E3%82%BD)

[「でも Xaeroxe 氏って確か Builder Pattern に鞍替えしたんだよね」](https://xaeroxe.github.io/init-struct-pattern/#:~:text=UPDATE)
と記憶してたけど, [rfcs#0736](https://github.com/rust-lang/rfcs/blob/master/text/0736-privacy-respecting-fru.md)
って Init Struct Pattern 関係なくない...? (Init struct は全フィールド `pub` なはずなので.)

## Re: [println するな log::debug しろ](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#println-%E3%81%99%E3%82%8B%E3%81%AA-logdebug-%E3%81%97%E3%82%8D)

最近なら `tracing` crate を使うでいい気がする. span も扱えるし [^7].

たま〜に `tracing_subscriber` の設定が面倒で移行してないという話を聞く.
こんな設定に著作権は発生しない [^8] ので適当にそのへんのアプリケーションからコピペしてくればよろしい [^9].

`debug!` より `info!` か `warn!` を使うことが多いかなぁ.
雑に `RUST_LOG=debug` と書くと依存 crate の `debug!` が出てきて面倒だったりするので.

## Re: [`let _ = hoge()` による `_` 束縛は使うな `_hoge` みたいに名前をつけろ](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#let-_--hoge-%E3%81%AB%E3%82%88%E3%82%8B-_-%E6%9D%9F%E7%B8%9B%E3%81%AF%E4%BD%BF%E3%81%86%E3%81%AA-_hoge-%E3%81%BF%E3%81%9F%E3%81%84%E3%81%AB%E5%90%8D%E5%89%8D%E3%82%92%E3%81%A4%E3%81%91%E3%82%8D)

でも現代においては全人類
[pandaman さんの記事](https://blog.idein.jp/post/644161837029605376/rust%E3%81%AB%E3%81%8A%E3%81%91%E3%82%8Birrefutable-pattern%E3%82%92%E4%BD%BF%E3%81%A3%E3%81%9F%E3%82%A4%E3%83%87%E3%82%A3%E3%82%AA%E3%83%A0)
を読んでるから別によくね?


[^1]: これは僕の Rust 歴が浅いという理由が大きそう. 始めたのは 2019/10 あたりだったから async/await が stable に来る直前だったし
      [error 周り](https://qiita.com/legokichi/items/d4819f7d464c0d2ce2b8) も thiserror/anyhow を使うで決着ついてる頃
      (な気がする)
[^2]: ISUCON14 反省記事は...感想戦が終わったら出るかもしれないし出ないかもしれない.
[^3]: 今見たら `--tests --examples` が付いていませんでした. 申し訳ない...
[^4]: 連鎖できる cancellation token みたいなやつの root をここで作っておき, panic_hook に入ったときに cancel する.
      自作テキストエディタと自作 watchexec でしか使ってないから例が出せない...
[^5]: と思ったが legokichi さんも smithay を例に挙げていた. わかるよ...
[^6]: [ごめんまだ直してなかった](https://github.com/kenoss/sabiniwm/blob/ed5d2c9cc199dd3358b33e8ef700e31fc39ea6ed/crates/sabiniwm/src/backend/udev.rs#L47-L48).
      直します...
[^7]: でも実際に可視化するのはめんどい.
      [このへん](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp-http/src/main.rs)
      を参考にして `OpenTelemetryTracingBridge` を使いつつ OTLP で吐くのが良い気がするのだけど,
      業務 Rust HTTP サーバーのことはなんもわからん...
[^8]: 「思想又は感情を創作的に表現したものであって、文芸、学術、美術又は音楽の範囲に属するもの」に該当するとは思えない.
      礼を尽したいのであれば author に一声かければよい. 皆快諾してくれるでしょう.
[^9]: 僕はめんどくさかったので調べて書きましたが...
