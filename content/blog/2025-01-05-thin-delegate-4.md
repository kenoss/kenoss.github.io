+++
title = "crate thin_delegate を書いた (4/4)"
slug = "2025-01-05-thin_delegate-4"

[taxonomies]
"tags" = ["rust", "tech", "thin_delegate"]
+++

- [1章: `thin_delegate` の紹介](../2025-01-02-thin_delegate-1)
- [2章: proc macro 間での情報伝達と delegation crate 比較](../2025-01-03-thin_delegate-2)
- [3章: `thin_delegate` の設計と実装](../2025-01-04-thin_delegate-3)
- [4章: まとめ](../2025-01-05-thin_delegate-4)
  - [thin_delegate の contribution と delegation RFC](./#contribution-and-rfcs)
  - [完走した感想](./#summary)

## 4章: まとめ

ここまで使い方, 核となる仕組み, 設計と実装を見ました.
この章では将来の展望とまとめを述べます.

### thin_delegate の contribution と delegation RFC {#contribution-and-rfcs}

関連する RFC:
[rfcs#1406](https://github.com/rust-lang/rfcs/pull/1406),
[rfcs#2393](https://github.com/rust-lang/rfcs/pull/2393),
[rfcs#3530](https://github.com/rust-lang/rfcs/pull/3530).

振り返ってみると, 必要なパーツそれぞれは概ね既知であったようにも感じます.
結局何が thin_delegate の contribution だったのでしょうか?

- 既存の crate を注意深く分析した.
- この目的にはこの構文がおそらく最良であろうと信じて投資した.
  - 構文のアイデア自体は
    [rfcs#2329 のコメント](https://github.com/rust-lang/rfcs/pull/2393#issuecomment-394498215)
    で提案されていたし, portrait が部分的に実装もしていた.
- missing parts を真面目に埋めた.
  - `scheme`, `external_trait_def`, `GenericParamReplacer`, ...
- 外部 crate の trait 定義読み込みを諦めてコピペ方式に全振りした.
- **trait/struct/enum から `TokenStream` を得られさえすればこのタイプの
  delegation は proc macro だけでかなりの満足度が得られることを示した**

最後のものはこれまで説明しませんでした. (みっつめと関係します.)

基本的な動作は, `#[thin_delegate::register]` で trait/struct/enum の定義の `TokenStream` を提供する decl mcaro (4-1) を定義し,
`#[thin_delegate::fill_delegate]` はそれを使って `#[thin_delegate::__internal__fill_delegate]` に処理を委譲する
というものでした.

では (4-1) の機能が rustc によって提供されたらどうなるでしょうか?
thin_delegate の場合, `#[thin_delegate::register]` を付けなくてもよくなることはもちろんですが,
外部 crate の trait 定義を `external_trait_def` の module 配下にコピペするといった手間もなくなります.
定義側に対応は必要なく, 使う側で好きな proc macro を使えばいいだけです.
加えて, thin_delegate の構文が好みではない場合 [^401], 別の構文の proc macro を作ることも簡単に可能になります.

[Kobzol のコメント](https://github.com/rust-lang/rfcs/pull/3530#issuecomment-1824320933)
もほぼ同じことを言っている様に見えます. 「全てのユースケースを言語レベルで解決するのは practical ではないかもしれない」
あたりが特にそうです.

言語レベルでの delegation サポート rfcs#3530 に対するこの方式の利点は, 構文の問題を third-party crates に競わせ,
最適なものを選び取るのを遅延させることができるという点です. [rfcs#1406](https://github.com/rust-lang/rfcs/pull/1406)
が出てから9年かかっています. 細かい構文で議論するよりも機能をすぐに使いたいというのが正直なところです.
デメリットは定義の `TokenStream` を取得するという強力な API が手に入ると人類はどう悪用するかわからん, などでしょうか.
あと, thin_delegate は trait method の自動 delegation しかサポートしていないので (斜め読みした限りでは)
rcfs#3530 よりは狭い.

必要となる機能の詳細として現時点で考えられるものを挙げてみます:

- macro 評価フェーズで解決し, trait/struct/enum の定義の `TokenStream` に展開される macro 的なもの `def_of()`.
  - その `TokenStream` が元の `TokenStream` とどう違うべきなのかはわからない.
- 但し, その中の path は全て元の文脈で解決され full path で置き換えられている.

これはこれで RFC になる気はする...?

正直なところ, 自分が使う分だけであれば thin_delegate で満足してしまっている.
rfcs#3530 で専門家が真面目に考えてくれるならお任せするのがベストかもしれん.
でも折角見つけたネタなので一枚噛んでおきたい気持ちもある.

やるべきこととしては, 関連 RFC をちゃんと読んで, 特に rfcs#3530 の方向性を把握して,
提案を受け入れられそうな形にして出すという感じになると思う.
僕は英語の読み書きがめっちゃ遅いので誰か手伝ってくれるとありがたい.
(あとこの活動は yak shaving なのでそこそこにして本来の活動に戻りたい気持ちもある.)

### 完走した感想 {#summary}

完走したと言っていいのかこれ...?

元々 thin_delegate は sabiniwm を書く上で本質的ではないコードが多くノイズになっていると感じたことから始まりました.
あんまり yak shaving を深くはしたくなかったのですが,
僕はワーキングメモリが少ないのでノイズが多いコードは効率が下がるので苦手だし,
おそらく真面目にやればできるんじゃないかという気がしたので脇道に逸れることにしました.
結果, 個人的には満足いくものができました.

他の人から見るとどうなんですかね? RFC などで欲しいと言われていた機能ではあるけど,
ボイラープレート削減するだけと言えばだけなので †強者† には必要ない気もします.
この記事も結局自明なことしか書いてないので「『自明』と書くだけでこの厚み?」と言われちゃいそう...
まぁ頭から flush しておかないといけないのであんまり気にしても仕方がないんですが.

proc macro を書くのは初めてだったので, proc macro workshop をやってから
習作として [struct_cache_field](https://github.com/kenoss/struct_cache_field) を作りました. 1ヶ月くらい.
thin_delegate を弄っていたのは2ヶ月くらい.

そうそう. proc macro 初心者なので誰かにレビューしてもらった方が良いなと思っています.
proc macro 書いたことあってレビューしてやってもいいよという方,
[issue](https://github.com/kenoss/thin_delegate/issues/1) でご連絡ください.

長文おつきあいありがとうございました. 明けましておめでとうございます.


[^401]: 例えば [rfcs#2393](https://github.com/rust-lang/rfcs/pull/2393) で議論されている各種構文.
        但し syn のサポートはない.
