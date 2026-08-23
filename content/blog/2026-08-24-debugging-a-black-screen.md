+++
title = "黒い画面のデバッグ: Claude Code が自作 Wayland compositor を直すまで"
slug = "2026-08-24-debugging-a-black-screen"

[taxonomies]
"tags" = ["tech", "sabiniwm", "wayland-compositor", "Asahi Linux", "AI"]
+++

## 前書き

人間です. Claude Code (Opus 5) に自作 Wayland compositor のバグを修正してもらいました.
自分ではわからず放置していたので嬉しい. 修正までに至る試行錯誤が良かったので本人に解説してもらいます [^1] [^2].

## 黒い画面には何も書かれていない

Claude Code です. 以下は今回の調査の記録で, ほぼそのまま時系列に沿っています.
先に結論を書いてしまうと, 原因は独立した二つのバグで, どちらも
「smithay の `anvil` が前提にしている挙動が Apple silicon で成立しない」という同じ形をしていました.
そして調査の大半は, バグそのものではなく **失敗が一切観測できない** という状況と戦うことに費やされました.

### 症状

- M2 MacBook Air, Asahi Linux (Fedora Asahi Remix). TTY から直接 compositor を起動する運用.
- sabiniwm を起動すると画面が黒くなり, 以降 TTY にも戻ってこない.
- sway は正常に動く.
- niri など他の smithay ベースの compositor も同様に動かない.
- 去年の11月から再現し続けている. きっかけはバッテリー切れだったらしい [^3].

「sway は動く」「smithay ベースは全滅」という切り分けが最初から与えられていたのは大きくて,
これは最後まで一貫して正しい道標でした.

なお私に許されていたのはリポジトリとログを読むことだけで [^4], compositor を実際に起動して試すのは
人間の担当です. そして一回試すたびに黒画面になり, 電源長押しで再起動する必要がある.
**1回の実験が1回の再起動と等価**という制約が, このあとの進め方をほぼ決めました.

### まず環境を読む

`/dev/dri` を見たところで話が決まりました.

```
$ for d in card0 card1; do echo "$d -> $(basename $(readlink -f /sys/class/drm/$d/device/driver))"; done
card0 -> apple-drm
card1 -> asahi

$ ls /sys/dev/char/226:0/device/drm   # card0
card0  controlD64                     # ← renderD* が無い

$ ls /sys/dev/char/226:1/device/drm   # card1
card1  renderD128
```

- `card0` = `apple-drm`. ディスプレイサブシステム. KMS を持ち `eDP-1` が生えているが, **render node が無い**.
- `card1` = `asahi`. GPU. render node (`renderD128`) を持つが, **KMS を持たない** (connector が0個).

いわゆる split display/render 構成です. スキャンアウトする device と, 描画する device が別物.

そしてもう一つ. 手元にあった約1年前のメモでは, この番号が `card1` = GPU, `card2` = display でした.
今は `card0` = display, `card1` = GPU. **入れ替わっている.**
`simpledrm` が minor 0 を解放するタイミングと `apple-drm` の登録が競合するためで,
journal に残っていた20回分の boot では現在の並びで固定されていました.
「一時的に起動できないことがあったが, やがて恒久化した」という症状の変化と一致します.

### 一つ目のバグ: 番号順で GPU を選んでいた [^5]

sabiniwm も, `anvil` も niri も, render node をこう決めています.

```rust
smithay::backend::udev::primary_gpu(session.seat())?
```

その中身 (`smithay/src/backend/udev.rs`, sabiniwm が pin していた `8886e3b` 時点) は,

1. PCI の親を辿って `boot_vga=1` の device を探す
2. 見つからなければ `all_gpus()`, すなわち **`/dev/dri/card*` をパス名でソートして先頭**

Apple silicon の GPU は platform device なので PCI の親が無く, 1 は必ず外れます.
よって常に 2 が効く. 以前は `card1` (GPU) が先頭だったので偶然正しく動いていて,
今は `card0` (display 専用) が先頭になった. sabiniwm はここから render node を逆引きしようとして,

```rust
device_node
    .node_with_type(NodeType::Render)
    .ok_or_else(|| eyre::eyre!("no corresponding render node for: path = {}", ...))?
```

`None` を掴んで起動時にエラー終了する, というのが一つ目です.

#### wlroots は推測しない

では sway はなぜ平気なのか. wlroots は候補を **KMS の有無で絞ります**
(`backend/session/session.c`).

```c
static struct wlr_device *open_if_kms(struct wlr_session *restrict session,
		const char *restrict path) {
	...
	if (!drmIsKMS(dev->fd)) {
		wlr_log(WLR_DEBUG, "Ignoring '%s': not a KMS device", path);
		wlr_session_close_file(session, dev);
		return NULL;
	}
	return dev;
}
```

そして render node は名前から推測せず, EGL に聞く. しかも split 構成の分岐を明示的に持っています
(`render/egl.c`).

```c
	} else if (!(match->available_nodes & (1 << DRM_NODE_RENDER))) {
		// Likely a split display/render setup. Pick the primary node and hope
		// Mesa will open the right render node under-the-hood.
```

実際, 動いている sway のプロセスは `/dev/dri/renderD128` を開いていて,
その fd が子プロセス全部に継承されていました. `card0` で KMS, `renderD128` で描画,
という正しい組み合わせで動いている.

そして重要なことに, sabiniwm 側も `device_added()` の中では既に EGL 経由で render node を引いていました.
使っていなかっただけです. なので修正は「起動時に推測するのをやめ, `device_added()` が
`EGLDevice::try_get_render_node()` で得た値を採用する」だけで済みました
(commit [16c8a65f](https://github.com/kenoss/sabiniwm/commit/16c8a65f43b5771b59054e927e84359d6f49501d)).
KMS を持たない `card1` は `DrmDevice::new()` が `EOPNOTSUPP` で落ちるので自動的に除外されます.

### ここで一度間違えます

私はこの時点で「原因はこれ一つ」と判断し, さらに
「起動コマンドには既に `SABINIWM_DRM_DEVICE_NODE=/dev/dri/renderD128` が付いているので,
今の Mesa なら現状のコードでも通るはずだ」と予測しました.

外れでした. 直しても黒画面のままです.

言い訳をすると, この予測にはそれなりの根拠がありました. EGL は健全 (`Apple M2 (G14G B0)`, Mesa 25.2.0),
GBM のバッファ確保も3つの node すべてで成功, コードを追う限り環境変数指定の経路に落ちる箇所は無い.
実際, その読み自体は正しかったのです. 単に**バグがもう一つあった**だけで.

ここで方針を変えました. 推測でコードを読み続けるのをやめて, **観測できるようにする**方を先にやる.

### 見えないという問題

そもそもなぜ何も分からなかったのか. `LibSeatSession` を作った瞬間に VT は `KD_GRAPHICS` になります.
以降 stderr に何を書いても画面には出ない. VT 切り替えも効かない. 電源長押しで再起動するしかない.

そして sabiniwm は udev backend のとき, ちゃんとログをファイルに書いていました. `/tmp/sabiniwm.log` に.

`/tmp` は tmpfs です. **黒画面 → 電源断 → 復帰したときにはログが消えている.**
ログが取れていなかった理由がこれでした.

そんなわけで, バグを追う前に以下を入れました.

- ログの出力先を `$XDG_STATE_HOME/sabiniwm/sabiniwm.log` に移す. 再起動を跨いで残るように
  (commit [51098e35](https://github.com/kenoss/sabiniwm/commit/51098e35e61f16428aaa191f4b6c75b6b823cadd)).
- `run()` を抜けるエラーを `error!` でログにも流す. 今までは stderr = 見えない黒画面行きだった
  (commit [88ecd09f](https://github.com/kenoss/sabiniwm/commit/88ecd09fd49a4fc142db76df15fee171bf08189d)).
- 初期化が既定20秒で終わらなければ console を text mode に戻して `exit(1)` する watchdog.
  `/dev/tty` に `VT_SETMODE(VT_AUTO)` と `KDSETMODE(KD_TEXT)` を投げるだけ
  (commit [007a85d7](https://github.com/kenoss/sabiniwm/commit/007a85d749b6491234a2fc937ab5c94b0b4109ef)).
- output が0個のまま `init()` が成功しないようにする.
  connector のセットアップ失敗は `error!` で握り潰されるので, 「動いているのに真っ黒」が容易に作れてしまう
  (commit [8dc14fab](https://github.com/kenoss/sabiniwm/commit/8dc14fab0868e3e57e3063c2eb6ade53d185984b)).

さらに, VT を奪わずに device 選択・EGL・renderer 構築までを実行する `probe_drm` という example を足しました
(commit [30ed7a7d](https://github.com/kenoss/sabiniwm/commit/30ed7a7dbed84aa3a1d92be2fa1f09b7f2c3cd1c)).
modeset だけが DRM master を必要とするので, その手前までは sway の中から実行できます.
これで**再起動を消費せずに検証できる範囲**が一気に広がりました.

```
$ cargo run -p sabiniwm --example probe_drm
  all_gpus()               = ["/dev/dri/card0", "/dev/dri/card1"]
  primary_gpu()            = Some("/dev/dri/card0")     ← 一つ目のバグ

== /dev/dri/card0
  driver (sysfs)           = Some("apple-drm")
  node_with_type(Render)   = None                       ← 旧コードはここで詰む
  EGL render_device_path   = Ok("/dev/dri/renderD128")
  try_get_render_node()    = Some(DrmNode { dev: 57984, ty: Render })   ← EGL 経由なら引ける
```

### heartbeat

それでも次の一手が決まりませんでした. ログにはこう出ていたのです.

```
23:01:03.679 WARN Error during rendering:
  TemporaryFailure(Access(AccessError { errmsg: "Page flip commit failed",
    dev: Some("/dev/dri/card0"), source: Os { code: 16, kind: ResourceBusy } }))
```

そしてこの1行のあと, 58秒間まったくログがありません.

リトライは走るはずのコードなので, 成功したのか, 無言でスピンしているのか,
イベントループごと止まったのかが区別できない. ここで私は smithay の内部状態機械について
何通りも仮説を立てましたが, 今思えば全部無駄でした. 分からないものは測るべきです.

というわけで, レンダーループのカウンタを数えて定期的に1行吐くようにしました
(commit [19ac6117](https://github.com/kenoss/sabiniwm/commit/19ac6117578fca15ca79d6614d1f274cb70fa9aa)).

- `rendered`: 描いたフレーム数 (damage が無ければ数えない)
- `queued`: KMS に渡した数
- `failed`: KMS に蹴られた数
- `vblanks`: 受け取った vblank の数, すなわち **実際に画面に出た**数

この4つが並んでいれば, 「ハングしている」「描いているが KMS に届いていない」
「KMS は受けたがスキャンアウトされていない」を一発で区別できます.

次の起動でこう出ました.

```
23:01:08.675 INFO heartbeat rendered=1  queued=0 failed=1  vblanks=0
23:01:13.675 INFO heartbeat rendered=0  queued=0 failed=0  vblanks=0
   ...
23:01:35.98  INFO InputEvent::DeviceAdded: "Apple MTP keyboard"
23:01:36.99  INFO run_manage_hook, app_id = Some("Alacritty"), title = "on_workspace_3"
23:01:37.96  INFO run_manage_hook, app_id = Some("emacs")
23:01:38.678 INFO heartbeat rendered=13 queued=0 failed=13 vblanks=0
```

決定的でした. `queued=0`, `vblanks=0`. **一度もフレームが KMS に渡っていない.**
一方でキーバインドは効いていて, alacritty 3枚と emacs がちゃんと起動して manage hook まで走っている.
compositor は完全に正常で, page flip だけが100%失敗していたのです.
「黒画面だがキー入力は受け付ける」の正体がこれでした.

ちなみに入力デバイスの追加が58秒後になっているのは, libinput がキューした `DEVICE_ADDED` を
fd が readable になるまで吐かないからで, つまりあの時刻が「人間が初めてキーを触った瞬間」です.
イベントループはずっと生きていました.

### 二つ目のバグ: DCP の電源を落としていた

`EBUSY` は, 同じ CRTC の前のコミットが完了していないうちに nonblock なコミットが来たときに
カーネルが返すものです (`drm_atomic_helper_setup_commit()`). つまり CRTC が永久に busy のまま.

ここで sway とカーネルログを比較しました.

```
### sway
dcp_poweron() starting                       ← boot 時の1回だけ
set_digital_out_mode ... "2560x1600"

### sabiniwm
dcp_poweroff() done                          ← ★これ
dcp_poweron() starting
set_digital_out_mode ... "2560x1600"
```

**sway は DCP を一度も電源断していません.** sabiniwm だけが poweroff → poweron している.

出どころは `device_added()` の一行でした.

```rust
DrmDevice::new(fd.clone(), true)
//                         ^^^^ disable_connectors
```

`anvil` 由来で, 「前の compositor が変な状態を残していても大丈夫なように,
全 connector と全 plane を無効化して既知の状態にする」というものです.
ところが Apple の DCP はこれに耐えられない. 全 connector を切ると DCP が電源断し,
**その後のモードセットは成功して画面モードも設定されるのに, CRTC が二度と page flip を完了しなくなる.**
以降すべてのコミットが `EBUSY`.

wlroots はデバイスをリセットしません. モードセットは `ALLOW_MODESET` のブロッキング,
page flip だけ `NONBLOCK`, それだけです. ここでも「余計なことをしない」方が勝っていました.

`disable_connectors` を `false` にして, 次の起動:

```
heartbeat rendered=37 queued=37 failed=0 vblanks=37
heartbeat rendered=26 queued=26 failed=0 vblanks=26
heartbeat rendered=54 queued=54 failed=0 vblanks=54
```

`rendered == queued == vblanks`, `failed=0`, page flip の失敗は0件. 直りました [^6].

なお同じ `reset_state()` は `DrmError::TestFailed` の復旧経路にも残っていて,
そちらを踏むとセッション中に同じ黒画面に落ちます. 併せて塞ぎました (commit [c54002da](https://github.com/kenoss/sabiniwm/commit/c54002dae473d919fb6dd4f3df2638ba937ee147)).

### おまけ: syncobj

ついでに一つ見つけたものを. `linux-drm-syncobj-v1` (explicit sync) が有効化されていませんでした.
smithay は「render node に対応する primary node」を開いている device の中から探すのですが,
split 構成ではそれは `card1` で, KMS を持たないので開いていない. よって永久に空振りします.

実測するとこうでした.

```
/dev/dri/card0       SYNCOBJ=no  TIMELINE=no  EVENTFD=n/a (Operation not supported)
/dev/dri/card1       SYNCOBJ=yes TIMELINE=yes EVENTFD=yes
/dev/dri/renderD128  SYNCOBJ=yes TIMELINE=yes EVENTFD=yes
```

`linux-drm-syncobj-v1` はクライアントが**描画に使う GPU 上**で作るタイムラインを import するプロトコルなので,
見るべきは `renderD128` の方でした. render node は非特権で開かれる前提のものなので, 直接開けば済みます.

無いと困るかというと, クライアントは implicit sync にフォールバックするだけで, Mesa 上では正しく動きます.
本当に必須なのは implicit sync を持たない NVIDIA プロプライエタリドライバなので, ここでは該当しません.
とはいえ直せるものは直しました (commit [e0ae8e2e](https://github.com/kenoss/sabiniwm/commit/e0ae8e2e29e1bffa941af5f9cdac387a4d09107d)).

### upstream ではどうなっているか [^7]

今回直した二つは, どちらも `anvil` から受け継いだコードです. 本家はどうなっているのか.
smithay `master` (`92ba0e92`, 2026-08-12) と niri `main` (`dd75865f`, 2026-08-21) を読みました.

**一つ目は smithay で既に直っていました.**
[`e9f00726`](https://github.com/Smithay/smithay/commit/e9f007263a1d19ef705f56facf468058c71c8567) と
[`6329f5b2`](https://github.com/Smithay/smithay/commit/6329f5b21693c53b0370cc7f62523ea4d0a42073)
(どちらも 2025-12-18) で `primary_gpu()` の優先順位が変わっています.

```rust
// 1st priority: GPU with boot_vga=1
// 2nd priority: GPU with a render node          ← 追加された
// 3rd priority: first GPU in alphabetical order
```

この2番目のおかげで, このマシンでは render node を持たない `card0` が飛ばされて `card1` が返ります.
つまり sabiniwm が pin していた `8886e3b` (2025-03-18) が古かっただけで, 私が「wlroots はこうしている」
と言って書いた修正は, 本家では別の形で先に入っていたわけです. 私の方が KMS の有無で絞るぶん wlroots 寄りですが,
このハードウェアではどちらでも結果は同じになります.

**二つ目は `anvil` に残っています.** `anvil/src/udev.rs`:

```rust
let (drm, notifier) = DrmDevice::new(fd.clone(), true).map_err(DeviceAddError::DrmDevice)?;
```

syncobj の device 選択も同じく残っています. `primary_gpu` の primary node を `backends` から引く,
というあの形のままです.

**niri は両方とも踏みません.**

- 一つ目: pin している smithay (`4cf0b620`, 2026-08-12) が上の修正を含んでいる.
- 二つ目:
  [`63f58086`](https://github.com/YaLTeR/niri/commit/63f58086b9783fd80839ef35df82b05682c1c495)
  (2025-11-19) *"tty: Avoid modeset on adding device if possible"* で
  `DrmDevice::new(device_fd.clone(), false)` になっている.
  動機は DCP ではなく「device 追加時に不要な modeset をしない」ですが, 触っている場所は同じです.
- syncobj: niri は `linux-drm-syncobj-v1` を実装していないので, そもそも該当しません.

面白いのは二つ目で, niri は今回の障害が出はじめたのとほぼ同時期に, まったく別の動機で同じ一行を倒しています.
「余計なことをしない」に寄せると, 理由が何であれ同じ地雷を踏まなくなる.


### 学んだこと

**推測をやめて計測する.** 当たり前ですが, 私は今回それを一度サボって外しました.
特に「1回の実験が1回の再起動」という環境では, 一手を無駄にする代償が大きい.
コードを読んで仮説を立てる時間より, ログを1行足す時間の方が安いなら, 足すべきです.

**失敗が見えるようにするのは, バグを直すのと同じくらいの仕事.**
ログが tmpfs に書かれていて再起動で消えていた, というのが数ヶ月分の情報を失わせていました.
VT を奪う前に何を記録するか, 失敗したときに console をどう返すか,
という設計は compositor では「あると便利」ではなく必須の機能です.

**「動いている実装」との差分を取る.** sway が動いていたことが最大の手がかりでした.
wlroots のソースが手元にあるおかげで, 「wlroots は KMS で絞る」「wlroots はリセットしない」
という二つの差分が両方とも当たりだった. どちらも wlroots 側は
「推測しない」「余計なことをしない」という同じ方針で, `anvil` 側が親切心で余分なことをしていた形です [^8].

**環境の前提は静かに壊れる.** 二つのバグはどちらも sabiniwm の変更で壊れたものではありません.
一つ目は, 番号順という推測がたまたま当たっていた状態から, たまたま外れる状態に移っただけ
(しかも環境変数で回避済みだったので, 今回の障害の引き金ではなかった).
二つ目に至っては, 同じコードが同じハードウェアで, ある日を境に通らなくなった.
`anvil` のコードは何も悪くなっていないのに, ある日突然動かなくなる.

### 修正一覧

最終的に10コミットになりました. 「まず失敗が見えるようにする」→「黙って壊れないようにする」→
「実際に直す」の順です.

- [`e0ae8e2e`](https://github.com/kenoss/sabiniwm/commit/e0ae8e2e29e1bffa941af5f9cdac387a4d09107d) fix: Enable explicit sync on a split display/render device
- [`c54002da`](https://github.com/kenoss/sabiniwm/commit/c54002dae473d919fb6dd4f3df2638ba937ee147) fix: Don't reset the DRM device to a known state ← 本命2
- [`19ac6117`](https://github.com/kenoss/sabiniwm/commit/19ac6117578fca15ca79d6614d1f274cb70fa9aa) feat: Log a heartbeat of the render loop
- [`1dbb7164`](https://github.com/kenoss/sabiniwm/commit/1dbb7164240eb8ec9e2d9570058f787219ac83c2) fix: Don't panic when a device has no usable EGL display
- [`16c8a65f`](https://github.com/kenoss/sabiniwm/commit/16c8a65f43b5771b59054e927e84359d6f49501d) fix: Don't refuse to start on a split display/render device ← 本命1
- [`8dc14fab`](https://github.com/kenoss/sabiniwm/commit/8dc14fab0868e3e57e3063c2eb6ade53d185984b) fix: Fail when the udev backend sets up no output
- [`007a85d7`](https://github.com/kenoss/sabiniwm/commit/007a85d749b6491234a2fc937ab5c94b0b4109ef) feat: Restore the console instead of leaving a black screen
- [`88ecd09f`](https://github.com/kenoss/sabiniwm/commit/88ecd09fd49a4fc142db76df15fee171bf08189d) fix: Log the error that terminates `SabiniwmState::run()`
- [`51098e35`](https://github.com/kenoss/sabiniwm/commit/51098e35e61f16428aaa191f4b6c75b6b823cadd) fix: Put the udev log file outside of tmpfs
- [`30ed7a7d`](https://github.com/kenoss/sabiniwm/commit/30ed7a7dbed84aa3a1d92be2fa1f09b7f2c3cd1c) feat: Add `probe_drm` example to diagnose udev backend setup

一つ目は上で見たとおり smithay 本体で既に直っています. 二つ目と syncobj は `anvil` に残っているので,
そこから派生した compositor が Apple silicon 上で同じことになる可能性はまだあります.


[^1]: (人間) というのもあるし, 自分が知らなかったことなので記録しておくという意味でも.
[^2]: (人間) 補足など以外はほぼ AI 出力ママです. 自分とはスタイルが違うけど, まぁ著者が違うしね?
      文責は自分にあります.
[^3]: (人間) Chat GPT (Coding agent ではなく chat) は強制電源断 (バッテリー切れ) で
      display pipeline / DCP / iGPU firmware が中途半端な状態になったのではと推測していた.
      去年12月時点では「この現象は発現したり収まったりする」と相談しているので, 確定落ちになったのはもっと後.
      Opus 君がこう言っているのは自分が貼った Chat GPT との会話ログにその回が含まれていなかったから. (ごめん.)
[^4]: (人間) 最初に「permission: このマシンで別の作業を並行してやっているので, 一旦このリポジトリ配下
      (sabiniwm, smithay, sway, wlroots を submodule) と log を読んでわかる範囲で.」と伝えました.
[^5]: (人間) 実際はこのあたりの事情は Anvil を fork する段階で M2 mac で動作確認したときに把握済みで
      [[commit](https://github.com/kenoss/sabiniwm/commit/09c14c419b886cd54ef3fd5545a147e2f0682329)],
      だから `SABINIWM_DRM_DEVICE_NODE=/dev/dri/renderD128` を付けて起動するという運用になっていたのでした.
      使っていた起動オプションだけ教えて, 探索はそのまま続けてもらいました. 実際修正によって (無限に遅延させていた)
      環境変数を不要にするというのが実現できたのでヨシ!
[^6]: (人間) AI 君の筆がそっけなさすぎるけど, ここめちゃ盛り上りポイントです.
[^7]: (人間) niri を読んでもらったのはこの記事の第一稿を書いてもらった後です. niri は GPL なので一応.
[^8]: (人間) これが余計なことかどうかは今の自分には判断しかねる. こうなった文脈も追ってないし.
