+++
title = "crate thin_delegate を書いた (3/4)"
slug = "2025-01-04-thin_delegate-3"

[taxonomies]
"tags" = ["rust", "tech", "thin_delegate"]
+++

- [1章: `thin_delegate` の紹介](../2025-01-02-thin_delegate-1)
- [2章: proc macro 間での情報伝達と delegation crate 比較](../2025-01-03-thin_delegate-2)
- [3章: `thin_delegate` の設計と実装](../2025-01-04-thin_delegate-3)
  - [proc macro crate は `trybuild` を使うべきである](./#proc-macro-crate-should-use-trybuild)
  - [thin_delegate の基本方針](./#direction-of-thin_delegate)
  - [thin_delegate の構文](./#syntax-of-thin-delegate)
  - [thin_delegate の実装](./#implementation-of-thin_delegate)
  - [その他のテクニック](./#other-techniques)
  - [3章まとめ](./#summary)
- [4章: まとめ](../2025-01-05-thin_delegate-4)

## 3章: `thin_delegate` の設計と実装 {#chapter-3}

1章では基本的な使い方を見, 2章では実装のひとつのコアである proc macro 間での情報伝達方法を比較しました.
thin_delegate はそのうち **proc macro 中で decl macro を経由する** 方法を採用しています.
この章では **thin_delegate の設計および実装** の残りの部分を掘り下げます.
また **trait method の自動 delegation の構文** についても議論します.

### proc macro crate は `trybuild` を使うべきである {#proc-macro-crate-should-use-trybuild}

本題に入る前にひとつ言っておきたいことがあります.
**proc macro crate は [`trybuild`](https://github.com/dtolnay/trybuild) を使うべき** です.
もっと言うと, **正常系以外もちゃんとテストを書くべき** です.

proc macro は黒魔術を誰でも行使できるようにパッケージ化したものです [^301].
ドキュメントを整備するのは当然ですが, もうひとつ大事なのはエラーです.
ユーザーにドキュメントを隅から隅まで読むことは期待してはいけません.
概要を読むくらいは期待したいところですが,
平均的なユーザーはその後実際に crate を使ってみて, シンプルなケースから始めて自分が本当にやりたい形に近づけていきます.
その道中の節々でエラーに出くわし, エラーを解決しながら進んでいきます.
典型的には エラー表示 -> Web 検索 -> ドキュメント -> テスト の順に見て行くと思っておけばよいでしょう.
エラーメッセージは最初に目にするものであるため, 親切であればあるほど良いです.
一般に, 可能な場合は **ユーザーに解決を求めるエラーはユーザーのアクション候補を明示するべき** です.

trybuild を使うことで次のテストができます:

- コンパイルが通るケース. 実行時の挙動もテストできる.
- コンパイルが通らないケース. どの様なエラー表示になるのかをテストできる.
  - ユーザーが陥りやすいケースにどの様なエラーメッセージが表示されるか.
    ユーザーはリポジトリを grep したりテストを眺めることでここに辿りつくケースがある.
  - サポートされない機能の明示.
    ファイル名を limitation とか unsupported とかで統一しておけばユーザーが把握しやすくなります.
    ドキュメントからリンクを貼ればより良いです.
- panic しないこと

まぁしかし, proc macro 側がいくら頑張ってもエラーメッセージを改善しようがないケースも多々あります.
そういうときはドキュメントで例示だけして諦めましょう. 逆にこのケースはユーザーが頑張るべきです.
ドキュメントを読んだり `cargo expand` しましょう.

### thin_delegate の基本方針 {#direction-of-thin_delegate}

proc macro の目的にはいくつかの系統があります. 雑に分類してみましょう.

- trait のデフォルト実装を埋める
  - derive macro, e.g. `#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]`
- ユーザーが展開系を想像できるものを楽に生成する
  - `#[derive(derive_more::From)]` [[doc](https://docs.rs/derive_more/latest/derive_more/derive.From.html)]
  - `#[derive(getset::Getters, getset::Setters)]` [[doc](https://docs.rs/getset/latest/getset/)]
  - `#[derive(derive_builder::Builder)]` [[doc](https://docs.rs/derive_builder/latest/derive_builder/)]
  - `#[thin_delegate::fill_delegate]` [[doc](https://docs.rs/thin_delegate/latest/thin_delegate/attr.fill_delegate.html)]
  - `seq!` [[README](https://github.com/dtolnay/proc-macro-workshop?tab=readme-ov-file#function-like-macro-seq)]
- 構造体定義の DSL
  - `#[derive(thiserror::Erorr)]` [[doc](https://docs.rs/thiserror/latest/thiserror/)]
  - `#[bitfield]` [[README](https://github.com/dtolnay/proc-macro-workshop?tab=readme-ov-file#attribute-macro-bitfield)]
- 制約を守らせる
  - `#[sorted]` [[README](https://github.com/dtolnay/proc-macro-workshop?tab=readme-ov-file#attribute-macro-sorted)]
  - `#[remain]` [[doc](https://docs.rs/remain/latest/remain/)]
- 下層の仕組みの隠蔽
  - `invently::submit!` [[repo](https://github.com/dtolnay/inventory)]
  - `#[distributed_slice]` [[repo](https://github.com/dtolnay/linkme)]

thin_delegate は **ユーザーが展開系を想像できるものを楽に生成する** に属します.
このケースでは構文も挙動も可能な限りシンプルであるべきと僕は思っています. 何故なら複雑にすると想像が効かなくなる/外れるからです.
以下ではこれを念頭に置いておきます.

### thin_delegate の構文 {#syntax-of-thin-delegate}

さて, 本題に入りましょう. まずは構文を決めておきましょう.

thin_delegate は enum_delegate, ambassador, delegate (, portrait) から大きな影響を受けています.
ここでは thin_delegate が何故この構文を採用しているのかを, 他と比較しながら説明していきます.

#### `impl Trait for StructEnum {...}` に付ける {#attr-macro-on-impl-trait-for}

基本的な構文は以下の様なものでした:

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

(1-2), (1-3), (1-4) で何が起こるかを想像できますか?

- `#[thin_delegate::register]` は trait/struct/enum の定義を後で使えるように登録する.
- `#[thin_delegate::fill_delegate]` は `impl Trait for StructEnum {...}` のうち未定義な trait method を埋める.

基本的にはこれだけです.

似た様なことを ambassador でやると以下の様になります.

```rust
#[ambassador::delegatable_trait]
trait Shout<T>
// これを付けると駄目
// where
//     T: std::fmt::Display,
{
    fn shout(&self, input: T) -> String;
}

impl<T> Shout<T> for String
where
    T: std::fmt::Display,
{
    fn shout(&self, input: T) -> String {
        format!("{}, {}", self, input)
    }
}

#[derive(ambassador::Delegate)]
#[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
pub struct Cat(String);

#[derive(ambassador::Delegate)]
#[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
enum Animal {
    Cat(Cat),
}

fn main() {
    let cat = Cat("meow".to_string());
    assert_eq!(cat.shout("world"), "meow, world");
    let animal = Animal::Cat(cat);
    assert_eq!(animal.shout("world"), "meow, world");
}
```

何故 `#[derive(ambassador::Delegate)]` と `[delegate(...)]` が分かれているのかや,
generics/trait bound の指定の仕方など, ぱっとわかりにくいと思います.

すぐわかる差分は次のみっつです:

1. `#[thin_delegate::fill_delegate]` は `impl Trait for StructEnum {...}` に付ける.
   ambassador の `#[delegate(...)]` は struct/enum に付ける.
2. ambassador では `#[derive(ambassador::Delegate)]` と `[delegate(...)]` の両方を記述する必要がある.
3. thin_delegate では generics の指定のための追加のオプションはない.
   ambassador は `#[delegate(...)]` の中で指定する必要がある.

これらは密接に絡み合っています.

まず 2. ですが, (1. ambassador のデザインの下では) struct/enum に対して複数の trait を自動 delegate
させるためには以下の様に書く必要があります:

```rust
#[derive(ambassador::Delegate)]
#[delegate(Shout<T>, generics = "T", where = "T: std::fmt::Display")]
#[delegate(Walk)]
enum Animal {
    Cat(Cat),
}
```

このため `#[derive(ambassador::Delegate)]` と `[delegate(...)]` を分ける必要があります [^302].

そうなのです. この問題を「`impl Trait for StructEnum {...}` を埋める」問題だと認識しさえすれば,
この macro は trait `Trait` と struct/enum `StructEnum` の組 `(Trait, StructEnum)`
に対して記述/動作するのが自然 (1.) だということがわかります. そうすると後のデザインはほぼ一直線です.

[derive macro は struct/enum/union にしか付けられません](https://doc.rust-lang.org/reference/procedural-macros.html#r-macro.proc.derive.intro).
なので thin_delegate では普通の proc macro で実装しています [^303].

3.の選択も (少なくとも構文的には) そこから従います. `impl Trait for StructEnum {...}` に付けるのであれば
この Rust の構文で記述される generics/trait bound の情報を使うのが最も自然でしょう.

この選択により `ambassador::Delegate` のオプション
[`where`](https://docs.rs/ambassador/latest/ambassador/derive.Delegate.html#delegate-where--a-debug---where-key)
[`generics`](https://docs.rs/ambassador/latest/ambassador/derive.Delegate.html#delegateshoutx-generics--x---trait-generics)
[`automatic_where_clause`](https://docs.rs/ambassador/latest/ambassador/derive.Delegate.html#delegateshout-automatic_where_clause--false---inhibit-automatic-generation-of-where-clause)
は不要になります [^304]. 残りは
[`target`](https://docs.rs/ambassador/latest/ambassador/derive.Delegate.html#delegate-target--foo---target-key)
だけです.

#### Non trivial delegation {#non-trivial-delegation}

ambassador の `target` は次の様に delegate すべきフィールドが一意に定まらない場合に使います:

```rust
#[derive(ambassador::Delegate)]
#[delegate(Walk, target = "foo")] // <-------- Delegate implementation of Shout to struct field .foo
pub struct MultiFieldStruct {
  foo: Cat,
  bar: Cat,
}

#[derive(ambassador::Delegate)]
#[delegate(Walk, target = "1")] // <-------- Delegate implementation of Shout to second field
pub struct MultiFieldTuppleStruct(Cat, Cat);
```

これをどう扱うか話の前に, もう少し一般化しましょう.

この記事では組 `(Trait, StructEnum, trait_item)` で **`trait_item` が trivial に delegation 可能** を以下で定義します:

- `trait_item` が trait function (method) である; and
- `StructEnum` が enum またはひとつの field のみを持つ struct である.

用語を濫用して単に **trivial である** とも言うことにする.

さて, thin_delegate は「trivial なものは delegation で自動に, そうでないものは段階的に自由度を上げられるように」しています.
そのためのふたつの機能が **`scheme`** と **手動実装への fallback** です.

#### Argument `scheme` {#arg-scheme}

thin_delegate は trivial な場合は delegation によって method を埋めますが, `scheme` はこのメソッド生成ルールを弄ります.

```rust
#[thin_delegate::register]
pub struct MultiFieldStruct {
  foo: Cat,
  bar: Cat,
}

#[thin_delegate::fill_delegate(scheme = |f| f(&self.foo))]
impl Walk for MultiFieldStruct {}

#[thin_delegate::register]
pub struct MultiFieldTuppleStruct(Cat, Cat);

#[thin_delegate::fill_delegate(scheme = |f| f(&self.1))]
impl Walk for MultiFieldStruct {}
```

この構文を追加することで複雑な場合もハンドリングできるのが良いですね.

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
            #[cfg(feature = "winit")]
            Self::Winit(backend) => f(backend),
        }
    }
)]
impl smithay::wayland::buffer::BufferHandler for Backend {}
```

これは `delegate` crate の構文をどうやって cryptic でない形で組み込むかを考えた結果です.

#### 手動実装への fallback {#fallback-to-manual-associated-items}

自動 delegation でも `scheme` でも associated const/type を扱うことはできません [^305].
**1. 手動の実装が提供されている場合, それについては自動生成をしない, 2. trivial でない場合は自動生成をしない**
とすることでこのケースをユーザーに委ねることができます:

```rust
#[thin_delegate::register]
trait Hello {
    type Return;

    const HAS_DEFAULT: &'static str = "HAS_DEFAULT";
    const NEED_TO_FILL: &'static str;

    // `thin_delegate` only can fill associated functions.
    fn filled(&self) -> Self::Return;
    fn override_(&self) -> Self::Return;
}

impl Hello for String {
    type Return = String;

    const NEED_TO_FILL: &'static str = "String";

    fn filled(&self) -> Self::Return {
        self.clone()
    }

    fn override_(&self) -> Self::Return {
        self.clone()
    }
}

#[thin_delegate::register]
struct Hoge(String);

#[thin_delegate::fill_delegate]
impl Hello for Hoge {
    // It can handle associated types in impl.
    //
    // You need to specify them by yourself as if you don't use `thin_delegate`.
    type Return = String;

    // It can handle associated consts in impl.
    //
    // You need to specify them by yourself as if you don't use `thin_delegate`.
    const NEED_TO_FILL: &'static str = "Hoge";

    // It can handle associated functions in impl.
    //
    // If an impl doesn't has an associated function (`filled()`), it is filled.
    // If an impl has an associated function (`override_()`), it is used.
    fn override_(&self) -> Self::Return {
        self.0.override_().to_uppercase()
    }
}
```

「associated const/type が与えられている場合にそれを使う」という挙動は portrait のものを継承しています [^306].
thin_delegate はより強く, 「与えなければならない」です. 
(associated const
[[rs](https://github.com/kenoss/thin_delegate/blob/v0.1.0/tests/ui/fail_intended_limitation_associated_const_misning.rs)]
[[stderr](https://github.com/kenoss/thin_delegate/blob/v0.1.0/tests/ui/fail_intended_limitation_associated_const_misning.stderr)],
associated type
[[rs](https://github.com/kenoss/thin_delegate/blob/v0.1.0/tests/ui/fail_intended_limitation_associated_type_missing.rs)]
[[stderr](https://github.com/kenoss/thin_delegate/blob/v0.1.0/tests/ui/fail_intended_limitation_associated_type_missing.stderr)]
)

#### Argument `external_trait_def` {#arg-external_trait_def}

さて, これで生成部分は良さそうです. もうひとつ, 「どうやって外部 crate の trait 定義を取り込むか」という問題があります.

1. 外部 crate に `#[thin_delegate::register]` 相当が付いている場合
2. 付いていない場合

の対応を考える必要があります.

ambassador では **`Trait` と同名の decl macro を定義する** ことで 1. に対応し,
[`ambassador::delegatable_trait_remote`](https://docs.rs/ambassador/latest/ambassador/attr.delegatable_trait_remote.html)
という attribute macro を提供することで 2. に対応しています.

```rust
#[ambassador::delegatable_trait_remote]
trait Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>;
}

#[derive(ambassador::Delegate)]
#[delegate(Display)] // <-------- Delegate implementation of Display to struct field
pub struct WrappedCat(Cat);
```

しかし僕はこの設計に以下の理由で同意できません:

- (2.) 上では `trait Display` は実際には定義されない. `#[ambassador::delegatable_trait_remote]`
  は「`#[derive(ambassador:Delegate)]` に定義を伝えるだけ」というのがぱっと見でわからない.
- (1., 2.) [`ambassador::delegate_remote`](https://docs.rs/ambassador/latest/ambassador/attr.delegate_remote.html)
  もあり, どう使い分ければいいのかぱっと見でわからない [^307].
- (1.) `pub` な macro をユーザーに暗黙で定義してしまう. 暗黙のうちに crate 側の契約が増える.
  - 上手く動かないときにユーザーはデバッグ不能 [[issue#45](https://github.com/hobofan/ambassador/issues/45)]
  - ambassador の version が異なると使えない.
  - (1.) [backward compatibility](https://docs.rs/ambassador/latest/ambassador/#backwards-compatibility)
    と破壊的変更に気を配る必要がある. 実際上記の issue に対応するために破壊的な変更が起こっており
    [[pr#60](https://github.com/hobofan/ambassador/pull/60/files)], 利用している crate はこの破壊的変更を
    その crate 自身の破壊的変更に持ち上げる必要がある.
- (1.) upstream crate に `register` 用の attribute がついていないとき, 2. を使わずに楽をするためには upstream crate に
  PR をマージしてくれとお願いする必要がある. 説得コストが高すぎる. 上述の問題もありメンテナンスコストを増やしてしまうので
  upstream crate 側は reject するのもやむなし.
- (1.) 2. を使わずにコピペなしで全てが動く世界に行くためには delegation 界の覇権を取る必要がある...どころか,
  全ての `pub` trait に **事前に** 付けておく必要がある. 非現実的である.

以上の理由から, thin_delegate は 1. を禁止し, 2. に全振りして **コピペしやすくする** ことで解決しています
[[example](https://github.com/kenoss/thin_delegate/blob/v0.1.0/tests/ui/pass_external_trait_def_with_uses.rs)]
.

```
mod external {
    pub struct Arg;

    pub trait Hello {
        fn hello(&self, arg: Arg) -> String;
    }

    impl Hello for String {
        fn hello(&self, _arg: Arg) -> String {
            format!("hello, {self}")
        }
    }
}

// Each `#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def)]` will be
// expanded with uses as:
//
// ```
// mod ... {
//    use super::*;
//
//    use crate::external::Arg;
//
//    impl ...
// }
// ```
//
// It's convenient to copy&paste original definition as is.
#[thin_delegate::external_trait_def(with_uses = true)]
mod __external_trait_def {
    use crate::external::Arg;

    #[thin_delegate::register]
    pub trait Hello {
        fn hello(&self, arg: Arg) -> String;
    }
}

#[thin_delegate::register]
struct Hoge(String);

#[thin_delegate::fill_delegate(external_trait_def = __external_trait_def)]
impl external::Hello for Hoge {}
```

`#[thin_delegate::external_trait_def]` は `mod` に付け, その module を外部 crate の trait 定義を貼り付けるためのものとして
扱います. `#[thin_delegate::fill_delegate]` 側では `external_trait_def` にその module の path を指定します.
`#[thin_delegate::external_trait_def]` を `with_uses = true` 付きで使うとその中の `use` を
`#[thin_delegate::fill_delegate]` の展開結果の中でも使われるようにします [^308].

このデザインの利点は以下です:

- (この module 配下には普通の手段でアクセスしようとしないはずなので,) ユーザーはこの trait 定義で実際に
  trait が定義されるのかどうかに気を使わなくてよい. (実際は定義されない.)
- `register`, `fill_delegate`, `external_trait_def` が直交している [^309] のでわかりやすく使いやすい.
- crate を跨ぐケースを考えなくてよい. 特に version 間差異.
- 覇権を取る必要も説得する必要もない.

以上で主要な構文は全てです. では実装の話に移りましょう.

### thin_delegate の実装 {#implementation-of-thin_delegate}

基本的に物事というのは答えがわかりさえすればそこに至るのは比較的に楽です [^310]. あとはやるだけってやつです.

なので, 頑張る.

...で済ますのもアレなのでかいつまんで説明しておきます.

#### proc macro -> decl macro -> proc macro {#chains-of-macros}

2章で enum_delegate v0.2.0, ambassador, portrait は proc macro 中で decl macro を使って情報伝達していることは見ました.
thin_delegate は (enum_delegate v0.2.0, portrait と同じく) proc macro -> decl macro -> proc macro
と処理を繋いでいくことで実現しています. decl macro フェーズで何をやっているかは
[src/decl_macro.rs](https://github.com/kenoss/thin_delegate/blob/v0.0.3/src/decl_macro.rs)
を見れば全てわかります.

```rust
    quote! {
        macro_rules! #feed_trait_def_of {
            {
                @KONT { $kont:path },
                $(@$arg_key:ident { $arg_value:tt },)*
            } => {
                $kont! {
                    $(@$arg_key { $arg_value },)*
                    @TRAIT_DEF { #trait_ },
                }
            }
        }
        #[allow(unused_imports)]
        pub(crate) use #feed_trait_def_of;
    }

...

    quote! {
        macro_rules! #feed_structenum_def_of {
            {
                @KONT { $kont:path },
                $(@$arg_key:ident { $arg_value:tt },)*
            } => {
                $kont! {
                    $(@$arg_key { $arg_value },)*
                    @STRUCTENUM_DEF { #structenum },
                }
            }
        }
        #[allow(unused_imports)]
        pub(crate) use #feed_structenum_def_of;
    }

...

    // Collect trait and structenum defs by CPS:
    //
    //    #feed_trait_def_of!
    // -> __thin_delegate__trampoline1!
    // -> #feed_structenum_def_of!
    // -> __thin_delegate__trampoline2!
    // -> #[::thin_delegate::__internal__fill_delegate]
    quote! {
        macro_rules! __thin_delegate__trampoline2 {
            {
                @IMPL {{ $impl:item }},
                @TRAIT_DEF { $trait_def:item },
                @STRUCTENUM_DEF { $structenum_def:item },
            } => {
                #[::thin_delegate::__internal__fill_delegate(#args)]
                mod __thin_delegate__change_this_name {
                    $trait_def

                    $structenum_def

                    $impl
                }
            }
        }

        macro_rules! __thin_delegate__trampoline1 {
            {
                @IMPL {{ $impl:item }},
                @TRAIT_DEF { $trait_def:item },
            } => {
                #feed_structenum_def_of! {
                    @KONT { __thin_delegate__trampoline2 },
                    @IMPL {{ $impl }},
                    @TRAIT_DEF { $trait_def },
                }
            }
        }

        #feed_trait_def_of! {
            @KONT { __thin_delegate__trampoline1 },
            @IMPL {{ #impl_ }},
        }
    }
```

trait と struct/enum に対するふたつの `#[thin_delegate::register]` 呼び出しで
`#feed_trait_def_of!` と `#feed_structenum_def_of!` を定義します.
(実際にはこれらの macro 名は trait/struct/enum 名依存で決まる.)
`#[thin_delegate::fill_delegate]` では最初に `#feed_trait_def_of!` を呼び出し,
CPS で情報を集めた上で `#[::thin_delegate::__internal__fill_delegate(#args)]`
を呼び出します.

これ, このどんどん macro 展開を繋げていって処理をする形式, どこかで見たことある人も多いかもしれません.
僕は「TeX だな〜」と思いました [^311].

この形式によって unit test も簡単になっています. [test](#test) で見ます.

#### generics/trait bound {#generics-tairt-bound}

thin_delegate は generics/trait bound をサポートしています [^312].

```rust
#[thin_delegate::register]
pub trait Hello<T, const N: usize> {
    fn hello(&self) -> [T; N];
}

impl Hello<u8, 4> for char {
    fn hello(&self) -> [u8; 4] {
        let mut buf = [0; 4];
        self.encode_utf8(&mut buf);
        buf
    }
}

#[thin_delegate::register]
struct Hoge(char);

#[thin_delegate::fill_delegate]
impl Hello<u8, 4> for Hoge {}
```

ここで問題となるのは trait 定義の signature `fn hello(&self) -> [T; N];` と `impl Hello<u8, 4> for Hoge {}` から
`fn hello(&self) -> [u8; 4]` を作る必要があるということです.

この処理は [src/generics_param_replacer.rs](https://github.com/kenoss/thin_delegate/blob/v0.0.3/src/generic_param_replacer.rs)
にあります. visitor を使って置き換えていくだけです [^313].
(上の例だと `{T => u8}`, `{N => 4}` という `HashMap` を使って発見次第代入していく.)

これもまた `impl` に付けるという構文を採用した利点のひとつです [^304].

ちなみに super trait は特に意識する必要ありません.
[`impl` を複数書いてもらうだけ](https://github.com/kenoss/sabiniwm/blob/ed5d2c9cc199dd3358b33e8ef700e31fc39ea6ed/crates/sabiniwm/src/backend/mod.rs#L34-L55)
です.

#### macro hygiene {#macro-hygiene}

おそらく最終段に attribute macro を使った影響だと思うのですが, macro hygiene が問題になりました [^314].

```rust
    quote! {
        macro_rules! __thin_delegate__trampoline2 {
            {
                @IMPL {{ $impl:item }},
                @TRAIT_DEF { $trait_def:item },
                @STRUCTENUM_DEF { $structenum_def:item },
            } => {
                #[::thin_delegate::__internal__fill_delegate(#args)]
                mod __thin_delegate__change_this_name {
                    $trait_def

                    $structenum_def

                    $impl
                }
            }
        }
...
```

`$trait_def` (や `$impl`) のところには trait method が含まれますが, そこには `self` というトークンが含まれ得ます.
(e.g. `fn hello(&self) -> [T; N];`) このトークン列は `#feed_trait_def_of!` から来ているので,
decl macro の (partial) hygiene によってこの `self` トークンをそのまま利用することはできません
[[1](https://veykril.github.io/tlborm/syntax-extensions/hygiene.html)]
[[2](https://veykril.github.io/tlborm/decl-macros/minutiae/hygiene.html)].
トークンがどこに由来するかという情報は `Span` に入っているので
[[3](https://veykril.github.io/tlborm/proc-macros/hygiene.html)],
全ての sigunature の `self` を置き換えて proc macro 由来とする
[[src/self_replacer.rs](https://github.com/kenoss/thin_delegate/blob/v0.0.3/src/self_replacer.rs)]
ことでこの問題を解決しています [^315].

#### Attribute macro `external_trait_def` {#attr-macro-external_trait_def}

`#[thin_delegate::external_trait_def]` と `#[thin_delegate::register]` は直交していました.
これを実現するために, 内部的には `#[thin_delegate::external_trait_def]` 呼び出しでその配下にあるものに
`#[::thin_delegate::__internal__is_external_marker]` attribute を付けて回り,
`#[thin_delegate::register]` の呼び出し時にその有無で挙動を変更しています.

実装における大きめの見所はこれで終わりです.

### その他のテクニック {#other-techniques}

#### test {#test}

個人的に **テストはその挙動がチェックできる最内に書くべき** と思っています. (このへんは別記事で書けるとよいですね.)

具体例として, thin_delegate では unit test と trybuild によるテストを行っています.

- unit test: macro の展開形のチェックをする.
- trybuild: 実際にコンパイルが通るか, コンパイルエラー,
  `#[thin_delegate::__internal__fill_delegate]` に至るまでの部分などをチェックする.

```rust
    quote! {
        macro_rules! __thin_delegate__trampoline2 {
            {
                @IMPL {{ $impl:item }},
                @TRAIT_DEF { $trait_def:item },
                @STRUCTENUM_DEF { $structenum_def:item },
            } => {
                #[::thin_delegate::__internal__fill_delegate(#args)]
                mod __thin_delegate__change_this_name {
                    $trait_def

                    $structenum_def

                    $impl
                }
            }
        }
...
```

最終段は `#[thin_delegate::__internal__fill_delegate]` でハンドリングしているのでした.
ということはこの attribute macro の入出力を unit test すればどう展開されるのかのチェックが可能です.

一方, 実際にコンパイルが通るかなどはこの unit test ではチェックできません. (e.g. macro hygiene of `self`)
なのでここは trybuild でテストしています. 逆に trybuild では展開形のチェックはできません.

エラーは執拗にテストをすることでのみ改善できます. テストを書くんだ.
[パラノイアになれ](https://qiita.com/legokichi/items/4e85ec1e74f4e754fb94#ci-%E3%81%A7-cargo-clippy---tests---examples-----dclippyall-%E3%81%97%E3%82%8D)
[^316]

#### trivial coding {#trivial-coding}

個人的に **プログラムの 97% は trivial であるべき** と思っています. (これもは別記事ry)

人間の脳が最も貴重なリソースだ. それを無駄にするのは可能な限り避けなければいけない.
何より僕が読めない.

というわけで全編において僕が読める様に書いています [^317].
一例でしかないけど, 
[`portrait::delegate` の引数の parse](https://github.com/SOF3/portrait/blob/master/codegen/src/derive.rs#L62-L113)
と
`thin_delegate::fill_delegate` の引数の parse
[[1](https://github.com/kenoss/thin_delegate/blob/v0.0.3/src/fill_delegate_args.rs#L149-L186)]
[[2](https://github.com/kenoss/thin_delegate/blob/v0.0.3/src/fill_delegate_args.rs#L125-L147)]
を比較してみるとよいかもしれない [^318].

この記事を書いているのも自明にするための活動のひとつです.

### 3章まとめ {#summary}

この章では thin_delegate の設計と実装を説明しました.
[4章](../2025-01-05-thin_delegate-4) ではまとめを行ないます.

[^301]: 包丁には「よく切れます」「危険なので取り扱い注意」「人に向けてはいけません」などが書かれているでしょう.
[^302]: 何故か? これは推測でしかありませんが, 考えられるのは 1. derive macro `#[derive(ambassador::Delegate)]`
        を struct/enum に付けることに固執した. 2. 2章で解説した様に `#[derive(ambassador::Delegate)]`
        は内部で decl macro のみでコードを生成しており, thin_delegate の様に proc macro を使ったり decl macro
        を多段にするのを嫌った, あたりでしょうか. (知らんけど)
[^303]: 実はそれ以上に, trait `Trait` が実在しないのに `#[derive(Trait)]` と書けるようにするのは慎重になるべきと
        僕は思っています. `derive_more::From`, `derive_builder::Builder`, `thiserror::Erorr`
        くらいまでなら許容かな. `ambassador::Delegate` や `auto_delegate::Delegate` はやりすぎだと思います.
        `thiserror::Erorr` が許せてこれらが許せないのはなんでだろう...?
[^304]: 「一箇所直せば全てが直る」
[^305]: 無理やりやろうとすると ambiguity が出てきてしまいます
        [[example](https://github.com/kenoss/kenoss.github.io/content/blog/2025-01-01-thin_delegate_aux/tests/ui/portrait/pass_limitation_default_impl_ambiguity.rs)].
[^306]: 実は設計段階で portrait を参考にしたわけではなく `impl` に付けるのと `delegate` crate 並の柔軟性を持たせようとしたら
        再発見したというやつなのですが, 説明が楽なのでそういうことにしてください.
[^307]: 流石にこれはちゃんとドキュメントを見ればいいのでは? はい...
[^308]: いちゃもんつけポイント. この挙動は「ユーザーが展開系を想像できるくらいシンプル」でしょうか?
        いやでもこれないと面倒なんだよぉ〜〜〜
        [[sabiniwm 41c0222](https://github.com/kenoss/sabiniwm/commit/41c0222)]
[^309]: (許容できる追加コストで) 直交にできるときは直交にした方が良い. 古事記にもそう書いてある.
[^310]: 数学とか NP とか.
[^311]: TeX にはレジスタもありますが, (先頭) 完全展開可能であることが重要なケースがあります.
        そういうケースではこの様に CPS で処理することがよくあります.
        [[1](https://zrbabbler.hatenablog.com/entry/20130526/1369591778)]
        [[2](https://zrbabbler.hatenablog.com/entry/20120128/1327735636)]
        [[3](https://zrbabbler.hatenablog.com/entry/20150606/1433549572)]
        [[4](https://blog.wtsnjp.com/2018/04/28/expl3-for-tex-users/)]
        (TeX 詳しくないからしらんけど)
[^312]: 全部サポートできていると思うのですが, 穴を見つけた方は issue で教えてください.
[^313]: 最終段に proc macro を使っていない ambassador や `impl` に付けない enum_delegate はともかく,
        処理方法も構文も似ている portrait が何故これをやっていないのかは謎です.
[^314]: わりとデバッグが面倒だった覚えがあります. unit test でも `cargo expand` でも問題ない様に見えるので.
[^315]: こうやって hygiene を回避するのは, まぁこのケースなら問題ないでしょう.
[^316]: まぁしかし全ての分岐に対してテストを書いたりするのは現実的ではありません.
        代表的なものだけテストするなどでバランスを保っています.
        穴があっても OSS なので文句がある人が issue/PR で教えてくれるでしょう.
[^317]: 「自分が読めるか」と「trivial か」というのには天と地ほどの差があるわけですが,
        しかし他に自分で完結する判定方法がない...
[^318]: この `eq_token` などまで parse するのは `syn` を真似ている.
        この無駄を消すのとプログラムを読みやすく保つのとどちらが大切か?
        この proc macro の展開なんてコンパイル中で数回しか起きないのだ.
