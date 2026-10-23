# オフラインレース独立レビュー

## 結論

指定差分 `origin/main...HEAD` をソースと既存テストから確認し、限定したレビュー範囲でP1/P2の指摘はありません。

## 確認した範囲

- `crates/core/src/game.rs`: seed/roundからの順位、締切と演出時刻、入力権限・ラウンド・phase、人数と選択肢の境界、整数時刻境界、Snapshotと描画Sceneの分離。
- `crates/runtime/src/offline.rs` と `crates/runtime/src/lib.rs`: Roomのworker所有、Probe/Race切替、キュー受付と適用結果の分離、入力時のround ID・Surface generation・input epoch、pause/resume・detach・destroy後の遅延入力。
- Android Kotlin host、iOS Swift host、C header: タッチ座標、ネイティブ操作、Surface/Metal layer attach/detachと画面離脱時の寿命。
- GPU取得gate分離とMetal timeout: 関連コードは差分に含まれず、既存修正への変更がないことを確認。
- `Status.game`はRoomのSnapshotを公開し、Probeの`touches`/`selected_lane`等は別フィールドの診断値として保持される。レース結果の決定にProbe値や描画Sceneは使われない。

## 根拠と再現性

- `crates/core/src/game.rs:98-200` は参加者上限20、option 0..2、host操作、round/phase遷移、単調時刻と締切境界を実装。締切と演出の加算前に全タイムラインを検査する (`139-156`)。順位はseedとroundから固定手順で決まる (`207-225`)。
- 同ファイルの既存テストは締切直前/境界、複数phaseを飛ばすadvance、最大時刻付近、20人集計、固定seedの再現、Scene生成がSnapshotを変更しないことを確認する (`402-699`)。
- `crates/runtime/src/lib.rs:113-134,451-494,520-558` は入力時に世代・epoch・観測roundを記録し、worker適用時に寿命条件を再確認する。関連する既存テストはpause/resume、detach、destroy、surface再生成、旧round入力を扱う (`770-862`)。
- `crates/runtime/src/offline.rs:35-123` は単一worker所有のRoomとProbe診断を分け、描画へSnapshotを渡す。既存テストは一巡、seed一致、背景時間経過、旧入力拒否を扱う (`138-221`)。
- iOS `MetalHostView.swift:42-64` はpoint座標をdrawable pixelへ変換し、Android `ProbeSurfaceView.kt:15-30` はView内座標を送る。RustのProbe入力でlaneを選ぶが、勝敗と集計はRoomのみが確定する。
- iOS `RuntimeSession.swift:79-137` はclose中にdetach/destroyを再試行可能に保ち、`ViewControllers.swift:108-142,193-205` はscene inactive・画面離脱・layer再生成を処理する。Android `ProbeSession.kt:25-108` と `MainActivity.kt:78-119,162-187` はSurface callbackとActivity pause/closeを接続する。

## 未確認

- このレビューではfmt/build/testを実行していない。既存テストのソースを確認したが、今回の実行結果としては扱わない。
- Android/iOSアプリの実ビルド・Emulator/Simulatorでの実操作・実機・性能は未確認。統合担当が別途行う範囲で、今回のコード不具合指摘には数えない。
- `RuntimeStatusTests`のUI文字列一致は受入証拠として使用していない。
- sessionの実効作業ルートと`workspace-write`、`auto_review`は現在の実行コンテキストで確認。`on-request`はこのコンテキストから独立して読み取れる値がなく、未確認として権限記録に明記した。

## 次の確認

統合担当の両OSビルドと実操作結果を進捗記録へ反映する。通信、投票、SDK、実機、性能はこのレビュー対象外の未完了条件として維持する。

統合担当はレビューセッションの`turn_context`を直接読み、専用cwd、`approval_policy=on-request`、`approvals_reviewer=auto_review`、`sandbox_policy.type=workspace-write`を確認した。秘密を除いた確認記録はローカルの`artifacts/offline-review-permissions-readback.json`。レビュー担当自身の確認範囲とは区別する。
