# 現在地 | SEED Race v1
更新: 2026-10-24。情報源はmain、GitHub Issue、マージ済みPR、端末実行記録。未確認は未確認と書く。

## 何ができているか
- main baseline: **5f665d667f8d04dc3310cc8d6320a8034ccbaeb2**（[PR #14](https://github.com/rioX432/rust-mobile-game-lab/pull/14)、[PR #15](https://github.com/rioX432/rust-mobile-game-lab/pull/15) 統合）。新計画のdocs-only PRは別。
- Android API35 arm64 Emulator/iOS26.5 Simulatorで同じRust 2D描画+オフライン3択レース一巡。入力、結果、次ラウンド、画面開閉10回、Resultから背景復帰5回、Layer/Surface再生成の操作証拠は[offline-native](offline-native.md)。
- Rust core16 + renderer4 + runtime16、Android host JUnit4、両OSビルド、関連CIはPR #15に記録。AndroidはDebug APK+Rust release、iOSはDebug app+Rust debug。**実機FPSや実物理ゲーム成功ではない**。
- 完了旧Issue: #2〜#5。旧#6/#7の密度変更等はG02へ移管。

## 新たな目標
[PROJECT](PROJECT.md)に沿ったSEED1体・独自コース1つ・20体の立体物理レースを、G03比較→G06物理→G05素材→G07 playableへ進める。その後G10/G11で視聴者参加、G12でLua、G13/G14でSDKと実機評価。現在の新Issueは[#16〜#31](plan.md)。

## 先に確認が必要なローカル資産
以前の作業報告にローカル作業として以下が記載されているが、**GitHubに該当ブランチが見えず、今回コードの実読・統合確認はしていない**。破棄・reset・stashしない。G01で作業者の実環境を検査して再利用/隔離/破棄方針を決める。
| ローカル作業名 | 記録されたcommit | 状態 |
| --- | --- | --- |
| feat/room-server | 80f4875 | サーバー6テスト/loopback報告、現物要確認 |
| feat/network-client | 1e736b1 | mock peer/再接続報告、現物要確認 |
| feat/race-network | 4e209e0 | SDK生成・通信の実験、消費側実証不足 |
| feat/offline-native-race | 72c6685 | オフライン統合の元。mainへ統合済み |

## 次の実行順
1. G01で旧参照を外し新ドキュメントとIssue整理、ローカル資産の棚卸し。
2. G02 Nativeの残検証、G03 3D方式比較、G04 SEEDデザイン・権利を並列で。
3. G05アセット・G06物理の結果からG07オフラインPlayableを生成。
4. 追加は[plan](plan.md)を参照。実装が始まったら各IssueとPR・実行証拠をこのページへ反映。

## 検証再現
```sh
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh run -p game-core --example offline_race --locked
scripts/cargo.sh run -p game-desktop -- --smoke
scripts/android-build.sh
scripts/ios-build.sh simulator
```
専用端末/権限の初回記録は[environment](environment.md)。2026-10-24に専用エミュレータを停止した記録があり、今回の文書更新で再起動・実機テストは実施していない。

## 未確認・ブロッカー
3D表現/物理、SEED原画の公開利用許諾、ゲーム制作・音声/FX、server実接続、Android/iOSの別端末参加、Lua adapter、SDK別app、物理実機性能・発熱、実GPU/Device Lost故障注入。ゲーム全体の完了とは宣言しない。
