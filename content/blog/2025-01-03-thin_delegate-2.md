+++
title = "crate thin_delegate を書いた (2/4)"
slug = "2025-01-03-thin_delegate-2"

[taxonomies]
"tags" = ["rust", "tech", "thin_delegate"]
+++

- [1章: `thin_delegate` の紹介](../2025-01-02-thin_delegate-1)
- [2章: proc macro 間での情報伝達と delegation crate 比較](../2025-01-03-thin_delegate-2)
  - [問題: proc macro 間での情報伝達](./#problem-definition-share-data-between-proc-macro)
  - [`enum_dispatch`](./#enum-dispatch)
  - [閑話休題: proc macro で過激なことをやりたい](#local-state-for-proc-macro)
  - [`enum_delegate` v0.2.0](./#enum-delegate-v0.2.0)
  - [`auto-delegate`](./#auto-delegate)
  - [`enum_delegate` v0.3.0](./#enum-delegate-v0.3.0)
  - [`ambassador`](./#ambassador)
  - [`portrait`](./#portrait)
  - [`delegate`](./#delegate)
  - [`delegate-attr`](./#delegate-attr)
  - [2章まとめ](./#summary)
- [3章: `thin_delegate` の設計と実装](../2025-01-04-thin_delegate-3)
- [4章: まとめ](../2025-01-05-thin_delegate-4)

## 2章: proc macro 間での情報伝達と delegation crate 比較

1章では基本的な使い方を見ました.
この章ではいくつかの技術的な課題のうち, 最も重要な **proc macro 間での情報伝達方法** を扱います.
またそれに伴い, **既存の delegation 系 crate の比較** も行います.

構文的な良し悪しはこの章では判断しません. 3章でまとめて扱います.

### 問題: proc macro 間での情報伝達 {#problem-definition-share-data-between-proc-macro}

thin_delegate の基本的な使い方は以下の様なものでした.

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

(1-2) で trait `ScoledShape`, (1-3) で struct `Window` の情報を覚え, (1-4) でこれらの情報を利用してメソッドを自動実装しています.
つまり, **複数の proc macro 間で trait や型に紐付く情報を伝達**しています.
また, この問題は **外部 crate の trait の扱い** も要素として絡んできます.

さて, これはどうやるのでしょうか?

この問題へのアプローチ方法は crate 毎に異なります. なので既存の具体的な crate を挙げ, それの実装方針を見ることにしましょう.

### `enum_dispatch` {#enum-dispatch}

- [crates.io](https://crates.io/crates/enum_dispatch)
- [repository](https://gitlab.com/antonok/enum_dispatch)
- [doc](https://docs.rs/enum_dispatch/latest/enum_dispatch)
- Initial commit: 2018/12

enum_dispatch は (僕が知る限り) trait method の自動 delegation を扱った最古の crate です.
crates.io で daily (?) ダウンロード35k越えで, Twitter でも度々目にします. 自分もこれを最初に手に取りました.

enum_dispatch の基本的な使い方は以下です.

```rust
#[enum_dispatch::enum_dispatch] // (2-1)
trait ShapeI {
    fn area(&self) -> f64;
}

struct Rect {
    width: f64,
    height: f64,
}

#[enum_dispatch::enum_dispatch(ShapeI)] // (2-2)
enum Shape {
    Rect(Rect),
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {
    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6.0);
}
```

(2-1) で trait 定義の情報を登録し, (2-2) で enum 定義と自動 delegation を同時に行っています.
thin_delegate の様に `impl` は不要です. また, trait と enum の順番を変えることもできます.

この情報伝達は
**proc macro crate 中の global variables に文字列化した trait/enum の定義を保存する**
ことによって行われます. より詳細には [^201]:

1. (2-1) の trait `ShapeI` に対する `enum_dispatch::enum_dispatch` 呼び出しにおいて,
   trait `ShapeI` の token stream を文字列化したものを `HashMap` `TRAIT_DEFS` に保存する
   [[code](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/lib.rs?ref_type=tags#L378)]
   [[`TRAIT_DEFS`](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/cache.rs?ref_type=tags#L38-39)].
   このとき generics の情報も一部 `HashMap` のキーとして含める
   [[code](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/cache.rs?ref_type=tags#L49-50)].
2. (2-2) の enum `Shape` に対する `enum_dispatch::enum_dispatch` 呼び出しにおいて,
   enum `Shape` の token stream を文字列化したものを `HashMap` `ENUM_DEFS` に保存する
   [[code](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/lib.rs?ref_type=tags#L382)]
   [[`ENUM_DEFS`](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/cache.rs?ref_type=tags#L40-41)].
   このとき generics の情報も一部 `HashMap` のキーとして含める
   [[code](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/cache.rs?ref_type=tags#L59-60)].
3. (2-1), (2-2) で引数として enum/trait 名が与えられていた場合 (上では (2-2)), その組が両方定義されていれば自動 delegation する.
   そうでなければ [^201] defer する
   [[code](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/lib.rs?ref_type=tags#L421-430)].
4. (2-1), (2-2) で defer した組が解決されたとき, 自動 delegation する [^202]
   [[code](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/lib.rs?ref_type=tags#L440-457)].

最初にこの処理を見たときは感動しました. 「これって合法なんだ!!」と.
でも実は合法ではありませんでした.

[What do you suggest proc macros to be aware of each other as opposed to a static variable?](https://users.rust-lang.org/t/what-do-you-suggest-proc-macros-to-be-aware-of-each-other-as-opposed-to-a-static-variable/77160/6)
から引用:

> bjorn3  
> Jun 2022
> 
> Proc macros should be expandable independently and without any side effects. We don't offer any guarantees around the order and amount of times a proc macro is expanded. For example rust-analyzer will only re-expand proc macros if you change the actual invocation of the proc macro. Rustc could also start caching proc macro outputs in the future I think.

意約:

> proc macro は副作用を持たずに独立して展開されるべきです.
> proc macro の展開においてその順序も呼び出し回数も何も保証されません.
> 例えば rust-analyzer は proc macro の呼び出し箇所の変更毎に再展開されます.
> 将来的には rustc が proc macro 出力をキャッシュすることも考えられます [^203].

これに照らして上記の方法を評価してみましょう.

- そもそも proc macro 呼び出し時 global variable がどういう状態か何も保証されていない.
  (例えば呼び出し毎に新しいプロセスを起動するのはコンパイラの挙動として合法.)
  以下では状態が保存されていて一部 rustc と挙動が違うと仮定する.
- 順序不定: defer していることにより trait/enum の順序が逆になっても問題ない (はず).
- 複数回呼び出し: `cache_trait()`, `cache_enum_dispatch()` は
  [`HashMap::insert()`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.insert)
  を利用しており, key が重複する場合は更新されるので中身の変更は大丈夫 (なはず).
  trait/enum 名や型パラメータを弄った場合は古い定義が別エントリとして残るので駄目.
  また, (2-2) -> (2-1) -> (2-1) で評価された場合最初の (2-1) で defer が
  [消費](https://gitlab.com/antonok/enum_dispatch/-/blob/v0.3.13/src/cache.rs?ref_type=tags#L131-134)
  されているから駄目 (なはず) [^204].
- キャッシュ: キャッシュ単体で異常ケースはない気がする? (まぁでも副作用なしという重大な前提条件が満たされていないので...)

...と, 全体的によろしくないです.

---

ふたつめの enum_dispatch の良くない部分として, 外部 crate の trait をサポートしていないということが挙げられます.

ある crate で `enum_dispatch::enum_dispatch` を使い
[[1](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/src/lib.rs)],
別の crate で同じ identifier に対して使っても
[[2](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/pass_basic.rs)]
[[3](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/pass_def_in_other_mod_without_arg_incorrect.rs)]
それ自体ではエラーにはなりません.
一方, 同一 crate 内で同じ identifier に対して使うと意図せぬ挙動になります
[[4 rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_def_in_other_mod_with_arg.rs)]
[[4 stdeerr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_def_in_other_mod_with_arg.stderr)].

このエラーは trait/enum の identifier (名前) を `HashMap` のキーとして利用し, それを module 毎に区別していないのが原因です.
(実装がナイーヴすぎる.)

この差は compilation unit (crate) を跨いでいるときに global variable を引き継いでいないからだと思われます [^205].
逆に言えば, **引き継げない** ということでもあります.

というわけで, **global variable を使う方法では異なる proc macro 間で情報を伝達できません**.

---

みっつめ. generics や trait bound などのサポートが弱い.

```rust
struct Rect<T> {
    width: T,
    height: T,
}

#[enum_dispatch::enum_dispatch]
enum Shape<T> {
    Rect(Rect<T>),
}

#[enum_dispatch::enum_dispatch(Shape<usize>)] // (2-3)
trait ShapeI {
    fn area(&self) -> f64;
}

impl<T> ShapeI for Rect<T>
where
    T: std::ops::Mul,
{
    fn area(&self) -> f64 {
        (self.width * self.height) as f64
    }
}
```

これで (2-3) で以下の様なコードが生成され,

```rust
impl<T> ShapeI for Shape<T> {
    #[inline]
    fn area(&self) -> f64 {
        match self {
            Shape::Rect(inner) => ShapeI::area(inner),
        }
    }
}
```

`T: std::ops::Mul` という制約が付いていないのでエラー
[[rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_generics.rs)]
[[stderr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_generics.stderr)]
になります.

これは単純に実装の問題です. 3章で議論します.

---

#### `enum_dispatch` まとめ

- enum_dispatch は **proc macro crate 中の global variables に文字列化した trait/enum の定義を保存する**
  ことで proc macro 間で情報伝達しているが, この方法は根本的な問題がある.
- 外部 crate の trait/enum 定義を扱えない.
- generics や trait bound のサポートが弱い.
- エラーがわかりにくい
  [[rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_implicit_error_if_typo.rs)]
  [[stderr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_implicit_error_if_typo.stderr)].
  軽率に panic する
  [[rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_panic_if_struct.rs)]
  [[stderr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_panic_if_struct.stderr)].

というわけで `enum_dispatch` は個人的にはおすすめしません. R.I.P. [^206] [^207]

### 閑話休題: proc macro で過激なことをやりたい {#local-state-for-proc-macro}

人間の欲求は底なしです.
「こうしたい」「ここでこれが使えればこれが作れるのに」「合法じゃないけど限定的に動作するものを作ってみた」
くらいならどんどん言ったりやったりしていくのが良いと思います.

今回のテーマである自動 delegation という文脈から少し外れますが, 「もっと一般に情報伝達したい」「状態を持ちたい」
という要望はちらほらと上がっています. 目に入ったものを挙げていきます.

- [What do you suggest proc macros to be aware of each other as opposed to a static variable?](https://users.rust-lang.org/t/what-do-you-suggest-proc-macros-to-be-aware-of-each-other-as-opposed-to-a-static-variable/77160)
- [Crate local state for procedural macros?](https://github.com/rust-lang/rust/issues/44034)
- [Is it possible to store state within Rust's procedural macros?](https://stackoverflow.com/questions/52910783/is-it-possible-to-store-state-within-rusts-procedural-macros)
- ちょっとオフトピだけど [proc macro 中で $crate を使いたい](https://github.com/rust-lang/rust/issues/54363)
  ために [Cargo.toml を読んでるやつ](https://github.com/bkchr/proc-macro-crate/blob/master/src/lib.rs).

では自動 delegation は合法的には無理なのでしょうか? 実は合法的に実現する方法があります.

### `enum_delegate` v0.2.0 {#enum-delegate-v0.2.0}

- [crates.io](https://crates.io/crates/enum_delegate)
- [repository](https://gitlab.com/dawn_app/enum_delegate)
- [doc](https://docs.rs/enum_delegate/0.2.0/enum_delegate)
- Initial commit: 2022/10

enum_delegate は enum_dispatch を改善した crate (だったもの) です.
しかし [この PR](https://gitlab.com/dawn_app/enum_delegate/-/issues/6) でデザインが根本的に変わりました.
なのでまず v0.2.0 までの話をします.

[doc -- Comparison with enum_dispatch](https://docs.rs/enum_delegate/0.2.0/enum_delegate/#comparison-with-enum_dispatch)
と
[README](https://gitlab.com/dawn_app/enum_delegate/tree/f5bcaf45#enum_dispatch)
には以下の点が差分として挙げられています.

- enum_dispatch は enum のみサポートするが, enum_delegate は **enum/struct with a field** をサポートする.
- **外部 crate の trait** をサポート
- より良いエラー
  (なんとちゃんと [failure case のテスト](https://gitlab.com/dawn_app/enum_delegate/-/tree/0.2.0/tests_error) がある!)
- Associated types サポート

使い方は enum_dispatch に近いです.

```rust
#[enum_delegate::register]          // (2-4)
trait ShapeI {
    fn area(&self) -> f64;
}

struct Rect {
    width: f64,
    height: f64,
}

#[enum_delegate::implement(ShapeI)] // (2-5)
enum Shape {
    Rect(Rect),
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {
    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6.0);
}
```

#### `enum_delegate` の情報伝達方法

情報伝達の仕組みは [How Does it Work.md](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/How%20Does%20it%20Work.md)
で解説されています. 以下で解説します.

自動 delegation のためには

- trait `Trait` の path
- trait `Trait` の定義
- struct/enum `StructEnum` の path
- struct/enum `StructEnum` の定義

が必要です.

定義があっても path は得られないことに注意してください.  
proc macro はコンパイルのかなり早いフェーズで展開されます. なので基本的に使えるのは字句情報 (`TokenStream`) のみです.
あるトークンが型なのか trait なのか変数なのかや, 事前に `use` されて別の module/crate を参照しているのかどうかなどはわかりません.
一方で `impl path::to::Trait for path::of::StuctEnum { ... }` を生成するためには trait と struct/enum の path を
どこかしらから得る必要があります. `enum_delegate::implement` の場合は trait の path は引数で与えられます. struct/enum の path は
この macro は struct/enum の定義に付けるという設計から ident `StructEnum` をそのまま使えばよいです:
`impl path::to::Trait for StuctEnum { ... }`.

残りは trait の定義のみです. enum_delegate は **proc macro 中で decl macro を経由する** ことで proc macro 間で情報を伝達します.

上記の例では以下が起こります:

1. (2-4) の trait `ShapeI` に対する `#[enum_delegate::register]` 呼び出しにおいて, 
   trait `ShapeI` の他に decralative macro `ShapeI` を定義する. (後述)
2. (2-5) の enum `Shape` に対する `#[enum_delegate::implement(ShapeI)]` 呼び出しにおいて,
   decl macro `ShapeI` を呼び出す
   [[1](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/enum_delegate_lib/src/macros/implement.rs#L65)].
   ここで trait の定義を獲得する.
3. その中で proc macro `enum_delegate::implement_trait_for_enum!{}` を呼び出す.
   これが `impl ShapeI for Shape { ... }` を生成する
   [[2](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/src/lib.rs#L141)]
   [[3](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/enum_delegate_lib/src/macros/implement_trait_for_enum.rs#L43)].

decl macro `ShapeI` は
[この様に定義](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/enum_delegate_lib/src/macros/implement.rs#L65) される:

```rust
quote! {
    #[doc(hidden)]
    #[macro_export]
    macro_rules! #macro_name {
        ($trait_path: path, $enum_path: path, $enum_declaration: item) => {
            enum_delegate::implement_trait_for_enum!{
                $trait_path,
                #parsed_trait,
                $enum_path,
                $enum_declaration
            }
        };
    }

    #[doc(hidden)]
    pub use #macro_name as #trait_name;

    #cleaned_trait
}
```

これは `(trait の path, struct/enum の path, struct/enum の定義)` を引数に取り,
そこに trait の定義 `parsed_trait` を合わせて `enum_delegate::implement_trait_for_enum!{}`
を呼び出しているだけである. (概ね)

#### 何故動くのか?

これは global variable を使う方法とは違い, 合法である. 何故だろうか?

- 副作用がなく, 渡された `TokenStream` 以外の情報を使っていない.
- 順序不定: `#[enum_delegate::implement(ShapeI)]` が先に呼び出されたとしても内側の decl macro `ShapeI`
  の展開はその定義の方が先に解決される (はず [^208]).
- 複数回呼び出し: 副作用がないので大丈夫.
- キャッシュ: decl macro の定義が変更されれば適切に invalidate されると期待できる.

#### では何故 `enum_delegate` で不満なのか?

少なくとも
generics
[[rs](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/tests_error/unsupported_generic.rs)]
[[stderr](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/tests_error/unsupported_generic.stderr)],
super trait
[[rs](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/tests_error/unsupported_supertrait.rs)]
[[stderr](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/tests_error/unsupported_supertrait.stderr)]
[^209],
trait item
[[rs](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/tests_error/unsupported_trait_item.rs)]
[[stderr](https://gitlab.com/dawn_app/enum_delegate/-/blob/0.2.0/tests_error/unsupported_trait_item.stderr)]
がサポートされていない [^210].

また, trait 定義が外部 crate にあり `#[enum_delegate::register]` を付けられないということは普通にあります.
そのときは `enum_delegate::implement` に直接 trait 定義を食わせます
[[doc](https://docs.rs/enum_delegate/0.2.0/enum_delegate/#3rd-party-traits)]
[[example](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v020/pass_implement_extarnal_trait.rs)]:

```rust
mod external {
    pub trait ShapeI {
        fn area(&self) -> f64;
    }
}

struct Rect {
    width: f64,
    height: f64,
}

#[enum_delegate::implement(
    external::ShapeI,
    trait ShapeI {
        fn area(&self) -> f64;
    }
)]
enum Shape {
    Rect(Rect),
}

impl external::ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}
```

これにはいくつか面倒なポイントがあります. 3章で再訪します.

#### `enum_delegate` v0.2.0 まとめ

- enum_delegate は **proc macro 中で decl macro を経由する** ことで proc macro 間で情報を伝達する.
  これは問題がない [^211].
- generics や trait bound のサポートが弱い.
- 外部 crate の trait の扱いが面倒.

ありがとう `enum_delegate`, 勉強になったよ. でも別方向に行っちゃったんだよなぁ. R.I.P.

### `auto-delegate` {#auto-delegate}

- [crates.io](https://crates.io/crates/auto-delegate)
- [repository](https://github.com/not-elm/auto-delegate)
- [doc](https://docs.rs/auto-delegate/0.1.3/auto_delegate)
- Initial commit: 2023/05

enum_delegate v0.3.0 の仕組みの解説の前に, より素朴な auto-delegate を取り上げます.

[使い方](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate/pass_basic.rs):

```rust
#[auto_delegate::delegate]         // (2-6)
trait ShapeI {
    fn area(&self) -> f64;
}

struct Rect {
    width: f64,
    height: f64,
}

#[derive(auto_delegate::Delegate)] // (2-7)
#[to(ShapeI)]
enum Shape {
    Rect(Rect),
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}
```

#### 情報伝達の仕組み: `impl Trait for auto_delegate::Delegates<...>`

auto-delegate は **`auto_delegate::Delegates<...>` という struct を介して自動 delegation する**.

[auto-delegate/src/lib.rs](https://github.com/not-elm/auto-delegate/blob/v0.1.3/src/lib.rs#L6-L20):

```rust
#[doc(hidden)]
pub struct Delegates<A, B, C, D, E, F, G, H, I, J, K, L>(
    pub Option<A>,
    pub Option<B>,
    pub Option<C>,
    pub Option<D>,
    pub Option<E>,
    pub Option<F>,
    pub Option<G>,
    pub Option<H>,
    pub Option<I>,
    pub Option<J>,
    pub Option<K>,
    pub Option<L>,
);
```

1. (2-6) で `#[auto_delegate::delegate]` すると `impl ShapeI for auto_delegate::Delegates<...>` が定義される.
   例えば receiver が `&self` の場合:
   [[gen 1](https://github.com/not-elm/auto-delegate/blob/v0.1.3/impl/src/delegate_trait.rs#L65-L89)]
   [[gen 2](https://github.com/not-elm/auto-delegate/blob/v0.1.3/impl/src/trait_meta/fn_meta.rs#L39)]
   [[gen 3](https://github.com/not-elm/auto-delegate/blob/v0.1.3/impl/src/trait_meta/fn_meta.rs#L79-L82)].
   (どこかひとつのフィールドのみ `Some` であることが仮定されている.)
2. (2-7) で `#[derive(auto_delegate::Delegate)]` すると, `impl ShapeI for Shape` の中で `&self` などを
   `auto_delegate::Delegates<variant, ...>` に変換し
   [[enum](https://github.com/not-elm/auto-delegate/blob/v0.1.3/impl/src/derive/enum.rs#L84-L91)]
   [[struct](https://github.com/not-elm/auto-delegate/blob/v0.1.3/impl/src/derive/struct.rs#L124-L126)],
   (2-6) の `#[auto_delegate::delegate]` で定義されていたメソッドを呼びだす
   [[gen 4](https://github.com/not-elm/auto-delegate/blob/v0.1.3/impl/src/trait_meta/fn_meta.rs#L107-L145)].
   各 enum variant が `auto_delegate::Delegates` のフィールドに対応する [^212].

前のふたつとは異なり proc macro 間で `TokenStream` を受け渡すのではなく,
**rustc に型と関数呼び出しを解決させることにより trait と struct/enum を行き来している** というのがキモである.
proc macro はその御膳立てをしているにすぎない.

この形式は [rfcs#2329 のコメント](https://github.com/rust-lang/rfcs/pull/2393#issuecomment-392248367) (2018/05)
で提案されています.

#### 評価

この方法には致命的な欠点がみっつある.

---

ひとつめ. delegatee が crate local に定義されている型でなければ trait の実装がコンフリクトすること
[[rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate/fail_conflict_impl_trait_for_field.rs)]
[[stderr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate/fail_conflict_impl_trait_for_field.stderr)].

`#[auto_delegate::delegate] pub trait Hello {...}` の時点で生成される中継となる定義の基本形は次です:

```rust
impl<DelegateImpl> Hello for DelegateImpl
where
    DelegateImpl: auto_delegate::Delegatable<...>,
    <DelegateImpl as auto_delegate::Delegatable<...>>::A: Hello,
    ...
    <DelegateImpl as auto_delegate::Delegatable<...>>::L: Hello,
```

これと自分で定義した `impl Hello for String` がコンフリクトします.
(将来的に auto_delegate が `impl DelegateImpl for String` する可能性があるため rustc はこれを reject せざるを得ない.)

---

ふたつめ. 外部 crate の trait に対して後付けできない.

struct/enum が crate 内で定義されていたとしても, この仕組みでは `impl external::Trait for auto_delegate::Delegates<...>`
を経由せざるを得ません. trait `external::Trait` に対して `external` 側では `#[auto_delegate::delegate]` が付けられていない場合
(enum_delegate で trait 定義を直書きしているケース), この双方は non local であり特殊な場合を除き
[orphan rule](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence)
に反します.

---

みっつめ. macro 展開結果がかなり読み難いということ
[[example](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate/pass_basic.rs)].

これは単に読み難いというだけの問題ではない. 例えばエラーが出たときに (proc macro やこの仕組みを知っているとは限らない)
ユーザーが解決しやすいわかりやすいメッセージが出せないといった practical な問題を孕んでいる.
(例えば上のコンフリクトしているケース
[[stderr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate/fail_conflict_impl_trait_for_field.stderr)]
でエラーメッセージからどうすれば解決できるのか, そもそも解決可能なのかがわかるだろうか?
もちろん `cargo expand` して実装を読めばわかるが, それは最終手段である.)

---

あと致命的と言うべきかはわかりませんが, 全体的に柔軟性に欠けます.
例えば associated type は第一 variant のものを使っています
[[rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/auto-delegate/fail_not_supported_associated_const.rs)].
しかし associated type はともかく (enum variant 毎に違っているものを纏めたいケースを思いつかない) associated const は delegator 側で新しく定義するユースケースが出てくることを否定できません.
少なくとも外部 crate で trait が定義されていて associated const を持つケースにエスケープハッチを開けておく必要があります.
(想定されているケースが素朴すぎる. 現場で使われるためには完璧なものかダクトテープで運用できるものが求められる.)

#### `auto-delegate` まとめ

- `impl` がコンフリクトするケースがある.
- 外部 crate の trait が扱えない.
- 生成されるコードが非自明.
- 柔軟性に欠ける.

仕組みの制約がきつすぎる. 黒魔術を使うならその原理をドキュメントに書いてほしい. R.I.P. [^213]

### `enum_delegate` v0.3.0 {#enum-delegate-v0.3.0}

- [crates.io](https://crates.io/crates/enum_delegate)
- [repository](https://gitlab.com/dawn_app/enum_delegate)
- doc: not yet released new version.
- commit: 2022/12 [^214]

この [issue](https://gitlab.com/dawn_app/enum_delegate/-/issues/6) でデザインが根本的に変更された.

- generics がサポートされている.
- super trait がサポートされていない
  [[rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030/fail_not_supported_super_trait.rs)]
  [[stderr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030/fail_not_supported_super_trait.stderr)]
  [^215].
- associated type/const がサポートされていない
  [[rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030/fail_not_supported_associated_const.rs)]
  [[stderr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030/fail_not_supported_associated_const.stderr)]
  [^216]

#### 仕組みと評価

`auto-delegate` に似ているが, struct ではなく
`enum_dispatch::Either<A, enum_dispatch::Either<B, ...enum_dispatch::Either<X, enum_dispatch::Void>>>`
に変換する
[[example](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030/pass_basic.rs)].

auto-delegate とのこの違いにより, std types は
[サポートされている](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_delegate_v030/pass_supported_std_types.rs).
`impl Hello for enum_dispatch::Either<String, enum_dispatch::Void>` になるため.
(`Hello` 側は `enum_dispatch` 側に見えないので auto-delegate の様な問題は起こらない.)
外部 crate の trait が orphan rule にひっかかる点は同じ. (`enum_dispatch::Either<A, ...>` は local ではないので.)
super trait は真面目にやればできるかも.

[最適化はされるらしい](https://gitlab.com/dawn_app/enum_delegate/-/issues/6#note_1189900264) [^217].

#### `enum_delegate` v0.3.0 まとめ

auto-delegate と同じ. 読めないし制約がきつすぎる.

### `ambassador` {#ambassador}

- [crates.io](https://crates.io/crates/ambassador)
- [repository](https://github.com/hobofan/ambassador)
- [doc](https://docs.rs/ambassador/0.4.1/ambassador) [^218]
- Initial commit: 2019/11 [^219]

ambassador は enum_dispatch に継いで古くからある crate で, 機能的には最もカバー範囲が広いです.
一方で考えられる全てをカバーしようとしており構文的には複雑なものになっています.

ここでは全てを紹介することはせず, 簡潔にだけ纏めておきます. 詳細は README やドキュメントを参照してください.

- generics サポート [[REDAME](https://github.com/hobofan/ambassador/tree/0.3.5?tab=readme-ov-file#delegateshoutx---trait-generics)]
- trait bound サポート [[README](https://github.com/hobofan/ambassador/tree/0.3.5?tab=readme-ov-file#delegate-where--a-shout---where-key)]
- 外部 crate の trait サポート
  - 外部 crate 側で ambassador を使っている場合
    [[doc](https://docs.rs/ambassador/0.4.1/ambassador/#cross-module-uses)]
  - 外部 crate 側で ambassador を使っていない場合
    [[README](https://github.com/hobofan/ambassador/tree/0.3.5?tab=readme-ov-file#for-remote-traits-delegatable_trait_remote)]
- 外部 crate の struct も扱える [[README](https://github.com/hobofan/ambassador/tree/0.3.5?tab=readme-ov-file#for-remote-types-delegate_remote)]
- trait のメソッドが receiver 以外の引数を取ってもよい
- 複数 field を持つ struct のサポート [[README](https://github.com/hobofan/ambassador/tree/0.3.5?tab=readme-ov-file#delegate-target--foo---target-key)]

#### 仕組み

基本的には enum_delegate と同じく, proc macro 中で decl macro を定義/利用しています
[[in `#[ambassador::derigatable_trait]`](https://github.com/hobofan/ambassador/blob/0.3.5/ambassador/src/register.rs#L69-L106)]
[[in `#[derive(ambassador::Delegate)`](https://github.com/hobofan/ambassador/blob/0.3.5/ambassador/src/derive.rs#L199-L208)].
derive する際に enum_delegate は proc macro -> decl macro -> proc macro と処理をしていましたが,
ambassador は proc macro -> decl macro だけでコードを生成しています.
これを可能にするため, ambassador の decl macro は先にパーツ単位に加工しておき, 呼び出し側でそれらを別個に取り出す形になっています.

#### 仕組みの評価

あんまり差がないので省略.

個人的には proc macro -> decl macro で処理している分煩雑になっていると思う. 4章でちらっと触れます.

#### `ambassador` まとめ

- 仕組み: proc macro 中で decl macro を使う.
- 機能的にカバー範囲が広い.

たぶんおすすめです [^220].

### `portrait` {#portrait}

- [crates.io](https://crates.io/crates/portrait)
- [repository](https://github.com/SOF3/portrait)
- [doc](https://docs.rs/portrait/0.3.0/portrait)
- Initial commit: 2023/01

portrait は enum をサポートしていませんが一応取り上げておきます.

```rust
#[portrait::make]
trait ShapeI {
    fn area(&self) -> f64;
}

struct RectWrapper(Rect);

#[portrait::fill(portrait::delegate(Rect; self.0))]
impl ShapeI for RectWrapper {}

struct Rect {
    width: f64,
    height: f64,
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}
```

proc macro の中に proc macro を記述するという独特なスタイルです. `portrait::delegate` の部分にはこれ以外も書くことができます
[[doc](https://docs.rs/portrait/0.3.0/portrait/#provided-fillers)].

また, **attribute を struct/enum の定義ではなく `impl Trait for Struct` に付ける** というのも他にはない特徴です.
3章で議論します.

この形式は [rfcs#2329 のコメント](https://github.com/rust-lang/rfcs/pull/2393#issuecomment-394498215) (2018/06)
で提案されています.

#### 仕組みと評価

情報伝達の仕組みは殆ど enum_delegate v0.2.0 と同じです.
proc macro -> decl macro -> proc macro で処理をしています.

#### `portrait` まとめ

- 仕組み: proc macro 中で decl macro を使う.
- delegation 以外もできる.
- enum をサポートしない.
- テストが少ない.

評価対象外 [^221].

### `delegate` {#delegate}

- [crates.io](https://crates.io/crates/delegate)
- [repository](https://github.com/kobzol/rust-delegate)
- [doc](https://docs.rs/delegate/0.13.1/delegate)
- Initial commit: 2018/05

「Rust で delegation といったらこれ」くらいの認知度を誇る crate です.
この記事でターゲットとしている自動 delegation ではないのですが, thin_delegate が大きく影響されているので取り上げます.

```rust
trait ShapeI {
    fn area(&self) -> f64;
}

enum Shape {
    Rect(Rect),
    Circle { circle: Circle, center: (f64, f64) }
}

impl Shape { // or `impl ShapeI for Shape` for `area()`.
    delegate::delegate! {
        to match self {
            Self::Rect(x) => x,
            Self::Circle { circle, .. } => circle,
        }
        {
            fn area(&self) -> f64;
        }
    }

    fn center(&self) -> (f64, f64) {
        match self {
            Self::Rect(_) => unimplemented!(),
            Self::Circle { center, .. } => center.clone(),
        }
    }
}

struct Rect {
    width: f64,
    height: f64,
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

struct Circle {
    radius: f64,
}

impl ShapeI for Circle {
    fn area(&self) -> f64 {
        3.14 * self.radius * self.radius
    }
}

fn main() {
    let rect = Rect { width: 2.0, height: 3.0 };
    assert_eq!(rect.area(), 6.0);
    let shape = Shape::Rect(rect);
    assert_eq!(shape.area(), 6.0);
    let circle = Circle { radius: 2.0 };
    assert_eq!(circle.area(), 12.56);
    let shape = Shape::Circle { circle, center: (0.0, 0.0) };
    assert_eq!(shape.area(), 12.56);
    assert_eq!(shape.center(), (0.0, 0.0));
}
```

見ての通りですが, delegate は柔軟性に特化しており, trait method の delegation に縛られません.
その代わり, メソッドの列挙は手動で行う必要があります.

また, [豊富な機能](https://github.com/kobzol/rust-delegate?tab=readme-ov-file#features) があるようです [^222].

#### 仕組みと評価

N/A

#### `delegate` まとめ

たぶんおすすめです.

### `delegate-attr` {#delegate-attr}

- [crates.io](https://crates.io/crates/delegate-attr)
- [repository](https://github.com/upsuper/delegate-attr)
- [doc](https://docs.rs/delegate-attr/0.3.0/delegate_attr)
- Initial commit: 2020/05

この記事でターゲットとしている自動 delegation ではないのですが目に入ったので一応取り上げておきます.

```rust
trait ShapeI {
    fn area(&self) -> f64;
}

struct RectWrapper(Rect);

#[delegate_attr::delegate(self.0)]
impl RectWrapper {
    fn area(&self) -> f64 {}
}

struct Rect {
    width: f64,
    height: f64,
}

impl ShapeI for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}
```

#### 仕組みと評価

N/A

#### `delegate-attr` まとめ

評価対象外 [^223].

### 2章まとめ {#summary}

この章では proc macro 間での情報伝達と delegation crate 比較を行いました.

trait method の自動 delegation に使われている情報伝達方法は現状みっつです:

- proc macro 中の global variable を使う
  - 合法ではない. 使うべきではない.
- proc macro 中で decl macro を使う
  - 柔軟に対応可能. 他のケースにも応用が効きそう.
- 間に struct/enum/trait を挟む
  - 制約が強い. テクニックとして知っておいて損はなさそう.

この観点からは, **trait method に対しては ambassador, それ以外については delegate** という使い分けになるでしょう [^224] [^225].

[3章](../2025-01-04-thin_delegate-3) では thin_delegate の設計と実装の話をします.


[^201]: 「そうでなければ」というのは正確ではないが理解しやすさを優先してこの様な説明にした.
        (まぁ挙動として間違ってはいない (はず).) 正確な理解のためにはコードを追ってください.
[^202]: 例では 1 ファイルで説明していますが, trait と enum の定義を別ファイルに置くのは普通にやります.
        このとき proc macro の評価順序は proc macro 作者としては期待できないと思っておいた方が良いです.
        (僕のクソ雑理解では rustc のコンパイルは (特に proc macro 解決フェーズは) 決定的ではあるが,
        どのファイルがどの順序で読まれるかは普通意識しないため.)
        あくまで推測ですが「defer の仕組みがないとこの問題に対応できない」ということでこういう作りになっているのでしょう.
[^203]: Rustonomicon にこのあたりのこと書いてないっぽいんですよねぇ...
       なのでここで言う「合法」というのは Rust 公式のワーディングではないし, 定義もない.
       この記事の中でのみ使い, 「なんか問題なく動きそう」くらいの意味である.
       ((ここに限らないけど) 誰か知ってたら教えてください.)
[^204]: でもなんか期待した挙動にはならないんですよね...
         [[3](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/pass_def_in_other_mod_without_arg_incorrect.rs)]
         [[4 rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_def_in_other_mod_with_arg.rs)]
         [[4 stdeerr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/fail_def_in_other_mod_with_arg.stderr)].
[^205]: rustc/cargo を読んで確定させろという話ですが...
[^206]: 記事を書くにあたって調べ直していて crates.io を見て驚きました.
        流石に ambassader の方がダウンロード数多いと思っていたので...
[^207]: enum_dispatch に限らずこの章でおすすめしないと書いている全ての crate に言えることですが,
        あなたが管理しているアプリケーションで使う分には問題ない壊れ方だと思います.
        とはいえ現代では ambassader や thin_delegate といった合法な crate があり,
        API も似た感じなので乗り換えるのをおすすめします.

[^208]: 僕は rustc の挙動に詳しくないのでぼかしておく. 
        [このへん](https://rustc-dev-guide.rust-lang.org/macro-expansion.html#expansion-and-ast-integration)
        から始めていけばわかる気がする.
[^209]: enum_dispatch の最新版 (0.3.13) だとこれは扱えます
        [[rs](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/pass_super_trait.rs)]
        [[stderr](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui/enum_dispatch/pass_super_trait.rs)].
[^210]: [trybuild](https://crates.io/crates/trybuild) を使って unsupported case を書いていてくれてマジでありがとう.
        [自分で書いて検証](https://github.com/kenoss/kenoss.github.io/tree/main/content/blog/2025-01-01-thin_delegate_aux/tests/ui) せずに済んだ.
        全人類見習うべきだよ.
[^211]: 発見されていない. まぁでも多分問題ないでしょう.

[^212]: (ちゃんとチェックしてないけど) `impl Trait for auto_delegate::Delegates<...>` の if 分岐はコンパイラの最適化で消えることが期待されていると思う.
        (ちゃんとやらないといけないので消えなくてもおかしくはないけど.)
[^213]: でも enum_delegate v0.3.0 ではサポートされていない super trait がサポートされていたり,
        (ちょっと弄った感じ) generics/trait bound まわりで反例が作れないので頑張ってはいると思う.
[^214]: auto-delegate の initial commit より前である. v0.3.0 リリースされてないけど...
        (リリースされてなくて困って [野良リリース](https://crates.io/crates/temporary_enum_delegate_0_3_0) してる人がおる.)

[^215]: 真面目にやればできるのかもしれないがこの仕組みの上であんまり頑張ってもな...
[^216]: この [PR](https://gitlab.com/dawn_app/enum_delegate/-/merge_requests/1/diffs)
        で not supported case のテストが削除されている. この仕組みは有望ではないと思っているので裏取りが面倒になってきた.
        enum_delegate v0.3.0 の実装は斜め読みでほぼほぼ `cargo expand` しか見ていない.
[^217]: しかしそれ, 現時点で差がないことはわかるかもしれないけど継続的保証になってないのでは...?

[^218]: docs.rs の方は 0.4.1 が最新だがリポジトリ側の tag は 0.3.5 が最新. ずれるけど許して.
        (最新を参照しないのはフェアじゃない感がある.)
[^219]: 説明の都合で順序が逆になって申し訳ないが, enum_delegate よりも ambassador の方が登場がだいぶ早い.
        (ambassodor, 機能も構文も複雑なんですよねぇ...)
[^220]: 実際のところは, 構文が好きじゃなくてあんまりまじめに秘孔を突く作業をしてないのでよくわからない...
        (先行研究はもっとちゃんと調べろというのははい...)
        まぁドキュメントやテストや issue を見ている限りでは信頼できると思います. (How it works は書いてほしかった.)
        機能がこれで十分で構文が嫌いでなければ使うのは止めません.

[^221]: `syn::custom_keyword!` はここで知った. ありがとう.

[^222]: こういうゴテゴテしたのは僕の好みではない. 覚えられない. (本当に必要?)
        まぁ [rfcs#3530 のコメント](https://github.com/rust-lang/rfcs/pull/3530#issuecomment-1824320933)
        で author の Kobzol が言っている様に, 色々事情はあるんでしょう.
        僕はこの問題を2ヶ月くらいしか考えてないので「好みじゃない」以上のことは言えない.

[^223]: delegate 使えばよくない? ambassador よりダウンロード数が多いのもよくわからないし...

[^224]: [rfcs#2329 のコメント](https://github.com/rust-lang/rfcs/pull/2393#issuecomment-801324543) (2021/03)
        でも ambassador と delegate が引き合いに出されている.
[^225]: 僕個人としては thin_delegate の方が好きなので, 「trait method については thin_delegate, それ以外については delegate」
        になると思います. delegate crate を使いたくなるケースは今のところ思い浮かびませんが.
