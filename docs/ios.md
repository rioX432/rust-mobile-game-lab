# iOS 部分埋め込み（独立ワーカー）

担当: [iOSホスト Issue #7](https://github.com/rioX432/rust-mobile-game-lab/issues/7)、[環境・契約 Issue #2](https://github.com/rioX432/rust-mobile-game-lab/issues/2)。基点は `feat/native-embedding` の `5fef344`、作業ブランチは `feat/ios-native`。Issueの受入条件は未完了。

## 実装範囲

- `apps/ios/Sources`: UIKitの上下のネイティブUI、中央の`UIView`/`CAMetalLayer`。Bridging Headerから正本`include/game_runtime.h`を直接取り込む。図形プローブのみでレース・通信は未接続。
- `Open drawing probe`で新しいRustハンドルを作成。`Reset`は`mg_action(handle, 0, 0)`。`Resize`は表示領域の高さを切替。`New layer`は同じハンドルのレイヤーを再生成。`Close`とナビゲーションの戻る操作はdetach→destroy。
- 状態JSONを下部のスクロール可能なネイティブ欄に表示。`phase / frames / touches / selected_lane / live_instances / surface_generation / last_error`の内容はRustから取得し、Swiftでゲーム状態を生成しない。
- `project.yml`: XcodeGen、最低iOS 16、arm64実機/Simulator、`libgame_runtime.a`リンク。署名を無効化し、DeveloperアカウントやTeam設定は変更しない。

## 所有権とライフサイクル

すべてのホスト側C ABI呼び出しはメインスレッド。GPU初期化・描画・解放はRustワーカーに任せる。Swiftは`CAMetalLayer`の非所有ポインタを渡し、契約どおりRustが呼び出し中にretainすることに依存する。RustはSurface破棄後にその参照を解放する。Swiftの`attachedLayer`はdetach受理まで保持する。

`sceneWillResignActive`/背景移行でdetach・pauseし、再びactiveになれば現在のレイヤーをattach・resumeする。画面が非表示の場合は再開しない。Scene切断ではハンドルも閉じる。初期化完了待ちやフレームの更新をSwiftで行わないため、背景時間をdelta timeへ加算しない。

レイアウト・window・trait変更で物理ピクセルサイズとscreen scaleを再計算する。ゼロサイズではdetachし、有効サイズへ戻れば再attach。タッチはゲーム領域内の開始点だけを物理ピクセル座標へ変換し、現在のレイヤーとサイズの接続が受理されている場合だけ送る。

画面ごとのsession UUID、現在のsession/viewの同一性確認、閉鎖時のコールバック解除で古い画面・レイヤーからの入力や状態更新を無効化する。接続・リサイズ・pause・detach・destroyが失敗した場合は0.1秒周期で再試行し、UIを同期的に待たせない。touch/resetは再送せずエラーを表示する。close中のタイマーはdetach→destroy受理までsessionを保持し、その後に解除する。`[iOS runtime UUID]`ログにhandle、attach、resize、detach、destroy受理を残す。受理ログはRustの非同期GPU解放完了の証拠ではない。

## ビルドと統合

リポジトリルートで実行する。XcodeGen 2.45.4、Xcode 26.6（17F113）、iPhoneOS/iPhoneSimulator SDK 26.5を確認した。既定ではSDK 26.5を検査する。別のSDKを使う場合は`IOS_SDK_VERSION`へ期待版を明示する。CIではランナーの選択中SDK版を明示し、スクリプトが実際の版とtargetをログへ出す。Rustは`scripts/cargo.sh`経由の固定1.99.0、`--locked`を使用する。

```sh
# Swiftソースを実SDKでオブジェクトへコンパイル。リンク・起動の証拠にはしない。
scripts/ios-build.sh check-sources
scripts/ios-build.sh check-device-sources

# Rust静的ライブラリ生成→必要C ABIシンボル検査→XcodeGen→署名なしアプリビルド
scripts/ios-build.sh simulator
scripts/ios-build.sh device
```

Rustターゲットは`aarch64-apple-ios-sim`と`aarch64-apple-ios`。この環境では導入済み。ライブラリは`target/<target>/debug/libgame_runtime.a`から`apps/ios/build/rust/<SDK>/`へコピーする。必要な9個の`mg_*`定義がなければ明示的に失敗し、アプリのリンクを行わない。Rust 1.99のLLVM 23 bitcodeをXcodeの古い`nm`が読めないため、`nm --no-llvm-bc`でネイティブシンボルを調べる。

Rust統合後に上記ビルドを再実行する。アプリは`apps/ios/build/DerivedData-<SDK>/Build/Products/Debug-<SDK>/RustMobileLab.app`。生成Xcodeプロジェクトとビルド成果物はGit対象外。スクリプトにはinstall/launch処理がない。署名や専用端末での実行は統合担当が別途扱う。

## 証拠と残作業（2026-10-24 JST）

権限は現在のsession `01a16982-8894-79a1-9a27-edc7c6a8f102`の`thread_settings_applied`と`pwd`で確認した。cwdはこの`.worktrees/ios`、`on-request / auto_review`、managed restricted workspace profile（当該Worktree書込可、ルート読取、ネットワーク制限）である。設定ファイルだけの確認ではない。GUIは未確認。完全な抽出記録はローカル`artifacts/worker-permissions.txt`（Git対象外）。

| 確認対象 | 結果・証拠 |
| --- | --- |
| Swift + 正本Cヘッダー | Simulator/実機arm64のオブジェクト生成成功。`artifacts/ios-source-check.log`、`artifacts/ios-device-source-check.log` |
| XcodeGen / plist / shell構文 | プロジェクト生成、`plutil -lint`、`bash -n`成功 |
| Simulator Rustアーカイブ | 生成成功。`mg_create`未定義でスクリプトは終了1。`artifacts/ios-simulator-build.log` |
| 実機Rustアーカイブ | 生成成功。`mg_create`未定義でスクリプトは終了1。`artifacts/ios-device-build.log` |
| アプリリンク・起動 | 未確認。共有Rust実装の統合待ち。偽stubは作成していない |
| Simulator / 実機の操作 | 未実施（このワーカーの担当外）。既存端末へinstallしていない |
| 実機性能・サイズ | 未測定 |

統合担当は専用端末で次を実行し、画面画像・Rust status JSON・両側の解放ログを残す。コンパイル成功を動作確認へ読み替えない。

1. 上下のネイティブUIとRust図形が同時に表示され、領域内タッチで`touches`が増える。Resetや画面外操作でRustタッチが増えない。
2. Closeと戻る（スワイプの確定・キャンセルを含む）で開閉10回。次のOpenで`live_instances`が増え続けず、閉じた画面の描画・コールバックが残らない。
3. 背景復帰5回。非active中に描画が停止し、復帰後に新しい接続世代で描画・入力が戻る。初期化直後のClose/背景化も含む。
4. Resize/New layer/回転/利用可能ならiPadのウィンドウサイズとscale変更。ゼロサイズからの復帰も含め、古い座標・レイヤーの入力が新しい接続へ混ざらない。
5. RustのSurface破棄→retain参照解放をRust側ログで確認する。Swiftのdetach受理だけで解放順を確認済みにしない。

## 確認した一次資料

- [Apple CAMetalLayer](https://developer.apple.com/documentation/quartzcore/cametallayer): `UIView.layerClass`によるMetalレイヤー。
- [Apple UISceneDelegate](https://developer.apple.com/documentation/uikit/uiscenedelegate): active/非active/背景/Scene切断の通知。公式Markdown本文も取得して確認。
- [Apple UIView.layoutSubviews](https://developer.apple.com/documentation/uikit/uiview/layoutsubviews()): レイアウト更新時のサイズ橋渡し。
- [Apple UIViewController.didMove(toParent:)](https://developer.apple.com/documentation/uikit/uiviewcontroller/didmove(toparent:)): 親からの取り外し完了通知で破棄し、戻るスワイプのキャンセル時はsessionを維持する。
- [wgpu 30.0.1 SurfaceTargetUnsafe](https://docs.rs/wgpu/30.0.1/wgpu/enum.SurfaceTargetUnsafe.html): `CoreAnimationLayer`の有効オブジェクト契約。固定依存ソース`wgpu-30.0.1/src/api/surface.rs`、`wgpu-hal-30.0.1/src/metal/surface.rs`のretain処理も読んだ。
- [XcodeGen ProjectSpec](https://github.com/yonaskolb/XcodeGen/blob/master/Docs/ProjectSpec.md): project/target/settings/schemeの形式。生成を実行して確認。
