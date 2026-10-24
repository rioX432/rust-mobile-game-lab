# Runtime境界: 実装済み2Dと提案中3D
更新: 2026-10-24。**この文書は既存のC ABI利用と、将来の設計候補を混同しない。**

## 現行実装 (PR #14/#15)
- `game_core::Scene { size: [f32;2], shapes: Vec<Shape> }` は論理px。矩形・円の描画データ。レースの勝敗をRendererが変更しない。
- `game-renderer::Renderer`はwgpu Surfaceへ描画し、`resize`、`render(&Scene)`で更新。ゼロサイズでは描画せず、Surface Lost等の復旧は呼出側に伝える。
- `game_runtime`のC ABI正本は [include/game_runtime.h](../include/game_runtime.h)。Androidは`lab.rustmobile.NativeBridge`、iOSはCAMetalLayerをattach。RustワーカーがGPUの初期化/描画/解放を所有。
- `create(seed)`、`attach`、`resize`、`touch`、`pause`、`detach`、`action`、`status`、`destroy`をOS固有アダプタが使用。handleを世代で検査し、古いSurface/旧ラウンド入力を拒否。
- 現行は同一アプリ内Host ID=1/Viewer ID=2。3択のseed+roundから順位を計算する **オフライン検証モード**。他端末参加・3D物理順位ではない。詳しい元契約は[race-contract](race-contract.md)と[race](race.md)。

## 将来の境界 (G03/G06/G10/G11が確定するまで提案)
- `PhysicsWorld`: 固定step/コース/Collider/レース結果。描画は物理状態を参照するだけ。
- `VisualRenderer`: 2D Billboardまたは3D mesh。3D方式はG03で比較後に決定。既存2Dを置換/削除しない。
- `RoomServer`: 参加権限・入力の検証・サーバー時刻・固定step/順位・リビジョン付きsnapshot・重複/旧ラウンド拒否。クライアントは状態を補間し勝敗を確定しない。
- `ScriptAdapter`: server側のLuaの限定Commandのみを型付き検証。開始時`rule_version/script_digest`を固定し次ラウンドから採用。
- `AssetPack`: rights/source/hash/renderer compatibility/collider/scale/LODsを明示。Creative-labの研究bundleをそのままゲーム配布物とみなさない。

新たなAPI名・versionや具体的なデータ形式は、担当Issueの技術選定が成立するまでは**仮契約**。実装済みかのように記載しない。1回の共通契約変更はG03/G10等の統合担当が管理し、Android/iOSの互換性と回帰を検査する。
