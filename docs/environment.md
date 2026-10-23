# 環境確認（2026-10-24 JST）

## 確認済み

- macOS 26.7.1 / arm64。Rust 1.96.0 / Cargo 1.96.0（Homebrew）。rustupの既定はstable-aarch64-apple-darwin。使用する固定ツールチェーンは公式互換性確認後に選定する。
- 導入済みターゲット: aarch64-apple-darwin、aarch64-apple-ios、aarch64-apple-ios-sim、aarch64-linux-android、wasm32-unknown-unknown。
- GitHub CLI 2.95.0。ネットワーク制限外でrioX432への認証・対象リポジトリADMIN権限を確認。認証情報は保存しない。
- 作者はrioX432にGitHub上で紐付く既存コミットから確認し、リポジトリローカルに設定。ユーザー全体のGit設定は変更しない。
- Java実行時はOpenJDK 21.0.11。java_homeが検出するJDKは17.0.19。Android Studio同梱JDKも存在する。ビルドではJAVA_HOMEを明示する。
- Android SDK: API 31、33、34、35、36、36.1、37.0を導入済み。NDK 25.1、27.0、27.1、28.2、30.0を導入済み。使用版は互換性確認後に固定する。
- Gradle、xcodegenを導入済み。Gradle版とAndroid Gradle Pluginの組み合わせは未選定。
- Xcode 26.6 (17F113)、iOS Simulator SDK 26.5。Simulatorランタイムは18.6、26.5、27.0が存在。
- サンドボックス外の読み取りで稼働中のiOS Simulator 1台、Android Emulator 3台を確認。既存の稼働端末を無断で変更せず、専用環境を選択・作成する。
- Android実機は接続なし。iOS実機は1台がpaired/availableだが、今回の使用対象としての確認と利用は未実施。他者の端末情報を記録しない。

サンドボックス内ではsimctl/ADBのサービス接続が拒否された。正規の権限昇格で読み取り確認が成功し、SDKや端末がないこととは区別できた。

## Permissions

- Codex CLI 0.164.0の`--help`と`resume --help`で`--cd`、`--approve-for-me`に対応することを確認。
- プロジェクトの`.codex/config.toml`に`workspace-write`、`on-request`、`auto_review`を保存。
- 最終確認の実効セッション記録では`on-request/auto_review`とworkspace profileを確認した。初回確認時のreviewerはuserだったが、最終確認時にはauto_reviewへ変わっている。切替操作そのものはこのエージェントでは実施していない。
- **専用作業ルートへの反映は未確認**。現セッションのcwdは親フォルダAnotherBallのまま。プロジェクト設定の保存を専用ルートでの実効確認として扱わない。
- CodexアプリのComputer Useは安全制約で禁止されており、GUI選択の確認は未確認。ユーザー全体の設定、信頼データ、セッション内部DBは変更していない。
- 正式なCLI再開後、作業ルート・`/permissions`と`/status`・新しい実効セッション記録を確認する。信頼確認が出た場合は正規の画面で確認する。Worktreeでも同じ確認が必要。
- CLIの`--approve-for-me`はworkspace-writeを保った自動レビュー選択。Full accessやneverへ置換しない。

公式資料を2026-10-24に確認:

- https://learn.chatgpt.com/docs/permission-modes
- https://learn.chatgpt.com/docs/sandboxing/auto-review
- https://learn.chatgpt.com/docs/config-file/config-basic

プロジェクト設定は信頼されたプロジェクトのみで読み込まれ、CLI指定が優先する。設定と現セッションでの実効確認は別工程。
