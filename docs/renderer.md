# 共通描画器とDesktop検証

担当: [Issue #4](https://github.com/rioX432/rust-mobile-game-lab/issues/4)。作業日: 2026-10-24 JST。
ブランチは`feat/shared-renderer`、基点は`feat/native-embedding`の`5fef344da0bf218bf1dfdf807f6399b94c007cb8`。

円・矩形の共通描画器と、Probeだけを接続するDesktopシェルを実装した。レース・通信は接続していない。
変更範囲は`crates/renderer/src/`、`apps/desktop/src/`、本書のみ。共有manifest、Cargo.lock、core/runtime、共有契約、`docs/progress.md`は変更しない。

## 作業ルートと実効権限

`pwd`は`/Users/rio/workspace/src/AnotherBall/rust-mobile-game-lab/.worktrees/renderer`。
現在のセッション`01a16982-8895-76e0-9b1a-301bc7904f54`の`thread_settings_applied`で、同じcwd、`approval_policy=on-request`、`approvals_reviewer=auto_review`を確認した。
実効profileは`managed/restricted`で、作業ルート・一時領域へのwrite、ルートへのread、network restricted。これは今回要求されたworkspace-writeの範囲と一致する。
設定ファイルだけで判定していない。該当記録を実装前に`artifacts/worker-permissions.txt`へ保存した。GUIでのPermissions表示は未確認。
GPU起動は正規の`require_escalated`を利用し、新規作成した自分のウィンドウだけを操作・撮影した。

## APIと責務

```rust
Renderer::new(&wgpu::Instance, wgpu::Surface<'static>, width: u32, height: u32)
    // async -> Result<Renderer, String>
renderer.resize(width, height);
renderer.render(&game_core::Scene) // -> Result<(), String>
renderer.presented_frames()       // -> u64
```

`Scene.size`を論理ピクセルの表示領域として、矩形を2三角形、円を64分割の三角形へCPUで分割する。
WGSLシェーダーで頂点色を出力し、Scene順のアルファ合成を行う。SceneやProbeの状態、ゲームの規則は変更しない。
GPU頂点バッファは容量が足りる限り再利用する。
非有限値、不正なSceneサイズ、負の幅・高さ・半径、表現範囲を超える座標はエラー。幅・高さ・半径0の図形は描かない。有限なRGBAは0〜1へ制限する。

Surfaceの幅・高さは物理ピクセル。幅または高さ0ではconfigure/presentを行わない。resize後は次のrenderで再構成する。
デバイスのテクスチャ寸法・バッファ上限を超える要求はエラーを返す。
`presented_frames`は実際にqueueへ提出・presentした回数で、描画を飛ばした呼び出しは数えない。画面の走査完了や性能値を示すカウンターではない。

wgpu 30.0.1ではSurface取得が`CurrentSurfaceTexture`を返す。`Outdated`は再構成して1回だけ再試行し、継続する場合はそのフレームを飛ばす。
`Suboptimal`は取得したフレームの提示後、次回に再構成する。`Timeout/Occluded`は飛ばす。`Validation`はエラー。
`Lost`またはdevice lostでは`Err(game_renderer::SURFACE_LOST.into())`を返す。
**統合担当はこのエラーでRendererを破棄し、wgpu Surfaceを新規作成してRendererを再初期化すること。** configureだけではLostの回復契約を満たさない。
同じRendererへのresize/render/破棄はホストのGPU所有スレッドで直列化する。ネイティブ参照はSurfaceを破棄するまで保持する。

APIは固定版のローカル公式ソース`wgpu-30.0.1/src/api/{surface,surface_texture,render_pipeline,device,queue}.rs`と`winit-0.30.13/src/application.rs`で確認した。
公式資料: [wgpu描画pipeline](https://docs.rs/wgpu/30.0.1/wgpu/struct.RenderPipelineDescriptor.html)、[Surface取得と回復条件](https://docs.rs/wgpu/30.0.1/wgpu/enum.CurrentSurfaceTexture.html)、[winitイベント処理](https://docs.rs/winit/0.30.13/winit/)。

Desktopはwinitの`ApplicationHandler`を使用し、イベントスレッドで`Arc<Window>`からsafeなSurfaceを作成する。
GPU初期化は別スレッドで行い、世代が一致する結果だけを採用する。前の初期化結果を破棄してから次を開始する。
マウス左クリック・タッチ開始の物理座標を密度で変換して`Probe::touch`へ渡し、選択された領域を可視化する。
リサイズ・密度変更を反映し、Spaceで一時停止、Escape/閉じるで終了する。
一時停止・非表示・occluded・suspended・ゼロサイズ時は描画要求を止める。suspended時はRendererを破棄し、resumed時に再初期化する。

## 再現コマンド

作業ルートで実行する。Rustは固定の1.99.0を使用する。

```sh
scripts/cargo.sh test --locked -p game-renderer -p game-core
scripts/cargo.sh fmt --package game-renderer --package game-desktop -- --check
scripts/cargo.sh clippy --locked -p game-renderer -p game-desktop --all-targets -- -D warnings
scripts/cargo.sh build --locked -p game-desktop
scripts/cargo.sh run --locked -p game-desktop -- --smoke
```

通常操作は最後のコマンドから`-- --smoke`を外す。
今回の環境ではClippyが固定toolchainに未導入で、スクリプト経由でもHomebrew版が選ばれて失敗した。
正規の`rustup component add --toolchain 1.99.0 clippy`で追加後、同じスクリプト経由のClippyが成功した。スクリプトや共有設定は変更していない。

`--smoke`は20秒以内に実描画、合成入力、OSウィンドウのリサイズ、200msの一時停止、200msの非表示、再表示、Rendererのゼロサイズと復帰を検査して終了する。
成功時は`SMOKE_OK`、提示回数、入力・リサイズ・停止検査の結果を出力し、終了コード0。入力は実マウス操作ではなく、同じ座標処理経路への合成入力である。

## 検証済みと証拠

- 描画器4テスト成功: 論理座標の端、長方形画面上の円の半径・閉じ目、退化・不正形状、浮動小数点の上限。coreの既存3テストも成功。
- Desktopビルド、担当2パッケージのformat/Clippy、`git diff --check`成功。
- macOS Desktop GPU実行成功。最初のsmokeは122フレーム、撮影時は121フレーム、最終差分は122フレームを提示し、すべて正常終了した。
- 1440×960（密度2）から1536×1008へのリサイズ、入力1件・選択2、一時停止/非表示中の提示増分0、ゼロサイズの提示なし、復帰後の描画を確認した。
- `artifacts/renderer-desktop.png`を目視確認し、3つの円と矩形、右領域の選択色を確認した。新規プロセスのPIDに一致するウィンドウだけを撮影した。
- ログ: `artifacts/renderer-tests.log`、`renderer-build.log`、`renderer-clippy.log`、`renderer-smoke.log`、`renderer-smoke-capture.log`、`renderer-smoke-final.log`。
- コメント整理はリポジトリ固有スキルが無いため`avvy-development:clean-slop`を適用。今回のコメントはいずれも寿命・数値・検証上の制約で、削除対象なし。

artifactsはgitignore対象のローカル証拠であり、コミットには含めない。別worktreeで統合する場合は本worktreeの証拠を参照またはコピーする。

## 未確認・統合時の残作業

Android/iOSのビルド・Emulator/Simulator・実機・性能はこのワーカーでは未確認。Desktopの証拠を代用しない。
実マウス・実タッチの操作、OSのsuspended/resumed、実際のSurface Lost/device lost/Timeout/Suboptimal発生時の回復は未確認。
通常リサイズとゼロサイズからの再構成は実行済みだが、失効イベントを発生させた証拠とは区別する。
native runtimeは統合担当が別途実装中で、本作業ではシンボルを隠すstubを追加していない。
共有描画契約を変更せず、上記Surface再作成経路をnative runtimeへ接続してモバイルで確認する必要がある。
push・PR・merge・Issueの終了は行わない。
