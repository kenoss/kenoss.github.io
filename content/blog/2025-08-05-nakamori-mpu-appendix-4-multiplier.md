+++
title = "中森 MPU 行間埋め Appendix IV 後半 乗算器"
slug = "2025-08-05-nakamori-mpu-appendix-4-multiplier"

[taxonomies]
"tags" = ["tech", "中森 MPU"]

[extra]
"use_katex" = true
+++

$$
\gdef\defeq{\mathrel{\mathop:}=}
$$

[前回](../2025-08-04-nakamori-mpu-appendix-4-adder) の続きで乗算器.

コードは Python.

## Disclaimer

論理合成とか信号処理とか全然わからない. 自分が納得できるまで調べた結果をまとめているだけである.

あと今回の内容は
[Wikipedia -- ブースの乗算アルゴリズム](https://ja.wikipedia.org/wiki/%E3%83%96%E3%83%BC%E3%82%B9%E3%81%AE%E4%B9%97%E7%AE%97%E3%82%A2%E3%83%AB%E3%82%B4%E3%83%AA%E3%82%BA%E3%83%A0)
を読めば大体わかるものである.

## 問題設定

Q. n bit の整数ふたつ $x, y$ を乗算したい.

signed と仮定してよい. (本書参照.)

(非標準的記法) `0b<abcd>_<efgh>` などを下位 bit から先に書いて `0q<hgfe>~<dcba>` などと書くことにする. (数学者かリトルエンディアンの気持ち.)

$X$ の $i$ bit 目を $X_i$ で表す.

$$
\begin{align*}
xy
&= x \left( \sum_{i=0}^{\infty} 2^i y_i \right) \\\\
&= \sum_{i=0}^{\infty} (2^i x) y_i
\end{align*}
$$

愚直にやると n bit 乗算は n 回和を取ることで実現できる. (固定 bit シフトはタダみたいなものなので忘れる.)
[modexp](https://en.wikipedia.org/wiki/Modular_exponentiation) みたいな感じですね.

```
# 8-bit 乗算
def mul8(x: int, y: int):
    r = 0
    for i in range(0, 8):
        print(i, x, y, r)
        y_ = y & 0b1
        a: int
        if y_ == 0b0:
            a = 0
        elif y_ == 0b1:
            a = x
        r += a
        x = x << 1
        y = y >> 1
    return r
```

```
>>> mul8(10, 224 + 6)
0 10 230 0
1 20 115 0
2 40 57 20
3 80 28 60
4 160 14 60
5 320 7 60
6 640 3 380
7 1280 1 1020
2300
```

ループ回数と和を減らしたい. (回路に落とすため, 和をしない場合は 0 を足すと見做す.)

## Booth のアルゴリズム

ところで `y = 0q1110~0000` の様に1が連続している部分がひとつのときはもっと簡単である.

$$
\begin{align*}
y
&= 2^0 + 2^1 + 2^2 \\\\
&= 2^3 - 2^0       \\\\
xy
&= 2^0 (-x) + 2^3 x
\end{align*}
$$

$-x$ を事前計算しておいて $x, -x$ をシフトしていけば和が2回になった.

`y = 0q0011~1100` の様に位置や幅が変わっても同じである.

$$
\begin{align*}
y
&= 2^2 + 2^3 + 2^4 + 2^5 \\\\
&= 2^6 - 2^2             \\\\
xy
&= 2^2 (-x) + 2^6 x
\end{align*}
$$

`y = 0q0110~0111` の様に1が連続する部分が複数ある場合, `y' = 0q0110~0000` と `y'' = 0q0000~0111` に分けて計算すればよい.

$$
xy = x (y' + y'') = 2^1 (-x) + 2^3 x + 2^5 (-x) + 2^8 x
$$

つまり一般に, 立ち上がり `0q01` のときはシフトした $-x$ を足し, 立ち下がり `0q10` のときはシフトした $x$ を足せばよい.

2-bit ずつ見ていく場合は以下の様になる.

```
# 8-bit 乗算, Booth のアルゴリズム, 幅 2
def mul8_booth2(x: int, y: int):
    mx = -x
    # y の 0 bit 目を下の i = 0 のときの 0b10 で足し込みたいので仮想的な -1 bit 目を値 0 で追加.
    y = y << 1
    r = 0
    # y の -1 bit 目を足したのでループ回数は1回増える.
    for i in range(0, 9):
        print(i, x, mx, y, r)
        y_ = y & 0b11
        a: int
        if y_ == 0b00:
            a = 0
        elif y_ == 0b01:
            a = x
        elif y_ == 0b10:
            a = mx
        elif y_ == 0b11:
            a = 0
        r += a
        x = x << 1
        mx = mx << 1
        y = y >> 1
    return r
```

```
>>> mul8_booth2(10, 224 + 6)
0 10 -10 460 0
1 20 -20 230 0
2 40 -40 115 -20
3 80 -80 57 -20
4 160 -160 28 60
5 320 -320 14 60
6 640 -640 7 -260
7 1280 -1280 3 -260
8 2560 -2560 1 -260
2300
```

これだとまだループ回数は減っていない. 幅を 3 にしよう. 先程とは違い, 立ち上がりと立ち下がりが同時に起き得ることに注意する. (`0q010` と `0q101`.)

```
# 8-bit 乗算, Booth のアルゴリズム, 幅 3
def mul8_booth3(x: int, y: int):
    mx = -x
    y = y << 1
    r = 0
    for i in range(0, 5):
        print(i, x, mx, y, r)
        y_ = y & 0b111
        a: int
        if y_ == 0b000:
            a = 0
        elif y_ == 0b001:
            a = x
        elif y_ == 0b010:
            # a = (x << 1) + mx
            a = x
        elif y_ == 0b011:
            a = (x << 1)
        elif y_ == 0b100:
            a = (mx << 1)
        elif y_ == 0b101:
            # a = (mx << 1) + x
            a = mx
        elif y_ == 0b110:
            a = mx
        elif y_ == 0b111:
            a = 0
        r += a
        x = x << 2
        mx = mx << 2
        y = y >> 2
    return r
```

```
>>> mul8_booth3(10, 224 + 6)
0 10 -10 460 0
1 40 -40 115 -20
2 160 -160 28 60
3 640 -640 7 -260
4 2560 -2560 1 -260
2300
```

これでループと和の回数が半分になった. これが **Booth のアルゴリズム** である.

乗算器では BCLA ではなく Carry Save Adder (CSA) を用いるといったことは本書参照.
