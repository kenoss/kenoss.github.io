+++
title = "中森 MPU 行間埋め Appendix IV 前半 加算器"
slug = "2025-08-04-nakamori-mpu-appendix-4-adder"

[taxonomies]
"tags" = ["tech", "中森 MPU"]

[extra]
"use_katex" = true
+++

$$
\gdef\defeq{\mathrel{\mathop:}=}
$$

## モチベーション

[[中森 MPU](https://shop.cqpub.co.jp/hanbai/books/45/45561.html)] は面白いのだが行間がそこそこある. なので埋める.

## Disclaimer

論理合成とか信号処理とか全然わからない. 自分が納得できるまで調べた結果をまとめているだけである.

## 問題設定

Q1.
n bit の整数ふたつ $x, y$ を加算することを考える: $s = x + y \mod 2^n$. 更に (n-1 bit 目の) carry $c_{n-1}$ も合わせて取得したい.

適切に問題設定してやれば unsigned のときのみを考えればよい. (本書参照.)

色々便利なので Lisp 的な疑似コードを用いる.

## Full adder

n = 1 のとき.

```
s = (xor x y)
c_0 = (and x y)
```

とすればよい. これを **Half Adder** と呼ぶ.

しかしこれでは話が進まない. よって問題を更新する:

Q2.
n bit の整数ふたつ $x, y$ と 0 bit 目の carry $c_{-1}$ を加算することを考える: $s = x + y + c_{-1} \mod 2^n$.
更に (n-1 bit 目の) carry $c_{n-1}$ も合わせて取得したい.

これも同様で, 以下の様にすればよい:

```
s = (xor (xor x y) c_{-1})
c_0 = (and (or x y) c_{-1})
```

($c_0$ の妥当性は真偽値表を書けばよい.) これを **Full Adder** と呼ぶ.

Oversimplify して, 回路遅延はゲートに依らず $d$ であり通過したゲートの個数のみで決まり, 配線長の問題なども考えないとする.

上記で $s$ も $c_0$ も2段の回路なので遅延は $2d$ である. 以降では $d$ で除して $d = 1$ として考える.

## Ripple Carry Adder (RCA)

i bit 目の入力 $x$, $y$, sum および carry をそれぞれ $x_i$, $y_i$, $s_i$, $c_i$ と書く.

```
s_i = (xor (xor x_i y_i) c_{i-1})
c_i = (and (or x_i y_i) c_{i-1})
```

とすればよい. これを **Ripple Carry Adder (RCA)** と呼ぶ.

さて, よく見るとこれは Full Adder を連結したものになっている. これを一般化しておく.

w bit の Q2 を解く加算器 Hoge があったとき, それを b 個接続することにより $n = bw$ bit の加算器が作れる. これを (n, b, w) Block Hoge と呼ぶことにする. (非標準的用語)

RCA は (n, n, 1) Block Full Adder である.

回路遅延: $s_i, c_i$ は $c_{i-1}$ に依存する. よって遅延は $2n$ である.

## Carry Look Ahead (CLA) と Block Carry Look Ahead (BCLA)

この遅延を小さくしたい. (w, 1, w) で適当に良い性質のものが作れると良さそうである. w が小さいとき ($w \le 4$ とか) に良いものがあり, **Carry Look Ahead (CLA)** と呼ぶ.
後にこれを使って **(bw, b, w) Block CLA** を作る.

こう置く:

```
g_i = (and x_i y_i)
p_i = (xor x_i y_i)
```

すると

```
s_i = (xor p_i c_{i-1})
c_i = (or g_i (and p_i c_{i-1}))
```

である. $c_i$ の方だけ Boole 代数で見て展開すると,

$$
\begin{align*}
c_i
&= g_i + p_i c_{i-1} \\\\
&= g_i + p_i (g_{i-1} + p_{i-1} c_{i-2}) \\\\
&= ... \\\\
&= g_i + p_i g_{i-1} + p_i p_{i-1} g_{i-2} + ... + p_i ... p_0 c_{-1}
\end{align*}
$$

さて, これを計算するにはどう2項演算をしていくかでバリエーションがある. ひとつめの式で計算するのが RCA であった.

$$
\begin{align*}
g_i^\ast &\defeq g_i + p_i g_{i-1} + p_i p_{i-1} g_{i-2} + ... + p_i p_{i-1} ... p_0 g_0 \\\\
p_i^\ast &\defeq p_i ... p_0
\end{align*}
$$

と置けば,

$$
c_i = g_i^\ast + p_i^\ast c_{-1}
$$

である. $g_i^\ast$ には最大 i+1 項の積, i+1 項の和があるので (共通化とか配線とか考えずに) テキトーにやると $2 \log_2(i+1)$ 段くらいでできる. 例えば $g_3^\ast$ は

```
  g_3^\ast
= g_3 + p_3 g_2 + p_3 p_2 g_1 + p_3 p_2 p_1 g_0
= (or (or g_3
          (and p_3 g_2))
      (or (and p_3
               (and p_2 g_1))
          (and (and p_3 p_2)
               (and p_1 g_0))))
```

よって w bit でこの加算器を作れば $c$ の回路遅延は $2 \log_2(w) + 2$ である. ($s$ も似たようなもの.)

これを使って (bw, b, w) の加算器を作ろう.

さて, 上記の $c_i$ では $g_i^\ast$ も $p_i^\ast$ も $c_i$ を含まず, $g_i^\ast$ と $p_i^\ast$ の計算が主要であった. すなわち

- 全 block で $g_i^\ast$ / $p_i^\ast$ は同時に計算できる. (最初の block の出力のみを block する.)
- propagation には 1 block あたり $2$ しかかからない.

従って遅延は $2 \log_2(w) + 2b$ である. これを **(bw, b, w) Block CLA** と呼ぶ.

(おそらく本書では (n, 1, n) Block CLA のことを CLA と呼んでいる気がする?
[Wikipedia](https://en.wikipedia.org/wiki/Carry-lookahead_adder) とかは CLA = Block CLA な気がする.)

$w = 4$ として RCA と比較すると ($n = bw = 4b$), RCA が $2n$, (n, n/4, 4) BCLA が $4 + n/2$ である.
つまり BCLA は定数倍を改善している.
(n が大きいときに (n, 1, n) で取れれば $O(\log(n))$ だが, 物理的な限界がある. ChatGPT によると $w = 4$ くらいが実用的な限界のようだ.
一応研究としては $w = 16$ とかもあるらしい
[[Design and Implementation of a 16 Bit Carry- Lookahead Adder](https://www.researchgate.net/publication/343982099_Design_and_Implementation_of_a_16_Bit_Carry-_Lookahead_Adder)].)

## fan-in

上記の $g_3^\ast$ では or と and は2項演算のみを用いた. 実際は and4 など 3〜4 入力くらいならひとつの CMOS ゲートとして作れるらしい. (詳細略.)

## Parallel Prefix Adder

さて, 上記の計算では共通項のことを何も考えなかった. 共通項をいい感じに計算するのが **Parallel Prefix Adder** である.
これは本書でわかりやすく解説してあるのでここでは扱わない.
