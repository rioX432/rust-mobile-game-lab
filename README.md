# Rust Mobile Game Lab

Android・iOSのネイティブ画面内へ、共通のRust 2D描画・ゲーム・通信基盤を埋め込む技術PoC。
同じRustゲーム処理・描画器で、Android EmulatorとiOS Simulatorのオフラインレースを確認済み。開始→タッチ/ネイティブ入力→締切→結果→次ラウンド、開閉10回・復帰5回・再生成後入力が動く。今回の区切りはオフラインまで。通信・投票・SDK最終取り込み・実機・性能は未完了で、PoC全体は未完了。

- [仕様全文](prompts/codex-poc.md)（公開は後続の依頼で承認済み）
- [計画とIssue対応](docs/plan.md)
- [受入条件](docs/acceptance.md)
- [環境とPermissions](docs/environment.md)
- [現在地と再開手順](docs/progress.md)
- [オフラインレースの操作・実行証拠](docs/offline-native.md)
- [初期の描画・寿命検証](docs/native-verification.md)

リポジトリルートで実行する。

```sh
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh run -p game-core --example offline_race --locked
scripts/cargo.sh run -p game-desktop -- --smoke
scripts/android-build.sh
scripts/ios-build.sh simulator
```

[Android](docs/android.md)と[iOS](docs/ios.md)に必要なSDK・成果物・起動方法、[Desktop](docs/renderer.md)に操作方法を記録する。サーバー起動とSDK生成は後続実装。

既存のpersona系リポジトリを変更せず、依存しない。
初期段階は図形・タッチ・Surface再生成の縦断確認を両OSで行い、その後にゲーム、通信の順で接続する。

モバイルアプリを起動し、Open → Race → Start / next → 3秒以内にレーンをタッチ（またはChoose）で結果まで進む。もう一度Start / nextで次ラウンド。オフラインでは同じインスタンス内のhost1/viewer2を使い、別端末間通信は行わない。Desktopは描画プローブ用シェル。
