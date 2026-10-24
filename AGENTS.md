# AI作業ルール | SEED Race v1 (2026-10-24)

## 必読順と正本
1. [PROJECT](docs/PROJECT.md): 何を作るか・非対象・マイルストーン
2. [ARCHITECTURE](docs/ARCHITECTURE.md): 実装済みと設計案・依存の向き
3. [ASSETS](docs/ASSETS.md) / [QUALITY](docs/QUALITY.md): 原画・権利・検証・美術基準
4. [plan](docs/plan.md): SEED-G01〜G18の依存、旧Issue移管
5. [progress](docs/progress.md) と担当GitHub Issue: 作業状況・実行条件・受入基準

**旧 `prompts/codex-poc.md`、`docs/issues/`、閉じた旧Issue #1,#6〜#13 は現行指示ではない。** 過去の実装履歴・証拠（docs/evidence, docs/reviews, offline-native.md）は保存するが、ここから次のタスクを自動発見しない。競合したら推測で実装せず担当Issueに記録する。

## 絶対に守る境界
- 現在の目的は「SEED系オリジナルキャラクターで高品質な視聴者参加物理レース1本を完成し、Rustモバイル共通基盤を検証」。アバター描画、3択投票の2本目、汎用ゲームエディタは優先しない。
- 既存2Dのネイティブ描画/オフラインレースはマージ済み基準線。壊さない。3D描画方式やRapier3DはG03の比較が終わるまで採用確定と書かない。
- 勝敗と物理演算はRustサーバーを最終正本とする設計。Lua/Luauは **server-side、イベント境界、型付きCommand、ラウンドごとに版固定** の後続検証であり、毎フレームの物理演算をスクリプトへ渡さない。
- Private `rioX432/creative-lab`・`rioX432/persona-script-lab` は外部の固定版利用候補。許諾のないソース・Avvy/AnotherBall素材・会社のレシピをPublic repoへ複製しない。
- ユーザー提示のSEED画像はデザイン参照。Publicへの原画/派生素材の公開・配布権が確認されるまでダミー素材を使用し、権利と美術採用を別に扱う。
- 未実測を成功とせず、コード/静的テスト、Emulator/Simulator、実機、アプリ/ネットワーク、実機性能を区別する。

## 担当Issueの実行プロトコル
- 開始時に最新mainとPR/Issue、dirty worktree、担当ファイル、依存の受入を確認。旧`feat/room-server`等のローカル途中成果は所在確認前に破棄しない。
- 専用リポジトリ/Worktreeの実効Permissionsを **workspace-write / on-request / auto_review (Approve for me)** と確認。`.codex/config.toml`の存在だけで有効と判断しない。Full access、never、policy bypassを代用しない。
- 1 Issue = 観測可能な結果と証拠。Workerごとに固有Worktree/担当範囲/基準commit/受入条件を渡す。共通FFI、Protocol、Cargo.lock、共有契約は統合担当が調整する。
- PRは目的、既存との違い、実行した検証・証拠のURL、未確認、契約/ドキュメント更新、権利確認を記載。独立レビューと既存CIの正常性を確認してからmerge commitで統合する。
- **新たなドキュメント・IssueドリフトCIガードは不要**。GitHub Issue・文書の整合はPRレビューで確認する。
- 未確認の必須受入条件を持つIssueを完了として閉じない。依存が未解決なら別の独立タスクを進め、状態を報告する。
- **制作の責務:** G04はSEED原画の仕様/2D・3D方式選定、G05はContent Packの生成・輸出入契約、G16 #33は編集可能な基準キャラ1体の実制作、G17 #34は独自コース/障害物の実制作、G09は動き/FX/音、G18 #35は**実ゲーム画像/動画からの批評と局所的な再制作**。候補の自動生成や資料化だけで制作完了としない。採用正本とAI試作を混同しない。
- 回答・進捗報告は日本語で「目的/今回の差分/根拠/未確認/次アクション」を短く明確に。
