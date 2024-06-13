+++
title = "jq はじめました"
slug = "2024-06-14-jq-hands-on"

[taxonomies]
"tags" = ["tech", "jq"]
+++

## TL;DR

jq を真面目に使ったことがなかったけどはじめてみたので備忘録.

初心者が書いているのでもっと良い書き方があったら教えてほしい.

## 問題

リポジトリにテストを管理しているディレクトリがある. テストランナーはそのディレクトリ配下のテストを実行して `test_result.json` に結果のサマリを書く.
詳細なエラーなどは別のファイルに書く.

この記事では以下の例を用いる. ちょっと長いので全体は [こちら](test_result.json):

```
$ cat test_result.json | jq -C .
{
  "version": 3,
  ...
  "tests": {
    "virtual": {
      "hoge": {
        "http": {
          "tests": {
            "protocol": {
              "domain0": {
                "test0.js": {
                  "expected": "PASS",
                  "actual": "SKIP",
                  "other": null
                },
                "test1.js": {
                  "expected": "CRASH",
                  "actual": "CRASH",
                  "other": null,
                  "crash_site": "hoge.cc(42)",
                  "artifacts": {
                    "command": [
                      "test-output-dir/virtual/hoge/tests/protocol/domain0/test1-command.txt"
                    ],
                    "stderr": [
                      "test-output-dir/virtual/hoge/tests/protocol/domain0/test1-stderr.txt"
                    ],
                    "carsh_log": [
                      "test-output-dir/virtual/hoge/tests/protocol/domain0/test1-crash-log.txt"
                    ]
                  }
                }
              },
              "domain1": {
                "test0.js": {
                  "expected": "PASS",
                  "actual": "FAIL",
...
```

この例では

- `virtual/hoge/http/tests/protocol/domain0/test0.js`
- `virtual/hoge/http/tests/protocol/domain0/test1.js`
- `virtual/hoge/http/tests/protocol/domain1/test0.js`
- `virtual/hoge/http/tests/other/test0.js`

という4つのテストファイルがあり, みっつめのものが PASS すべきなのに FAIL している. 他は SKIP 指定をされているか (`"expected": "SKIP"`) , 期待した結果になっている (`"expected"` と `"actual"` が一致している).
(例なので少ないが, 実際にはもっと多い.)

さて, ここから人間が読みやすいサマリを生成したい. この記事では以下の出力を目指す:

```
$ cat test_result.json | <somecommand>
{
  "virtual/hoge/http/tests/protocol/domain1/test0.js": {
    "expected": "PASS",
    "actual": "FAIL"
  }
}
```

こういうのは jq でできるはずである. (たぶん)

## 心構え

検索するときにおすすめなのは「絶対にできるはず. 情報はどこかに書かれているはずであり見付けられていないのは自分が見付けられていないからである」という自己催眠をかけることである.
これで検索力が10倍になる. 本当になかったときだけ自分で書く. 自分で書けば見付かる.

皆 jq 使っとる. 使っとらんのお前だけ.

[公式ドキュメント](https://jqlang.github.io/jq/manual/) を常に開いておいて Ctrl+f する.

やりたいことから方法を引くために Stack Overflow を検索する.
ちゃんとフレーム付けられた質問に対し解決方法と原理原則からの解説が書かれている答だけがベストアンサーだ.

## 基本のき

pretty print できる. (解説はしない.)

```
$ cat test_result.json | jq -C .
```

## jq '.test'

興味があるのは `.test` という部分のみである.

特定パスを抜く:

```
$ cat test_result.json | jq '.tests' | head -n 10
{
  "virtual": {
    "hoge": {
      "http": {
        "tests": {
          "protocol": {
            "domain0": {
              "test0.js": {
                "expected": "PASS",
                "actual": "SKIP",
```

[https://jqlang.github.io/jq/manual/#basic-filters](https://jqlang.github.io/jq/manual/#basic-filters)

jq は command line や grep/sed/awk/perl や Lisp みたいなもので, フィルタを繋げるのが基本である.

## 入れ子になっている object を flatten する

入れ子の object は jq で処理しにくそうである. object を deep flatten したい. とりあえず 1 level flatten したい.

```
$ cat test_result.json | jq '.tests' | jq <something>
{
  "virtual/hoge": {
    "http": {
      "tests": {
```

まだ早い. レベルを上げて出直して参れ.

## `object.items().map(f).collect()`

他の言語でも object (`HashMap` など) を iterate するには `object.items()` などで iterator を使って順に処理する.

jq も arary は map できるが object は array に変換してから map する.

[https://jqlang.github.io/jq/manual/#to_entries-from_entries-with_entries](https://jqlang.github.io/jq/manual/#to_entries-from_entries-with_entries)

`with_entries(f)` is `to_entries | map(f) | from_entries` である.

## 入れ子になっている object を flatten する 2

1. [https://stackoverflow.com/a/24710337](https://stackoverflow.com/a/24710337)
2. [https://stackoverflow.com/a/74789205](https://stackoverflow.com/a/74789205)
3. [https://stackoverflow.com/a/74791148](https://stackoverflow.com/a/74791148)

これらが近そうである. ふたつめは深さが既知で固定という制約がある.
みっつめは内側の object の key がひとつでなければならないという制約がある.
よってひとつめを真似る.

```
$ cat test_result.json | jq '.tests' | jq '. | to_entries | map(.key as $parent_key | .value | to_entries | map(.key |= $parent_key + "/" + .))' | jq -C . | head -n 10
[
  [
    {
      "key": "virtual/hoge",
      "value": {
        "http": {
          "tests": {
            "protocol": {
              "domain0": {
                "test0.js": {
```

- ひとつめの `. to_entries | map(` で外側の `object.items().map(` し,
- `.key as $parent_key` で外側の key を束縛し, [[doc](https://jqlang.github.io/jq/manual/#variable-symbolic-binding-operator)]
- ふたつめの `.value | to_entries | map(` で内側の `object.items().map(` し
- `.key |= $parent_key + "/" + .` で key だけ弄った iterator を得る. [[doc](https://jqlang.github.io/jq/manual/#addition)]

二重に map しているので出力は2次元配列になっている. なのであとは `flatten` して `from_entries` で元に戻す.

```
$ cat test_result.json | jq '.tests' | jq '. | to_entries | map(.key as $parent_key | .value | to_entries | map(.key |= $parent_key + "/" + .)) | flatten | from_entries' | jq -C . | head -n 10
{
  "virtual/hoge": {
    "http": {
      "tests": {
        "protocol": {
          "domain0": {
            "test0.js": {
              "expected": "PASS",
              "actual": "SKIP",
              "other": null
```

よさそう.

## くりかえし

あとはこれを必要なだけ繰り返せばよい. いくつか方法が考えられる.

<ol type="A">
  <li>結果が変わらなくなるまで繰り返すのをシェル芸でやる.</li>
  <li>処理すべき <code>.value</code> がなくなるまで繰り返す. (例えば全ての <code>.value</code> が <code>"expected"</code> を持つ.)</li>
  <li>先に JSON の max depth を取っておいてその回数繰り返す. (1回の処理につきネストがひとつ減るので.)</li>
</ol>

A. はつまらないのでここではやらない.

B. は素直だが処理が必要ないケースの判定が微妙に重複している.

C. は (depth が取れないので) 2 pass 舐める必要があるが汎用的になりそう.

B. および C. を試す.

### B. 処理すべき `.value` がなくなるまで繰り返す. (例えば全ての `.value` が `"expected"` を持つ.)

- 分岐は if-then-else-end [[doc](https://jqlang.github.io/jq/manual/#if-then-else-end)] がある.
- ループは while [[doc](https://jqlang.github.io/jq/manual/#while)] や until [[doc](https://jqlang.github.io/jq/manual/#until)] がある.
- all [[doc](https://jqlang.github.io/jq/manual/#all)] /any [[doc](https://jqlang.github.io/jq/manual/#any)] もある.
- 関数も定義できる. [[doc](https://jqlang.github.io/jq/manual/#defining-functions)]

```
$ cat test_result.json | jq '.tests' | jq 'def should_process(x): x | has("expected"); until(. | to_entries | map(should_process(.value)) | all; . | to_entries | map(.key as $parent_key | if (should_process(.value)) then [.] else (.value | to_entries | map(.key |= $parent_key + "/" + .)) end) | flatten | from_entries)' | jq -C . | head -n 10
{
  "virtual/hoge/http/tests/protocol/domain0/test0.js": {
    "expected": "PASS",
    "actual": "SKIP",
    "other": null
  },
  "virtual/hoge/http/tests/protocol/domain0/test1.js": {
    "expected": "CRASH",
    "actual": "CRASH",
    "other": null,
```

よさそう.

### while と until の違い

驚きの事実なのだが, `while(condition; f)` と `until(condition | not; f)` は等価ではない.

`while(condition; f)` は `condition` が満されなくなくなるまで `f` を繰り返し適用し, 満たさなくなったものは出力されない.
つまり `. | f^n` が初めて `condition` を満たさないとき, `. | f`, `. | f^2`, ..., `. | f^{n-1}` を出力する.

`until(condition | not; f)` は `condition` が満されなくなくなるまで `f` を繰り返し適用し, 初めて満たさなくなったもののみを出力する.
つまり `. | f^n` を出力する.

```
$ echo 1 | jq '[while(. < 100; . * 2)]'
[
  1,
  2,
  4,
  8,
  16,
  32,
  64
]

$ echo 1 | jq 'while(. < 100; . * 2)'
1
2
4
8
16
32
64

$ echo 1 | jq '[until(. < 100 | not; . * 2)]'
[
  128
]

$ echo 1 | jq 'until(. < 100 | not; . * 2)'
128
```

これマジ!?

(jq Manual の "See advanced topics below." ってどこを見ればいいんだ?)

だから until を使う必要があったんですね.

## C. 先に JSON の max depth を取っておいてその回数繰り返す. (1回の処理につきネストがひとつ減るので.)

```
$ echo '0\n"hoge"\n[]\n{}\n[{"a": []}, 1]\n[{"a": []}, [{"b": 1}, {"c": {"d": 1}}]]\n' | jq 'def depth(x): if (. | [type] | inside(["array", "object"]) | not) then 0 else ({depth: 0, xs: [x]} | until((.xs | length) == 0; {depth: (.depth + 1), xs: (.xs | map(if (. | type == "array") then . else (if (. | type == "object") then (to_entries | map(.value)) else [] end) end) | flatten)}) | .depth) end; depth(.)'
0
0
1
1
2
4
```

はい. BFS っぽく root から `{depth, xs}` を持ち回って空になるまで `depth <- depth + 1; xs <- xs.map(children).flatten()` するだけですね.

```
$ cat test_result.json | jq '.tests' | jq 'def depth(x): if (. | [type] | inside(["array", "object"]) | not) then 0 else ({depth: 0, xs: [x]} | until((.xs | length) == 0; {depth: (.depth + 1), xs: (.xs | map(if (. | type == "array") then . else (if (. | type == "object") then (to_entries | map(.value)) else [] end) end) | flatten)}) | .depth) end; def should_process(x): x | has("expected"); {i: depth(.), x: .} | until(.i == 0; {i: (.i - 1), x: (.x | to_entries | map(.key as $parent_key | if (should_process(.value)) then [.] else (.value | to_entries | map(.key |= $parent_key + "/" + .)) end) | flatten | from_entries)}) | .x' | jq -C . | head -n 10
{
  "virtual/hoge/http/tests/protocol/domain0/test0.js": {
    "expected": "PASS",
    "actual": "SKIP",
    "other": null
  },
  "virtual/hoge/http/tests/protocol/domain0/test1.js": {
    "expected": "CRASH",
    "actual": "CRASH",
    "other": null,
```

よさそう.

## 結局欲しかったもの

少しだけ短かい B. を使う. あとはちょろっとフィルタするだけである.

```
$ cat test_result.json | jq '.tests' | jq 'def should_process(x): x | has("expected"); until(. | to_entries | map(should_process(.value)) | all; . | to_entries | map(.key as $parent_key | if (should_process(.value)) then [.] else (.value | to_entries | map(.key |= $parent_key + "/" + .)) end) | flatten | from_entries)' | jq '. | with_entries({"key": .key, "value": {"expected": .value.expected, "actual": .value.actual}})' | jq 'with_entries(select(.value.actual != "SKIP"))' | jq 'with_entries(select(.value.actual != .value.expected))' | jq -C .
{
  "virtual/hoge/http/tests/protocol/domain1/test0.js": {
    "expected": "PASS",
    "actual": "FAIL"
  }
}
```

ええやん!! こういうのが欲しかった!!

で, これどうやってメンテするんだろ...?
