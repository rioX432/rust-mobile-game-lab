# 環境確認（2026-10-24 JST）

## 確認済み

- macOS 26.7.1 / arm64。Rust 1.96.0 / Cargo 1.96.0（Homebrew）。rustupの既定はstable-aarch64-apple-darwin。ビルド固定ツールチェーンはRust 1.99.0で、scripts/cargo.shがHomebrewより優先する。
- 導入済みターゲット: aarch64-apple-darwin、aarch64-apple-ios、aarch64-apple-ios-sim、aarch64-linux-android、wasm32-unknown-unknown。
- GitHub CLI 2.95.0。ネットワーク制限外でrioX432への認証・対象リポジトリADMIN権限を確認。認証情報は保存しない。
- 作者はrioX432にGitHub上で紐付く既存コミットから確認し、リポジトリローカルに設定。ユーザー全体のGit設定は変更しない。
- Java実行時はOpenJDK 21.0.11。java_homeが検出するJDKは17.0.19。Android Studio同梱JDKも存在する。ビルドではJAVA_HOMEを明示する。
- Android SDK: API 31、33、34、35、36、36.1、37.0を導入済み。NDK 25.1、27.0、27.1、28.2、30.0を導入済み。使用版はSDK35 / NDK28.2.13676358に固定した。
- Gradle、xcodegenを導入済み。Gradle8.11.1 / AGP8.9.2 / Kotlin2.2.10を使用し、JDK17でビルド成功。
- Xcode 26.6 (17F113)、iOS Simulator SDK 26.5。Simulatorランタイムは18.6、26.5、27.0が存在。
- サンドボックス外の読み取りで稼働中のiOS Simulator 1台、Android Emulator 3台を確認。既存の稼働端末を無断で変更せず、専用環境を選択・作成する。
- Android実機は接続なし。iOS実機は1台がpaired/availableだが、今回の使用対象としての確認と利用は未実施。他者の端末情報を記録しない。

サンドボックス内ではsimctl/ADBのサービス接続が拒否された。正規の権限昇格で読み取り確認が成功し、SDKや端末がないこととは区別できた。

## Permissions

- Codex CLI 0.164.0の`--help`と`resume --help`で`--cd`、`--approve-for-me`に対応することを確認。
- プロジェクトの`.codex/config.toml`に`workspace-write`、`on-request`、`auto_review`を保存。
- 再開後の実効セッション記録でcwdが専用リポジトリ、on-request/auto_review、workspace profileを確認。今回のdeveloper指示もworkspace-write/auto_reviewで、書込み範囲は専用フォルダ。設定保存と実効確認の両方を満たした。
- CLIで確認したためGUIの表示確認は未確認。GUI操作は禁止されている。
- CodexアプリのComputer Useは安全制約で禁止されており、GUI選択の確認は未確認。ユーザー全体の設定、信頼データ、セッション内部DBは変更していない。
- 再開後の専用cwdと実効セッション記録を確認済み。独立レビューWorktreeもturn_contextのon-request/auto_review/workspace-writeを統合担当が直接確認した。
- CLIの`--approve-for-me`はworkspace-writeを保った自動レビュー選択。Full accessやneverへ置換しない。

公式資料を2026-10-24に確認:

- https://learn.chatgpt.com/docs/permission-modes
- https://learn.chatgpt.com/docs/sandboxing/auto-review
- https://learn.chatgpt.com/docs/config-file/config-basic

プロジェクト設定は信頼されたプロジェクトのみで読み込まれ、CLI指定が優先する。設定と現セッションでの実効確認は別工程。

ビルド固定版はRust 1.99.0。Homebrewのrustc 1.96が先に解決されるためscripts/cargo.shがrustupの1.99.0 binをPATH先頭にする。最小コアテスト3件とハンドルレジストリテスト3件を実行済み。

## 専用の検証環境

Android AVD RustMobileGameLab_API35_20261024（emulator-5630、API35 arm64、Pixel5定義）を作成し、既定data領域の容量不足を-partition-size 2048で解消した。iPhone17 / iOS26.5の専用Simulator 06E9B566-FCC7-4A9B-9A0D-046B9A28D126も作成。既存端末は変更しない。修正後の両OSアプリが起動し、開閉10回/復帰5回/入力を確認。AndroidはDebug APKにRust release、iOSはDebug app/Rust debugなので実機Release性能の証拠にはしない。

CIの初回確認ではLinuxのRustチェックとmacOS iOS SDK26.5 / arm64 Simulatorビルドが成功。UbuntuランナーのSDKmanagerはPATH外なのでANDROID_HOME/cmdline-tools/latest/binから呼ぶ。ランナー既定NDKを継承せず、CIコマンドで28.2.13676358を明示する。参照: https://github.com/actions/runner-images/blob/main/images/ubuntu/scripts/build/install-android-sdk.sh
