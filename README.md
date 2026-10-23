# Rust Mobile Game Lab

Android・iOSのネイティブ画面内へ、共通のRust 2D描画・ゲーム・通信基盤を埋め込む技術PoC。
現時点では準備中。実装・モバイル起動・実機測定は未確認で、PoCは未完了。

- [仕様全文](prompts/codex-poc.md)（公開は後続の依頼で承認済み）
- [計画とIssue対応](docs/plan.md)
- [受入条件](docs/acceptance.md)
- [環境とPermissions](docs/environment.md)
- [現在地と再開手順](docs/progress.md)

既存のpersona系リポジトリを変更せず、依存しない。
初期段階は図形・タッチ・Surface再生成の縦断確認を両OSで行い、その後にゲーム、通信の順で接続する。
