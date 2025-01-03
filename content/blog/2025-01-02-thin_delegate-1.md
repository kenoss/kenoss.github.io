+++
title = "crate thin_delegate を書いた (1/4)"
slug = "2025-01-02-thin_delegate-1"

[taxonomies]
"tags" = ["rust", "tech", "thin_delegate"]
+++

- [1章: `thin_delegate` の紹介](../2025-01-02-thin_delegate-1)
- [2章: proc macro 間での情報伝達と delegation crate 比較](../2025-01-03-thin_delegate-2)
- [3章: `thin_delegate` の設計と実装](../2025-01-04-thin_delegate-3)
- [4章: まとめ](../2025-01-05-thin_delegate-4)

## crate `thin_delegate` を書いた

[`thin_delegate`](https://github.com/kenoss/thin_delegate) という crate を書きました.
thin_delegate は **trait method [^101] を delegation によって自動生成する proc macro** を提供します.
このような crate は他にもありますが, [`sabiniwm`](https://github.com/kenoss/sabiniwm) を実装する上で色々制限があったり機能が足りませんでした.
thin_delegate は

- 制限が (ほぼ) なく使いやすい API と丁寧なエラー
- 良いデフォルト (auto impl for thin delegation) と柔軟な例外への対応
- 「合法な実装」

が特徴です.

作成にあたって色々勉強になったので記事として纏めておこうと思います. 4つの記事 (章) に分けて連投していきます.

- [1章](../2025-01-02-thin_delegate-1) (この記事) では thin_delegate が扱う問題と具体的な使い方を紹介します.
- [2章](../2025-01-03-thin_delegate-2) ではこのシチュエーションにおいて複数の proc macro 間で情報を伝達する方法の分類, および既存 crate の紹介・評価を行います.
  「合法な実装」とは何かについてもここで述べます.
- [3章](../2025-01-04-thin_delegate-3) では thin_delegate の設計および実装詳細について述べます.
- [4章](../2025-01-05-thin_delegate-4) では全体のまとめを述べます.

crate を使いたいだけであれば1章だけ見れば十分です.
2章以降は内部実装の話になります.

長くなりますが年末年始のお茶請けにでもどうぞ.

## 1章: `thin_delegate` の紹介

### 問題: trait method と delegation

delegation (委譲) とは処理を他の誰かに委譲することを指すらしいです.

[Wikipedia -- Delegation pattern](https://en.wikipedia.org/wiki/Delegation_pattern) から引用します.

```kotlin
class Rect(val width: Int, val height: Int) {
    fun area() = width * height
}

class Window(val bounds: Rect) {
    // Delegation
    fun area() = bounds.area()
}
```

この例では `Window` が `bounds: Rect` field を持ち, 明示的に `Window::area()` を `Rect::area()` に delegate
しています.


```kotlin
interface ShapeI {
    fun area(): Int
}

class Rect(val width: Int, val height: Int) : ShapeI {
    override fun area() = width * height
}

// The ShapeI implementation of Window delegates to that of the Rect that is bounds
class Window(private val bounds: Rect) : ShapeI by bounds
```

この例では `Window` が `bounds: Rect` フィールドを持ち, `ShapeI` という interface を `Rect` に delegate
しています.

厳密には 
[delegation](https://en.wikipedia.org/wiki/Delegation_pattern) と
[forwarding](https://en.wikipedia.org/wiki/Forwarding_(object-oriented_programming)) と
[proxy](https://en.wikipedia.org/wiki/Proxy_pattern) と
[facade](https://en.wikipedia.org/wiki/Facade_pattern)
は違う!もっとよく見ろ!!と言われそうですが, この記事では区別せず delegation と呼ぶことにします.
(筆者は違いを知らないし興味もない.)

Rust で書くと以下の様な感じでしょうか.

```rust
trait ShapeI {
    fn area(&self) -> f64;
}

struct Rect {
    width: f64,
    height: f64,
}

struct Window {
    rect: Rect,
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl ShapeI for Window {
    fn area(&self) -> f64 {
        ShapeI::area(&self.rect) // (1-1)
    }
}
```

trait を使っているにもかかわらず (1-1) で明示的に delegation しています.
これがこの記事の主題です. すなわち
**Rust で trait method を delegation するのを楽にしたい**
です.
簡単のため, 一連の記事では **trait method の自動 delegation** と呼称します.

この例では method がひとつなので手動でやっても別に問題にはなりません. しかし記述が煩雑になる要素はたくさんあります:

- method の数 (3個以上だとしんどい.)
- method の引数 (`self` 以外が生えた瞬間にしんどい.)
- enum variant の数 (enum になった瞬間にしんどい.)
- 継続的なメンテナンス (trait 定義を変えたり外部 crate の update で変わったりしたときに絶望.)

というわけで, 自動でできるなら自動でやった方が良いです. 面倒なことはコンピュータにやらせよう.

### 解法: `thin_delegate`

proc macro `thin_delegate` はこの問題を解決します.

```rust
#[thin_delegate::register] // (1-2)
trait ShapeI {
    fn area(&self) -> f64;
}

struct Rect {
    width: f64,
    height: f64,
}

#[thin_delegate::register] // (1-3)
struct Window {
    rect: Rect,
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

#[thin_delegate::fill_delegate]
impl ShapeI for Window {} // (1-4)
```

まず trait 定義 (1-2) と struct 定義 (1-3) に `#[thin_delegate::register]` を付けます.
そうすると (1-4) の `impl ShapeI for Window {}` のところで `#[thin_delegate::fill_delegate]`
が自動で (1-1) の様なコードを生成してくれます [^102].

thin_delegate には便利な機能がいくつかありますが, 主要なものを紹介します.

- ergonomic [^103]
- エラーが読みやすい [^104]
- Generics, trait bound, GATs などの制限なし (のはず. 駄目なケースが発見されてないだけ [^105].)
- 外部 crate の trait 定義を利用する (`external_trait_def`)
  - [基本](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_external_trait_def.rs)
  - [`with_uses` で自動で `use` する](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_external_trait_def_with_uses.rs)
- メソッド生成ルールを弄って自明じゃない delegation をする (`scheme`) [^106]
  - [struct](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_scheme.rs)
  - [enum](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_scheme_enum.rs)
- [手動で実装したい部分は手動で実装可能](https://github.com/kenoss/thin_delegate/blob/main/tests/ui/pass_items_in_impl.rs) [^107]

基本的な使い方は [ドキュメント](https://docs.rs/thin_delegate/latest/thin_delegate/) を読んでください.

関連する RFC として
[rfcs#1406](https://github.com/rust-lang/rfcs/pull/1406),
[rfcs#2393](https://github.com/rust-lang/rfcs/pull/2393),
[rfcs#3530](https://github.com/rust-lang/rfcs/pull/3530)
があります.

### Real world example: sabiniwm

実際の使い方として sabiniwm (commit [1904a51](https://github.com/kenoss/sabiniwm/tree/ed5d2c9)) を見てみます.

#### external_trait_def

まず基本的に sabiniwm は smithay などの外部 crate の trait をたくさん使っています.
delegate 対象の外部 crate の trait は
[crates/sabiniwm/src/external_trait_def.rs](https://github.com/kenoss/sabiniwm/blob/1904a51bd3346154a60551cda70f6bfd0f3b63f2/crates/sabiniwm/src/external_trait_def.rs)
で管理しています. `mod` に `#[thin_delegate::external_trait_def(with_uses = true)]` を付け,

1. この `mod` は外部 crate の trait をコピペするための場所であること,
2. `#[thin_delegate::fill_delegate]` したときに自動で `use` すべきこと,

を宣言しています. その中に `#[thin_delegate::register]` を書いていきます.

trait の定義が更新されたときは `external_trait_def` の該当部分をコピペで更新するだけです.

#### focus.rs

実際に利用しているのは例えば
[crates/sabiniwm/src/focus.rs](https://github.com/kenoss/sabiniwm/blob/1904a51bd3346154a60551cda70f6bfd0f3b63f2/crates/sabiniwm/src/focus.rs)
です.

```rust
#[derive(derive_more::From, Debug, Clone, PartialEq)]
#[thin_delegate::register]
pub enum PointerFocusTarget {
    WlSurface(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface),
    X11Surface(smithay::xwayland::X11Surface),
}

#[thin_delegate::fill_delegate(external_trait_def = crate::external_trait_def::smithay::utils)]
impl smithay::utils::IsAlive for PointerFocusTarget {}

#[thin_delegate::fill_delegate(external_trait_def = crate::external_trait_def::smithay::input::pointer)]
impl smithay::input::pointer::PointerTarget<SabiniwmState> for PointerFocusTarget {}

#[thin_delegate::fill_delegate(external_trait_def = crate::external_trait_def::smithay::input::touch)]
impl smithay::input::touch::TouchTarget<SabiniwmState> for PointerFocusTarget {}

#[thin_delegate::fill_delegate(external_trait_def = crate::external_trait_def::smithay::wayland::seet)]
impl smithay::wayland::seat::WaylandFocus for PointerFocusTarget {}
```

これは単純ですね. `PointerFocusTarget` に対して例えば `smithay::input::pointer::PointerTarget` を impl するために trait 定義を
`crate::external_trait_def::smithay::input::pointer` から引いています.

```rust
#[derive(derive_more::From, Debug, Clone, PartialEq)]
#[thin_delegate::register]
pub enum KeyboardFocusTarget {
    Window(smithay::desktop::Window),
    LayerSurface(smithay::desktop::LayerSurface),
    Popup(smithay::desktop::PopupKind),
}

#[thin_delegate::fill_delegate(external_trait_def = crate::external_trait_def::smithay::utils)]
impl smithay::utils::IsAlive for KeyboardFocusTarget {}

#[thin_delegate::fill_delegate(
    external_trait_def = crate::external_trait_def::smithay::input::keyboard,
    scheme = |f| {
        match self {
            Self::Window(w) => match w.underlying_surface() { // (1-5)
                smithay::desktop::WindowSurface::Wayland(s) => f(s.wl_surface()),
                smithay::desktop::WindowSurface::X11(s) => f(s),
            }
            Self::LayerSurface(l) => f(l.wl_surface()),
            Self::Popup(p) => f(p.wl_surface()),
        }
    }
)]
impl smithay::input::keyboard::KeyboardTarget<SabiniwmState> for KeyboardFocusTarget {}
```

こちらは `impl smithay::input::keyboard::KeyboardTarget` に対して `scheme` が指定されています.
(1-5) のところで `Window` の arm に対して `smithay::desktop::Window::underlying_surface()` で分岐する必要があるからです.
(これは更に wrapper struct を噛ませてもいいのですが, 結局似たようなものになるのでこうしています.)

ちなみに [`KeyboardTarget` の定義](https://smithay.github.io/smithay/smithay/input/keyboard/trait.KeyboardTarget.html)
は手で管理したくない程度には複雑です.

(sabiniwm は anvil の fork であり) このファイルは
[smithay/anvil/src/focus.rs](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/focus.rs)
を元にしています. 書き換えの履歴 (一部抜粋
[78f082c](https://github.com/kenoss/sabiniwm/commit/78f082cf0b10630d2af0c5fc1ec94209125ffaa0),
[ffcb2dc](https://github.com/kenoss/sabiniwm/commit/ffcb2dc59a7a046ae4618eeb60571145cd3f7688),
[ed5d2c9](https://github.com/kenoss/sabiniwm/commit/ed5d2c9cc199dd3358b33e8ef700e31fc39ea6ed)
) を追うと人間が管理可能になっていく様子がわかると思います.

#### sabiniwm::backend::BackendI

[crates/sabiniwm/src/backend/mod.rs](https://github.com/kenoss/sabiniwm/blob/1904a51bd3346154a60551cda70f6bfd0f3b63f2/crates/sabiniwm/src/backend/mod.rs)

smithay を利用した Wayland compositor は udev/winit backend をサポートしているものが多いです. (winit は開発用.)
これを切り替えるために
[`AnvilState<UdevData>` の様に型パラメータを利用](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/udev.rs#L158)
しています.
`AnvilState` を書くときに常に型パラメータと trait bound を書く必要があって非常にめんどいのと,
これが anvil/src/udev.rs, anvil/src/winit.rs, anvil/src/main.rs で ベタ書き
[[1](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/udev.rs#L844)]
[[2](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/main.rs#L45)]
されている理由のひとつです.

niri では [enum で定義](https://github.com/YaLTeR/niri/blob/v0.1.10.1/src/backend/mod.rs) されています.

sabiniwm では一度 `Box<dyn BackendI>` を経て
[[1](https://github.com/kenoss/sabiniwm/commit/e80231c493943dc533f1a9c95489723726920af8)]
[[2](https://github.com/kenoss/sabiniwm/commit/b00910d8aa6b2e0cf601c025f247d58e0b2d7500)],
最終的に
[enum](https://github.com/kenoss/sabiniwm/blob/1904a51bd3346154a60551cda70f6bfd0f3b63f2/crates/sabiniwm/src/backend/mod.rs)
での実装となりました [^108].

見所としては `scheme` を使うことで (1-6) の `#[cfg(feature = "winit")]` の分岐がちゃんと扱えていることでしょうか [^109]:

```rust
#[derive(derive_more::From)]
#[thin_delegate::register]
pub(crate) enum Backend {
    Udev(udev::UdevBackend),
    #[cfg(feature = "winit")]
    Winit(winit::WinitBackend),
}

#[thin_delegate::fill_delegate(
    external_trait_def = crate::external_trait_def::smithay::wayland::buffer,
    scheme = |f| {
        match self {
            Self::Udev(backend) => f(backend),
            #[cfg(feature = "winit")]           // (1-6)
            Self::Winit(backend) => f(backend),
        }
    }
)]
impl smithay::wayland::buffer::BufferHandler for Backend {}
```

#### render_elements!

smithay では `render_elements!` というマクロで
[描画要素を定義します](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/drawing.rs#L53-L57).
定義は [これ](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/src/backend/renderer/element/mod.rs#L1333).
(人間には読めん...)

niri はこれを [人間が読める](https://github.com/YaLTeR/niri/blob/v0.1.10.1/src/render_helpers/render_elements.rs) ようにしています.
(偉すぎる...)

必須なのは delegation する部分だけなのでこれも
[thin_delegate で書き換えています](https://github.com/kenoss/sabiniwm/blob/7a89f6dfc4cc46db66ea5e966e59db4ac500f6bd/crates/sabiniwm/src/render.rs#L12-L38).

```rust
#[derive(derive_more::From)]
#[thin_delegate::register]
pub enum CustomRenderElement<R>
where
    R: Renderer,
{
    Pointer(PointerRenderElement<R>),
    Surface(WaylandSurfaceRenderElement<R>),
}

#[thin_delegate::fill_delegate(external_trait_def = crate::external_trait_def::smithay::backend::renderer::element)]
impl<R> smithay::backend::renderer::element::Element for CustomRenderElement<R>
where
    R: smithay::backend::renderer::Renderer,
    <R as smithay::backend::renderer::Renderer>::TextureId: 'static,
    R: ImportAll + ImportMem,
{
}

#[thin_delegate::fill_delegate(external_trait_def = crate::external_trait_def::smithay::backend::renderer::element)]
impl<R> smithay::backend::renderer::element::RenderElement<R> for CustomRenderElement<R>
where
    R: smithay::backend::renderer::Renderer,
    <R as smithay::backend::renderer::Renderer>::TextureId: 'static,
    R: ImportAll + ImportMem,
{
}
```

複雑な trait bound であってもちゃんと動いている!! [^110] 良いですね.

### 1章まとめ

- 問題: Rust で trait method を delegation するのを楽にしたい
- 解法: `thin_delegate` という crate を書いたのでそれが使える
- 既に `sabiniwm` で色々な使い方をしている

[2章](../2025-01-03-thin_delegate-2) からは内部実装の話になります.


[^101]: 正確には trait function/associated function である. <https://doc.rust-lang.org/reference/items/traits.html>
        しかし method の方が通りが良いのでこの記事ではそう呼ぶ.
[^102]: 「これって既にある crate `<hoge>` ですよね?」そうとも言えるしそうとも言えない. 2章と3章で議論します.
[^103]: 完全に主観でしかない.
[^104]: 完全に主観でしかない. 3章で議論します.
[^105]: GitHub issue での報告やテストの追加などお待ちしております.
[^106]: 「これって既にある crate [`delegate`](https://crates.io/crates/delegate) ですよね?」アイデア元はそう. 2章と3章で議論します.
[^107]: 「これって既にある crate [`portrait`](https://crates.io/crates/portrait) ですよね?」結果的にはそう. 2章と3章で議論します.
[^108]: このあたりの niri と sabiniwm の比較はまた別の記事で. enum でという点では似ていますがそこそこ違います.
[^109]: この分岐は分岐予測でほぼ 100% 当たるから feature flag で消すのは趣味でしかないですね. most-inner loop でもないだろうし. まぁデモンストレーションだと思ってください.
[^110]: 推測ですが, smithay がこうしているのは当時複雑な trait bound に対して enum を delegation できる crate がなかったのが要因のひとつでしょう. (しらんけど)
      パワーで解決できるところはパワーで解決してこだわらない. それもまた正解ですしね.
