# Rust Mobile Game Lab

Android・iOSのネイティブ画面内へ、共通のRust 2D描画・ゲーム・通信基盤を埋め込む技術PoC。
同じRust描画器をAndroid・iOSのネイティブUI内で起動し、入力・再生成をEmulator/Simulatorで確認済み。ゲーム・通信・SDK消費側・実機・性能の受入は未完了で、PoC全体は未完了。

- [仕様全文](prompts/codex-poc.md)（公開は後続の依頼で承認済み）
- [計画とIssue対応](docs/plan.md)
- [受入条件](docs/acceptance.md)
- [環境とPermissions](docs/environment.md)
- [現在地と再開手順](docs/progress.md)
- [両OSの実行条件と証拠](docs/native-verification.md)

リポジトリルートで実行する。

```sh
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh run -p game-desktop -- --smoke
scripts/android-build.sh
scripts/ios-build.sh simulator
```

[Android](docs/android.md)と[iOS](docs/ios.md)に必要なSDK・成果物・起動方法、[Desktop](docs/renderer.md)に操作方法を記録する。サーバー起動とSDK生成は後続実装。

既存のpersona系リポジトリを変更せず、依存しない。
初期段階は図形・タッチ・Surface再生成の縦断確認を両OSで行い、その後にゲーム、通信の順で接続する。
