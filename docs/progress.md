# 現在地（2026-10-24 JST）

PoC未完了。同じRust描画器のAndroid/iOS部分埋め込みをEmulator/Simulatorで確認済み。ゲーム・通信・投票・SDK消費・実機・性能は未完了。

## 作業状態

- リポジトリ: https://github.com/rioX432/rust-mobile-game-lab （Public、rio承認済み）
- 作業ルート: `/Users/rio/workspace/src/AnotherBall/rust-mobile-game-lab`
- 実効セッション記録とdeveloper指示で専用cwd、`workspace-write/on-request/auto_review`を確認済み。CLIで確認。GUI表示は未確認（操作禁止）。全ワーカーも専用Worktreeで実効記録を保存。
- 作者はrioX432に紐付く設定。現在のブランチは`feat/native-embedding`。
- 図形/FFI/両OSホストを並列実装して統合。共有契約基点`5fef344`、runtime統合`841b9ff`、重大レビュー指摘の修正`5f34102`。
- 修正後の両OSビルド、開閉10回・背景復帰5回・再生成後入力が成功。Rustの所有インスタンス/ワーカー/NativeTarget参照は各1、停止中の提出増分0、last_error=null。
- Rust core3/renderer4/runtime8テスト、strict Clippy/format、Androidホスト3テスト/lintを確認。詳細と画像はdocs/native-verification.md。
- コメント整理はrepo固有スキルがないため中央clean-slopを適用。安全契約・API制約のコメントは保持し、冗長コメントの削除対象なし。

## Issue・PR

親: https://github.com/rioX432/rust-mobile-game-lab/issues/1

- 環境/契約 https://github.com/rioX432/rust-mobile-game-lab/issues/2 （CI待ち）
- レース https://github.com/rioX432/rust-mobile-game-lab/issues/3 （独立ワーカーが実装・テスト済み、統合前）
- 描画/Desktop https://github.com/rioX432/rust-mobile-game-lab/issues/4
- FFI/寿命 https://github.com/rioX432/rust-mobile-game-lab/issues/5
- Android https://github.com/rioX432/rust-mobile-game-lab/issues/6
- iOS https://github.com/rioX432/rust-mobile-game-lab/issues/7

全対応はdocs/plan.md。実装PR: https://github.com/rioX432/rust-mobile-game-lab/pull/14 （open）。完了Issue・マージ済みPRはまだなし。CIのRust/iOSは成功し、AndroidのsdkmanagerがPATH外で終了127になったため公式配置のパスを明示する修正を追加した。ランナー既定NDKも28.2へ明示する。検証対象は減らさず再確認中。未検証条件のあるIssueには自動closeキーワードを使わない。

## レビュー

最初の独立レビューでMetalの取得待ちがUI停止用gateを保持するP1を確認。取得/準備とsubmit/presentを分離し、遅延フレームの再検査・gate外破棄・Metal timeoutを実装。故障注入テストと限定再レビューで残存P1なし。修正後の両OS実行も再確認。docs/reviews/native-boundary.mdに記録。

## 並列作業と次の段階

完了したWorktreeは`.worktrees/android`（5ad8565）、`.worktrees/ios`（db9d928）、`.worktrees/renderer`（8ab2449）、`.worktrees/review-native`（a4719c4）。ソースと証拠は保持し、完了3実装ワーカーの再生成可能targetだけを容量確保のため削除した。

レースは`.worktrees/race` / `feat/race-core` / `fbbbba8`。契約docs/race-contract.mdに沿い、core限定16テストとoffline_race例が成功したとの報告を受領。seed42、順位[0,2,1]、人数[2,0,1]、勝者[10,30]。親によるコード/証拠確認と統合はこれから。共有manifest/lockfile/FFI/ネイティブは統合担当が所有する。

1. native PR #14のRust/Android/iOSのCIを確認し、必要チェック成功後にmerge commitでmainへ統合する。
2. レースの確定APIと証拠を読み、別の統合ブランチでオフラインゲームを実ホストへ接続する。
3. 通信契約を確定し、サーバー/共通クライアントを独立Worktreeで分担する。
4. 実通信と途中参加/再接続 → 投票 → SDK消費側 → 実機/性能を進める。

```sh
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings
scripts/android-build.sh
scripts/ios-build.sh simulator
```

専用端末: Android AVD `RustMobileGameLab_API35_20261024` / emulator-5630（2GB data）、iOS Simulator `06E9B566-FCC7-4A9B-9A0D-046B9A28D126`。両方稼働中でPoCアプリ導入済み。既存の他端末を変更しない。根拠はartifacts/native-mobile-smoke-after-acquisition-fix.jsonとコミット対象docs/evidence。full logsはartifactsのios-runtime.stderr.log、android-runtime.log、各build/test log。

## 未確認・ブロッカー

実機、5分性能/メモリ推移、実GPU/OSハング、実Surface/Device Lost故障注入、モバイル初期化中高速Close、ゲーム/通信/投票、SDK生成物の消費側。Android実機は未接続。iOS実機の今回の対象は未確認で、対象端末の指定を非同期で質問済み。実機の返答を待つ間も独立作業を進める。親Issueを閉じずPoC完了としない。

初回CI: https://github.com/rioX432/rust-mobile-game-lab/actions/runs/39993616105 （Rust/iOS成功、Android SDKコマンド解決で失敗）。CIのiOSもSDK26.5 / arm64-apple-ios16.0-simulatorでビルド成功。独立レビューはPRへCOMMENTとして記録済み。
