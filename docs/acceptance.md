# 横断受入マトリクス | SEED Race v1
現行の判断基準は[QUALITY](QUALITY.md)と担当Issue。以下は2026-10-24時点で既に実証したものと、これからの作業を分けた要約。

| 目標/観測項目 | Android | iOS | 実測・次Issue |
| --- | --- | --- | --- |
| 旧2D共通Rendererとタッチ・画面復帰 | Emulator確認 | Simulator確認 | [offline-native](offline-native.md) / 旧PR #14,#15 |
| 旧3択のStart→Result→次ラウンド | Emulator確認 | Simulator確認 | Rust状態遷移の技術baselineで、物理ゲームではない |
| 動的密度・初期化中Close・全phase復帰 | 未確認 | 未確認 | G02 #18 |
| 3D renderer方式のモバイル埋込 | 未実装 | 未実装 | G03 #19 |
| SEEDモデルとゲーム用Content Pack | 未採用 | 未採用 | G04 #20,G05 #21（原画公開権利未確認） |
| 20体の物理走行/順位 | 未実装 | 未実装 | G06 #22,G07 #23 |
| カメラ・FX・音・ゲーム可読性 | 未実装 | 未実装 | G08 #24,G09 #25 |
| サーバーRoomと再接続・視聴者参加 | 未受入 | 未受入 | G10 #26,G11 #27 |
| ラウンドごとのserver側Lua更新 | 未実装 | 未実装 | G12 #28 |
| AAR/XCFrameworkを別アプリへ組込 | 未受入 | 未受入 | G13 #29 |
| Host-owned Embedded 2D / 同一GPU疑似Avatar合成 | API追加、GPU実走未確認 | API追加、GPU実走未確認 | G19 #37、現状2Dコードスパイクで3D/合成未受入 |
| Private配信HostでAvatar/RTC同居 | 未検証 | 未検証 | G20 #38。実証情報はPrivateに保存 |
| Release相当の実機性能・発熱・サイズ | 未測定 | 未測定 | G14 #30 |
| ゲーム作品の総合品質・再利用評価 | 未評価 | 未評価 | G15 #31 |

各Issueのcheckboxは**実際の証拠でのみ**完了へ。CI成功はAndroid/iOS実機の証拠ではない。SDKの生成は消費側の起動まで検証する。60fpsは初期目標であり数値の観測ではない。

既存の実行記録の原本は[offline-native](offline-native.md)、[native-verification](native-verification.md)、[レビュー](reviews/offline-race.md)。旧受入条件の達成履歴として保持する。
