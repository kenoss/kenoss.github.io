+++
title = "memo: Wayland compositor と redraw と VBlank"
slug = "2024-10-29-vblank-and-redraw"

[taxonomies]
"tags" = ["memo", "tech", "wayland-compositor"]
+++

## Anvil vs niri

[smithay](https://github.com/Smithay/smithay) crate は Rust で Wayland を扱える素晴らしい crate です.

ですが付属の Wayland compositor 実装である [Anvil](https://github.com/Smithay/smithay/tree/master/anvil) が
([niri と比較すると](https://github.com/YaLTeR/niri)) かなり読みにくいという問題があります.
その一因として backend (udev, winit) が抽象的に分離されていないというのがある気がします.
このへんを整理していたのですが udev backend と winit backend で redraw の処理がかなり違うのがネックになってきました.

というわけで調査と比較するね. [はいどうぞ！](https://www.nicovideo.jp/watch/sm39360075)

## niri

<https://github.com/YaLTeR/niri/tree/f203c8729a8535f6a317df5a35dc01306be2e45c>

<https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/main.rs#L261-L263>

niri は [calloop::EventLoop::run()](https://docs.rs/calloop/latest/calloop/struct.EventLoop.html#method.run)
でループを駆動している. 中の [dispatch()](https://docs.rs/calloop/latest/calloop/struct.EventLoop.html#method.dispatch)
で Wayland client からのイベントなども処理している. `niri::State::refresh_and_flush_clients()` から
[niri::Backend::render()](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/niri.rs#L2708)
が呼び出される.

tty backend の場合は
[DrmCompositor::render_frame()](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/backend/tty.rs#L1274)
と
[DrmCompositor::queue_frame()](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/backend/tty.rs#L1300)
が呼び出される. しかしこのループを愚直に回すと以下の問題が出てしまう.

1. ディスプレイの FPS を上回る頻度で render してしまう.
2. rerender の必要がないのに render してしまう. (表示しているどのウインドウも更新がない場合など.)

1.は VBlank を使って解決する.
redraw は [niri::RedrawState](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/niri.rs#L314)
を使って管理されている. frame が scan-out されたら
[DrmEvent::VBlank](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/backend/tty.rs#L557)
が通知されるのでここで
[redraw state を Idle にする](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/backend/tty.rs#L1154).
[redraw state が Idle のものは skip される](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/niri.rs#L2223-L2226)
ので無駄な render が抑えられるというワケ.

2.は `wl_surface::commit` などを使って解決する.
[CompositorHandler::commit()](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/handlers/compositor.rs#L193),
つまり
[wl_surface::commit](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/src/wayland/compositor/handlers.rs#L248-L252)
が呼ばれるときに
[niri::Niri::queue_redraw()](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/handlers/compositor.rs#L193)
する.

他にも `queue_redraw()` を呼んでいるところはたくさんある. 例えばウインドウを掴んで移動しているときなどはウインドウの内容自体は変わっているとは限らないので
`queue_redraw()` する必要がある.

この様に, redraw を抑制し, 必要があるときだけ発火させるという感じでやっている. 上手いね.

因みにこの挙動は terminal を開いて `for i in $(seq 0 10000); do echo $i; sleep 1; done` とかやるとわかりやすい.

注意としては, niri はアニメーションもあるので
[アニメーション中は redraw を続ける](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/backend/tty.rs#L1162)
といった処理も必要になるあたりか.

winit backend も基本は同じだが, (僕の理解では) VBlank 通知に相当する frame callback を待たずに 
[winit::window::Window::request_redraw() している](https://github.com/YaLTeR/niri/blob/f203c8729a8535f6a317df5a35dc01306be2e45c/src/backend/winit.rs#L236-L240).

## Anvil

<https://github.com/Smithay/smithay/tree/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3>

udev/winit backend はそれぞれ異なるループを持つ.

まずは udev backend の方.

[main のループ](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/udev.rs#L486)
を持つこと自体は同じ. これは簡単に `calloop::EventLoop::run()` に置き換え可能である.
さて, この main ループ自体は rendering に絡まない. 実はもうひとつ rendering 用のループが存在する.

* [UdevData::render()](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/udev.rs#L1361)
* -> [oneshot timer](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/udev.rs#L1505-L1510) if reschedule; or
* -> [VBlank](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/udev.rs#L866)
* ---> [oneshot timer](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/udev.rs#L1351-L1356)

この oneshot timer のどちらかを何らかのタイミングでやめると描画されなくなる. (挙動がめちゃくちゃ追いにくい. もうちょっとなんとかならんかったんか?)

このループは概ね 1/FPS 秒ごとに1周する. scan-out されるパスは VBlank に入るが, scan-out されないケースでも (例えば `DrmCompositor` を使っている場合)
[DrmCompositor::render_frame()](https://smithay.github.io/smithay/smithay/backend/drm/compositor/struct.DrmCompositor.html#method.render_frame)
が呼び出される. (詳しくないのだけれど, 消費電力とか大丈夫なんですかね...?)

このループが切り離されているから main ループが簡素というワケ.

winit backend の場合
[main ループの中で render している](https://github.com/Smithay/smithay/blob/8e49b9bb1849f0ead1ba2c7cd76802fc12ad6ac3/anvil/src/winit.rs#L248).
この違いが backend を `state` から分離するのを難しくしている.

## Mutter (GTK)

<https://gitlab.gnome.org/GNOME/mutter>

読もうとしたけどまったくわからん. C 言語と GObject 難しすぎひん?

## KWin (KDE)

<https://github.com/KDE/kwin/tree/v6.2.2>

ここから雑度が上がる.

[main ループはここ](https://github.com/KDE/kwin/blob/v6.2.2/src/main_wayland.cpp#L630) で, それとは別に
[WaylandOutput が RenderLoop を保持している](https://github.com/KDE/kwin/blob/v6.2.2/src/backends/wayland/wayland_output.h#L92).
`RenderLoop` は [ふたつ timer を持っている](https://github.com/KDE/kwin/blob/v6.2.2/src/core/renderloop.cpp#L34-L41).
ふたつめの timer のことは忘れよう. (他に `RenderLoop::scheduleRepaint()` を呼び出しているの, `DrmOutput` だけだし.)

* `compositeTimer`
* -> [call frameRequested()](https://github.com/KDE/kwin/blob/v6.2.2/src/core/renderloop.cpp#L190)
* -> [RenderLoop::frameRequested() -> Compositor::handleFrameRequested()](https://github.com/KDE/kwin/blob/v6.2.2/src/compositor.cpp#L89)
* -> [WaylandCompositor::composite(](https://github.com/KDE/kwin/blob/v6.2.2/src/compositor_wayland.cpp#L295)]
* -> [call paintPass()](https://github.com/KDE/kwin/blob/v6.2.2/src/compositor_wayland.cpp#L372)
* -> VBlank
* -> [KWayland::Client::Surface::frameRendered() -> OutputFrame::presented()](https://github.com/KDE/kwin/blob/v6.2.2/src/backends/wayland/wayland_output.cpp#L128)
* -> [RenderLoopPrivate::notifyVblank で lastPresentationTimestamp しつつ](https://github.com/KDE/kwin/blob/v6.2.2/src/core/renderloop.cpp#L180)
* -> [call RenderLoopPrivate::scheduleRepaint()](https://github.com/KDE/kwin/blob/v6.2.2/src/core/renderloop.cpp#L162)
* -> [compositeTimer.start()](https://github.com/KDE/kwin/blob/v6.2.2/src/core/renderloop.cpp#L120)

これでループする.

上では scan-out されたケースを書いたが, そうでないケースは
[OutputFrame::~OutputFrame() から RenderLoopPrivate::scheduleNextRepaint() される](https://github.com/KDE/kwin/blob/v6.2.2/src/core/renderbackend.cpp#L58-L60).

つまり Anvil とだいたい一緒.

## まとめ

Anvil ベースのやつを改良しようと思ったけど KWin と似た方法だしこれが特別変なことをしているわけではないっぽいことがわかってしまった.
どうしよう... (まぁ構造をキープしつつマシにするのはどうとでもできる気はするが.)
