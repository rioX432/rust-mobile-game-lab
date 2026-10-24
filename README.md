# Rust Mobile Game Lab | SEED Race

**SEED系キャラクターを使った、高品質な視聴者参加型モバイル物理レースをつくる技術・制作PoC。** ゲーム自体を完成させることで、Rust共通Runtime、ネイティブ埋め込み、3D、オンライン参加、Luaルール、制作パイプラインの実用性を検証する。Avvyのアバターを使うことは必須ではない。

> 現在地（2026-10-24）: [PR #14](https://github.com/rioX432/rust-mobile-game-lab/pull/14)、[PR #15](https://github.com/rioX432/rust-mobile-game-lab/pull/15) はmainへ統合済み。**既存の2D・3択オフラインレース**がAndroid Emulator/iOS Simulatorで一巡する。新しいSEED 3D物理ゲーム・通信・Lua・実機/性能は未実装または未受入。従来の3択レースは技術baselineであり完成ゲームではない。

## 開発の入口
- [作るものと非対象](docs/PROJECT.md) / [設計と既存実装](docs/ARCHITECTURE.md)
- [SEEDの制作・権利と輸出](docs/ASSETS.md) / [ゲーム品質の基準](docs/QUALITY.md)
- [開発順・現行Issue一覧](docs/plan.md) / [進捗・保留Worktree](docs/progress.md)
- [AI向け作業ルール](AGENTS.md) / [受入の現状](docs/acceptance.md)
- [親Epic #16](https://github.com/rioX432/rust-mobile-game-lab/issues/16)

## 既存の2Dオフラインbaselineを動かす

```sh
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh run -p game-core --example offline_race --locked
scripts/cargo.sh run -p game-desktop -- --smoke
scripts/android-build.sh
scripts/ios-build.sh simulator
```

[Android起動](docs/android.md) / [iOS起動](docs/ios.md) / [現行2D Renderer](docs/renderer.md)。アプリではOpen → Race → Start / next → 3秒以内にレーン選択 → Result → 次ラウンド。これは単一アプリ内の固定Host/Viewerを使用しており、**別端末の視聴者参加ではない**。

オフラインbaselineの記録: [実操作と画像](docs/offline-native.md)、[初期Native検証](docs/native-verification.md)、[履歴上のレース規則](docs/race.md)、[FFI契約](docs/contracts.md)。

## 公開/権利
このrepoはPublic。SEEDの原画・派生素材・会社固有アセット・Private制作基盤のコードを、利用許諾確認なしにここへ追加しない。実験には権利問題のない合成仮素材を使う。共通制作基盤は別repoにあり、利用版・出典・判断を残す。
