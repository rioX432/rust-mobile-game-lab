# 現在地（2026-10-24 JST）

PoC未完了。リポジトリ作成と再開準備まで。実装・ビルド・起動・実機・性能測定は未確認。

## 作業ルートと権限

- リポジトリ: https://github.com/rioX432/rust-mobile-game-lab （Public、rioの後続承認による）。所有者ADMIN権限を確認。
- フォルダ: `/Users/rio/workspace/src/AnotherBall/rust-mobile-game-lab`
- 初期ブランチ: `main`。既存内容なしを確認してclone。作者はrioX432に紐付くGitHub上の既存コミットから確認し、ローカル設定へ適用。
- `.codex/config.toml`へ`workspace-write/on-request/auto_review`を保存済み。
- **現在のセッションは未反映**。作業ルートは親のAnotherBall、実効approval reviewerは`user`。ファイル保存を実効設定確認として扱わない。
- CodexアプリのComputer Useは禁止されている。内部DBや信頼データの直接書き換えは行わない。ユーザー全体の設定は変更していない。

## 完了した準備

- 仕様全文を`prompts/codex-poc.md`へ保存。Publicへの変更は別途rioが承認。
- `README.md`、`AGENTS.md`、環境、計画、受入表、Issue本文を作成。
- 親Issue: https://github.com/rioX432/rust-mobile-game-lab/issues/1
- 子Issue: https://github.com/rioX432/rust-mobile-game-lab/issues/2 から https://github.com/rioX432/rust-mobile-game-lab/issues/13 。対応と実際の依存先は`docs/plan.md`。受入条件を満たしたIssueはまだない。
- マージ済みPRなし。実装PRなし。準備のみの初期コミットをmainへ保存する。
- 環境診断は`docs/environment.md`。SDK/NDK/Xcodeは導入済み、利用する版の互換性確認・固定は次段階。
- サンドボックス内のGitHub/Simulator/ADB接続失敗は正規の権限昇格で再確認。SDK不足とは区別した。

## 次の操作

現在のCLIを終了して、同じスレッドを正式なオプションで開き直す（CLI 0.164.0の`resume --help`で確認済み）。同じスレッドを同時に2つのCLIで稼働させない。

```sh
codex resume 01a16961-c487-7d63-b9d5-55c57af3e782 --cd /Users/rio/workspace/src/AnotherBall/rust-mobile-game-lab --approve-for-me
```

信頼確認が出た場合は正規の画面で確認する。再開したセッションで`/permissions`と`/status`を確認し、作業ルートと実効`workspace-write/on-request/auto_review`を記録する。GUI選択を確認していない場合、GUI確認済みとは書かない。

再開指示:

> docs/progress.mdとprompts/codex-poc.mdを読み、作業ルートとApprove for meの実効状態を確認して続行。まず環境・共通契約Issueを進め、両OSの図形・タッチ・Surface再生成を早期に検証する。共有契約確定後は独立Worktreeのサブエージェントへ分担する。既存Issueは再作成しない。検証とレビューを通したPRをmerge commitでmainへ統合する。実機条件が未確認の間はPoC完了としない。

## 未検証・未完了

全12子Issueの受入条件、現在の実効Permissions、Rust描画/FFI/ゲーム/通信実装、各OSビルドと起動、実機、5分性能、生成SDKの消費側実行、重点レビュー3回と実装PR統合。Android実機は接続なし。iOS実機は今回の使用対象として未確認。

Permissions反映のため開き直しが必要な場合は進捗保存と再開操作を案内する、という仕様の指示に従い、この段階で新ルートでのセッション再開を待つ。これはPoC完了や目標の取り下げではない。
