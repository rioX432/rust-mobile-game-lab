# 現在地（2026-10-24 JST）

PoC未完了。最初の図形・入力・Surface寿命を実装中。ゲーム/通信は未接続。

## 確認済み

- リポジトリ: https://github.com/rioX432/rust-mobile-game-lab （Public、rio承認済み）
- 作業ルート: `/Users/rio/workspace/src/AnotherBall/rust-mobile-game-lab`
- 再開後の実効セッション記録とdeveloper指示で`workspace-write/on-request/auto_review`、専用ルートを確認。GUI表示は未確認（操作禁止）。
- 作者はrioX432に紐付くローカル設定。初期記録はmainへpush済み。
- 現ブランチ: `feat/native-embedding`。共有契約コミット`5fef344`。
- コアの座標/密度/復帰時間テスト3件、レジストリの無効ID/世代/上限テスト3件が成功。
- Rust 1.99.0 / wgpu 30.0.1 / winit 0.30.13、Cargo.lockを作成。詳細はdocs/contracts.mdとdocs/adr/0001-native-embedding.md。

## Issue・PR

- 親Issue: https://github.com/rioX432/rust-mobile-game-lab/issues/1
- 開発環境/契約: https://github.com/rioX432/rust-mobile-game-lab/issues/2
- 描画/Desktop: https://github.com/rioX432/rust-mobile-game-lab/issues/4
- FFI/寿命: https://github.com/rioX432/rust-mobile-game-lab/issues/5
- Android: https://github.com/rioX432/rust-mobile-game-lab/issues/6
- iOS: https://github.com/rioX432/rust-mobile-game-lab/issues/7
- 全対応はdocs/plan.md。完了Issueなし、実装PR/マージ済みPRなし。

## 並列作業

基点は共有契約コミット。各ワーカーは`codex exec --cd <worktree> --approve-for-me`で開始し、それぞれ実効セッション記録を`artifacts/worker-permissions.txt`へ保存した。

| 担当 | Worktree / branch | 編集範囲 |
|---|---|---|
| Android | `.worktrees/android` / `feat/android-native` | apps/android、scripts/android*.sh、docs/android.md |
| iOS | `.worktrees/ios` / `feat/ios-native` | apps/ios、scripts/ios*.sh、docs/ios.md |
| 描画/Desktop | `.worktrees/renderer` / `feat/shared-renderer` | crates/renderer、apps/desktop、docs/renderer.md |

統合担当はcore/runtime/共有契約/Cargo.lock/CIを所有する。workerはローカルcommitのみ、統合担当が検証・PR・マージを管理する。同一ディレクトリを共同編集しない。

## 次の作業

1. ランタイムとJNI/C ABIを完成させ、描画ワーカーのコードを統合して境界テスト。
2. Android/iOSホストを統合し、両OSの専用Emulator/Simulatorで図形・タッチ・開閉・復帰・再生成を実行。
3. FFIと両OSの重点レビュー、修正と再検証、実装PRを作成してCI通過後にmerge commitで統合。
4. その後、オフラインレース→通信→投票→SDK消費側→実機/性能へ進む。

```sh
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings
scripts/android-build.sh
scripts/ios-build.sh
```

後2つのスクリプトはワーカー作業中。未実施のチェックを成功と扱わない。

## 未確認・未完了

現在の変更全体のビルド、両OS/ Desktopでの起動と入力、Surface寿命の実行検証、実機、ゲーム/通信、投票、SDK、性能測定、3回の重点レビュー、実装PR統合。Android実機は接続なし。iOS実機は今回の使用対象として未確認。必須条件が残る間、親Issueを閉じずPoC完了としない。
