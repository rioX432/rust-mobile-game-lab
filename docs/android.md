# Android | 現行起動・検証ガイド (SEED Race v1)
更新: 2026-10-24。これは現行の**2D/3択オフライン技術baseline**を動かす手順。新しいSEED 3D物理/オンラインゲームはG03/G07/G11で実装予定で、今は動かない。

## ビルド
ホストはJDK17、Android SDK platform35、NDK 28.2.13676358、Rustのaarch64-linux-androidを使用する。SDKやRustの固定版は `docs/environment.md` に記録。

```sh
scripts/android-build.sh --host-only # Kotlin hostの単体テストとlint。Rust/JNIリンクはしない
scripts/android-build.sh             # JNI shared libraryとDebug APKを作る
```

生成APK: `apps/android/app/build/outputs/apk/debug/app-debug.apk`。任意の専用Emulatorへインストール・起動し、Open → Race → Start / next → Selecting中にタッチまたはChoose → Result → 次のroundを確認する。未選択で3秒経つと選択不可となる。現状は1プロセス内のHost/Viewer固定で、WebSocket越しには参加しない。

## 検証の範囲
API35 arm64 Emulatorで3択のゲーム一巡、画面開閉10回、Resultからの背景復帰5回、Resizeと再生成後タッチを確認した。実機、動的画面密度変更、全phase背景復帰、Release相当の性能は未確認。結果は[証拠](offline-native.md)を参照。残件は[G02 #18](https://github.com/rioX432/rust-mobile-game-lab/issues/18)で追跡する。

新ゲームの描画/操作/通信を同じHostに入れる際も、既存FFIの世代付き所有権とpause/detach/destroyの安全条件は保持する。アプリ起動やE2E検証は専用端末のみ行い、ほかの稼働中の端末は操作しない。
