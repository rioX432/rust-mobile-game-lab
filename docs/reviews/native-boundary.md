# FFI・Surface寿命・両OS部分埋め込みの重点レビュー

2026-10-24。結論: **修正が必要な重大問題を1件（P1）確認した。** iOSの描画バッファ取得待ちがUI側の停止・破棄をブロックする経路がある。コードと固定依存の公式ソースで確認した指摘であり、端末でのハング再現は未確認。

## 対象と作業境界

- 対象Issue: [FFI・寿命 #5](https://github.com/rioX432/rust-mobile-game-lab/issues/5)、[Android #6](https://github.com/rioX432/rust-mobile-game-lab/issues/6)、[iOS #7](https://github.com/rioX432/rust-mobile-game-lab/issues/7)。各本文の受入条件を取得して確認した。
- 作業ルート: `/Users/rio/workspace/src/AnotherBall/rust-mobile-game-lab/.worktrees/review-native`。
- レビューブランチ: `review/native-embedding`。開始HEADと対象実装HEADはともに `841b9ffc5da6e5396a4b4a1413d4ee3be6b77bd0`。
- 比較: `git diff 51a4931f1459cc65b33bac1fa8b2a5b05a3f52bf...841b9ffc5da6e5396a4b4a1413d4ee3be6b77bd0`。前者は確認時の `main` と `feat/native-embedding` のmerge-base。対象を固定し、親の後続変更は含めない。
- 親AGENTS.md、当該AGENTS.md、`prompts/codex-poc.md`、`docs/contracts.md`、`include/game_runtime.h`、`docs/progress.md`を読了。
- 実装・共有契約・Cargo.lockは変更しない。追跡ファイルの編集は本報告書のみ。指定された権限記録はGit対象外の `artifacts/worker-permissions.txt` に保存した。
- セッション記録の `session_meta` と `turn_context`、実際の `pwd` から専用cwd、`originator=codex_exec`、CLI 0.164.0、`approval_policy=on-request`、`approvals_reviewer=auto_review`、`sandbox_policy.type=workspace-write`を確認。実効profileはmanaged/restrictedで専用Worktreeと一時領域への書込みを許可し、ネットワークは制限されている。設定ファイルだけによる判定ではない。GUIのPermissions表示は未確認。

## 指摘

### P1: Metalの無期限バッファ取得待ちを、UIの停止・破棄と共有するロック内で行う

**主箇所:** `crates/runtime/src/lib.rs:562-577`（ロック解放は600行）。関連: 同ファイル `163-166`、`275-280`、`320-336`、`crates/renderer/src/lib.rs:159-165`、`apps/ios/Sources/RuntimeSession.swift:94-121`。

**コードで確認した経路:**

1. Rustワーカーは `control.gate` を保持したまま `Renderer::render()` を呼ぶ。
2. `render()` は `Surface::get_current_texture()` へ入る。固定した `wgpu-hal 30.0.1` のMetal実装は `configure()` で `setAllowsNextDrawableTimeout(false)` を設定し、`acquire_texture()` では渡されたtimeoutを使用せず `nextDrawable()` を呼ぶ（公式ソース334、349、373-375行）。
3. Appleの仕様では、この設定でdrawable（描画先バッファ）が空くまでの待機は無期限になる。
4. UIスレッドからの `mg_pause()`、`mg_detach()`、`mg_destroy()` も同じgateを取得する。iOSは非表示・非アクティブ化時に `setActive(false)` → `reconcile()` → `mg_detach()` を同期呼出しするため、ワーカーのバッファ待ちをUIまで引き継ぐ。

**再現条件:** iOS/Metalで描画バッファがすべて使用中となり、ワーカーが次の `nextDrawable()` で待機している間に、画面を閉じる・非アクティブ化する・pause/detach/destroyを呼ぶ。GPU処理が遅れてバッファが戻ればその間UIが停止し、戻らなければ停止解除に上限がない。通常の背景復帰が必ずハングするとは主張しない。

**影響:** 初期化自体をワーカーへ移していても、停止・破棄時にUI応答を失い得る。停止フラグの更新とSurfaceの後始末も進まない。既存の正常系スモークや「初期化中にinvalidateできる」単体テストでは、この描画中の待機を検査していない。

**推奨修正:** 描画をバッファ取得・準備とsubmit/presentに分け、バッファ取得やGPU待ちをライフサイクル用gateの外で行う。submit/present直前にgate内で世代・paused・destroyedを再検査し、無効なら取得済みフレームを破棄する。Surfaceとネイティブ参照の解放は引き続きワーカーが所有し、参照解放順とdetach後の新規描画禁止を維持する。gateを外すだけで世代再検査を省く修正は不可。Metalの取得待ち自体も停止可能または上限付きにし、未完了ワーカーが残り続けないようにする。

**修正後の最小検証:** フレーム取得を意図的に保留し、その間のpause/detach/destroyがUIを待たせず、取得が後から完了しても旧世代のsubmit/presentが起きないことを確認する。その後、専用iOS環境で描画と背景復帰を確認する。本レビューではコード変更・故障注入・端末操作をしていないため、この条件での実行再現は未確認。

**一次資料:** ローカル依存の `.cargo_vcs_info.json` が示すコミットと[wgpu-halの公式ソース](https://github.com/gfx-rs/wgpu/blob/40f4a34ebaf56f9a046231f54125ad046239d3f3/wgpu-hal/src/metal/surface.rs#L329-L375)を照合した。待機仕様は[Apple `allowsNextDrawableTimeout`](https://developer.apple.com/documentation/quartzcore/cametallayer/allowsnextdrawabletimeout)で確認した。

## そのほかの確認結果

以下はソース上の確認であり、実機・負荷条件の成功を意味しない。上の1件以外に、このレビュー範囲で確定できた重大問題はない。

| 観点 | 確認した実装 |
|---|---|
| 不透明ハンドル・無効ID・二重破棄 | `crates/runtime/src/registry.rs:17-53`。スロットと32bit世代をIDに符号化し、世代枯渇スロットを再使用しない。`mg_destroy`はレジストリから除去し、解放済みポインタを検査しない。 |
| panic境界 | `crates/runtime/src/lib.rs:155-156,228-272,433-450`、`android.rs:121-127`。C ABI/JNIはunwindを捕捉し、ワーカーにも捕捉境界がある。release設定は `panic="unwind"`。abort・メモリ不足によるプロセス終了・不正な外部ポインタまでは回復できない。 |
| 同時呼出しと上限 | レジストリ、gate、statusをMutexで保護。コマンドは64件、インスタンスと未終了ワーカーは各16件に制限。描画中にgateを保持する問題は上で指摘した。 |
| GPU初期化中のdetach/destroy | `crates/runtime/src/lib.rs:493-511`。初期化はgate外のワーカーで行い、完了後に世代とdestroyedを再検査して古い結果を採用しない。 |
| 描画停止と解放順 | `crates/runtime/src/lib.rs:114-130,163-176,570-605`。invalidateは世代更新とactive解除を行い、SessionはRenderer→NativeTargetの順に解放する。取得待ち中の停止に上の問題がある。 |
| ネイティブ参照 | `native.rs:56-90,124-136`、`android.rs:40-55`。Androidは取得したANativeWindowを保持し、AppleはCAMetalLayerをretainする。Surfaceより後に対応参照を解放する。 |
| 遅延した画面コールバック | Androidは `MainActivity.kt:67-83,95-103,115-124` のsession/view同一性と世代、iOSは `ViewControllers.swift:53-55,120-126,140-148,172-178` のsession/view同一性とコールバック解除で旧画面を除外する。 |
| pause・復帰 | gateのpausedで描画を抑止。iOSは非アクティブ時にdetachし、復帰時に同じレイヤーを再接続する。現段階は静止図形で時間更新をしておらず、ゲーム接続後の復帰deltaは未確認。 |
| ゼロサイズ | `Renderer::configure/render` はゼロサイズを描画しない。Androidはresize、iOSはdetachと有効サイズ復帰後のattachで扱う。 |
| Surface Lost | `crates/renderer/src/lib.rs:149-184`、`runtime/src/lib.rs:585-595`。Outdatedは再構成、Suboptimalは次フレームで再構成、Timeout/Occludedはスキップ、Lost/device lostは最大3回再生成。実際のLostを発生させた復旧は未確認。 |
| ネイティブUIとRust入力 | 両OSともゲームViewだけからローカル物理ピクセル座標を渡す。上下UIのResetは別のaction APIを使い、ゲーム規則をホストへ複製していない。 |

Surfaceの生ハンドル寿命は[wgpu 30.0.1の契約](https://docs.rs/wgpu/30.0.1/wgpu/enum.SurfaceTargetUnsafe.html#variant.RawHandle)、panicの制限は[Rust公式 `catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)と照合した。`crates/renderer/src/lib.rs:48-53` の `downlevel_defaults().using_resolution(adapter.limits())` も確認した。iOS上限15と既定要求16の差については親からの申告であり、実機上限を本レビューで独立測定したものではない。

## 既存証拠と未確認の区分

証拠ディレクトリは `/Users/rio/workspace/src/AnotherBall/rust-mobile-game-lab/artifacts/`。既存ログは親が実行した結果で、本レビューによる再実行ではない。ログだけでは対象コミットとの完全な対応を独立保証できない。

| 層 | 確認した証拠と限界 |
|---|---|
| Rustテスト・lint | `native-tests.log`、`workspace-tests.log`、`clippy.log`。core 3件、renderer 4件、runtime 7件の成功とstrict Clippy成功を確認。初回ログのdeprecated警告に対応する箇所は、対象ソースでは `try_update` に置換済み。GPU待ち中の停止検証は含まれていない。 |
| Androidビルド | `android-integrated-build.log` の単体テスト・lintタスクとAPK生成成功を確認。ホスト単体テストは3件のテストコードも読んだ。JNI/GPU実行の代替にはしない。 |
| iOSビルド | `ios-integrated-build.log` のSimulatorアプリのリンク・BUILD SUCCEEDEDを確認。実機ビルド・署名・実機起動の証拠ではない。 |
| 両OSの図形・部分埋め込み | `ios-native-probe.png` と `android-native-probe.png` を実際に表示して確認。同じ3レーン・円と上下ネイティブUIがあり、診断値はRunning、live_instances/live_workers/retained_targetsが各1。静止画像だけで入力や停止を判定していない。 |
| 入力・Reset・復帰 | `native-smoke-initial.json` は両OSで右レーンtouch後にtouches=1/selected_lane=2、native_reset後にselected_lane=null、最初の背景・resize・再生成後にRunning/last_pause_frame_delta=0/last_error=null。iOS New layerの操作は `ios-runtime.stderr.log` のdetach/attach経路と合わせて確認。 |
| 参照解放順 | `ios-runtime.stderr.log` と `android-runtime.log` で、同世代のsurface_dropped→native_reference_releasedの順を確認。ホストのdetach受理ログだけを解放完了とは扱わない。 |

**未確認:** 親が追加検証中の開閉10回・背景復帰5回の完了、初期化中detach/destroyと遅延結果のモバイル実行、実Surface Lost/device lost、今回のP1条件でのハング再現、実機動作・実機性能、ゲーム/通信、SDK消費側の最終受入。既存端末は操作していない。ディスク制約に従い追加GPUビルド・広範なテスト再実行もしていない。

重点レビューの判定は **P1修正後に影響範囲を再確認**。Issue #5/#6/#7の受入完了やPoC完了を意味しない。本報告書だけを承認済みのローカルコミット対象とし、push・PR・merge・Issue closeは行わない。
