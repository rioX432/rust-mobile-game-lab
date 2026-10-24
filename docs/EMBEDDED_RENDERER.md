# Embedded Game Renderer / 配信ホスト連結設計（G19 / G20）

更新: 2026-10-24。正本の実装範囲は各crateの公開APIとテスト。**本設計は製品結合完了を示さない。**

## 目的とスコープ

SEED Raceを単独アプリとして完成させながら、将来、別のネイティブ配信Hostが持つアバター描画と
**同一GPU・単一フレーム内で合成**できるゲームRuntimeへ育てる。
公開PoCでは自由利用できる図形・ダミーAvatar layerのみ使用し、非公開Hostや社内素材は依存にしない。

達成を独立して判定すること:

1. **Standalone Game:** Android/iOS/desktopで自分のSurfaceを保持した状態でゲームを動かす。
2. **Public Embedded:** 別のRust Hostが作成したGPU Device/Queue/RenderTargetへゲームを描き、合成する。
3. **Private Consumer:** 実際の配信Hostへ統合し、Avatar/音声/ゲームネットワークと共存させる。

Standalone SDK成功や、単一Textureへの描画成功を、Private製品連結の受入として扱わない。

## 結論: 二つのRendererモード、一つのGame Core

```text
game-core (OS/GPU非依存のルール、入力、state/snapshot)
  |                                           |
  +--> Standalone Renderer                    +--> Embedded Renderer
       - gameがGPU/Surfaceを所有                    - HostがDevice/Queue/Surfaceを所有
       - 自前frame scheduler/Present                 - Hostのencoder + TextureViewへencode
       - 開発用Native Host/desktop                    - 単一Rustリンク単位で型付きGPU共有

[Private Native Host: 別リポジトリ／公開リポジトリでは実装しない]
   既存アバターRuntime  ---> avatar texture ---+
   Game Runtime          ---> game texture   ---+--> Host compositor --> 1 Present
   RTC audio/data              別パイプライン |
   Game room WebSocket         別パイプライン |
```

### 所有権と呼出し順

| 責務 | Standalone | Embedded |
| --- | --- | --- |
| GPU Adapter / Device / Queueの作成 | game native wrapper | **Hostのみ** |
| Surfaceの生成/resize/reconfigure/detach | game native wrapper | **Hostのみ** |
| シミュレーションとsnapshot | game core / client | game core / client |
| Render passの内容 | game renderer | game renderer |
| target format/size/MSAA/alpha | game側 | **Hostが契約を決める** |
| Frame clock / encode順序 / submit / present | game native wrapper | **Hostのみ** |
| GPU device loss / background復帰 | game native wrapper | **Hostが通知して再生成** |
| RTC音声/コメント/アバターTracking | 対象外 | 製品Host、ゲームとは別経路 |
| multiplayer Room / 入力検証 | Rust game server | Rust game server |

ゲームは描画対象を借り、**HostのSurfaceを取得せず、勝手にpresentせず、GPU Deviceを新規生成しない**。
同じGPU Deviceを共有することと、最終画面上で正しく合成されることは独立した検証項目。

### 2026-10-24時点の2D APIスパイク

`game-renderer::EmbeddedRenderer` を追加し、Hostの `wgpu::Device` と `wgpu::Queue` のclone、
color attachmentの `TextureFormat` を受け取る。ゲームの2D `Scene` とHostの
`TextureView`、`CommandEncoder`、明示 `LoadOp` を `encode` に渡す。
元の `game-renderer::Renderer` はStandalone専用に維持する。
**新しい3Dエンジンの採否や3DのEmbed完了を意味しない。**

```rust
// 同一Rustリンク単位にいるHost内での使用イメージ（擬似コード）
let mut game = EmbeddedRenderer::new(
    host_device.clone(), host_queue.clone(), host_layer_format,
).await?;
game.encode(
    &game_scene,
    &host_game_layer_view,
    &mut host_frame_encoder,
    wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
)?;
// host が必要な別layerの合成をencodeしてから queue.submit、最後にpresentする
```

- このAPIは同一Rustプロセス・リンク単位で利用する**Rust内部の実験契約**。
- `wgpu::Device`等をJNI、Swift、C ABI、別々の`.so`やXCFramework間に公開しない。
  別ネイティブSDK同士のGPUハンドル交換を独自の共通ABIと誤解しない。
- 実製品では必要なRust依存を**一つの統合ライブラリ**にリンクしてNativeへの公開口を一つにまとめるか、
  Host内のRustアダプタを作成する。外部アダプタの版固定と配布許諾をレビューする。
- `Renderer`の2D shader/pipeline定義は両モードで共有し、StandaloneとEmbeddedの描画差分を減らす。

### GPU合成の技術契約

1. **Format**: Hostが色フォーマットとサンプル数を宣言。最初の2Dスパイクは
   `Rgba8Unorm`、1x MSAA、ColorAttachment。生成したpipeline形式とTextureViewの形式は一致させる。
2. **Clear / alpha**: game layerは透明でclearし、Hostが合成順序を定義。
   `Load`は初期化済みRenderTargetに限る。alpha mode、premultiplyの有無、linear/sRGB変換は
   Golden比較で確認してから本番契約を固定する。GPUの見た目が正常でも色が等価とは限らない。
3. **Ordering**: Hostが入力/シミュレーション→game encode→avatar encode→composite→
   queue submit/presentを同期。別encoder/Queue submitの場合は順序と資源hazardを明示する。
4. **Resource lifecycle**: HostのDevice/Queueは全レイヤーより長命。
   Surface detach/resize/GPU lossではHostが新Targetを提供し、必要ならEmbeddedRendererを作り直す。
   描画中のdetachとGPU frameの寿命を混同しない。
5. **No readback**: 毎フレームCPU RGBA readbackやPNG化をしない。テスト/診断の明示的readbackのみ許可。
6. **3D**: G03の結果に応じ、depth/stencil ownership、camera、model、physicsとは独立した
   Host-owned render targetの同じ思想を維持。2D APIをそのまま汎用3Dへ過剰一般化しない。

### wgpuバージョン／ビルド契約

2026-10-24の調査時点ではStandalone PoCが `wgpu = 30.0.1`、
連結候補のPrivate Avatar Hostは `wgpu = 29.0.4` を使っている。
両方を直接リンクする際はCargoで単一の互換`wgpu`系統を使い、
統合先で公開型の一致を静的に検証する。別バージョンのDeviceを無理に変換しない。
PoC側のversion変更は3D方式選定やStandalone回帰の検証を伴って行う。
PublicにPrivateの依存ソースや秘匿マニフェストをコピーしない。

## 通信の責務分離

- **RTC**: 配信/通話、音声、既存アバター情報など。ゲームの権威的物理計算は持たせない。
- **Game Network**: クライアントからRoom Serverへtyped input、serverからsnapshot/event。
  途中参加・disconnect/reconnect・遅延入力の拒否はG10/G11が担う。
- **UI**: Start/Join/Exit、配信コメント、Avatar表示、タッチのフォーカス切替はHost。
- **Game**: 単一プレイヤーでも複数プレイヤーでも、描画は受信済みsnapshotの補間結果のみ。
  視聴者が参加しない観戦モードも、映像RTCでゲームを再配信するのではなくローカル描画を基本とする。

## 実証順序・責任範囲

| Gate | 実証 | Issue | 状態 |
| --- | --- | --- | --- |
| A | Android/iOSの旧Standalone 2D回帰 | G02 #18 | 一部受入、残件あり |
| B | Host所有Device/Queue/2D offscreen encode（API実装・明示GPU smoke） | G19 #37 | 実装スパイク、GPU実走/合成未受入 |
| C | ダミーAvatar + gameのGPU合成、1 Surface/1 Present | G19 #37 | 未受入 |
| D | G03が選んだ3Dモードでも同じ所有権を維持 | G03 #19 / G19 #37 | 未受入 |
| E | Native standalone SDKを別アプリで起動 | G13 #29 | 未受入 |
| F | Private Hostで実Avatar/RTC/ゲームが共存 | G20 #38 | **Private実証未受入** |

G19/G20は追加の連結ゲート。**SEED Race本編のPlayable完成はPrivate Hostを待たない**。
PublicのCIやGPU smokeはPrivateの実アバター/RTCとの結合を保証しない。

## G19の次の実装・測定（実作業として未完了）

- Host疑似アバターを含むoffscreen compositor consumerを作る。
- GPUの単一Device/Queue、同一frameでの複数pass/色合成を実GPU検証し、Goldenと修復不能状態の失敗を記録する。
- Android/iOSのNative wrapperを経由した2D Embedded表示を実操作で検証し、従来Standalone起動を回帰確認。
- 現行2DスパイクのGPU smokeは `#[ignore]` にしてあり、GPU runnerで明示実行する必要がある。
- G03の3D選定後、Host GPU ownershipと3D asset/depth/camera contractを更新する。

## セキュリティ・情報境界

Publicには許諾済みの合成ダミー素材・公開可能な契約・再現手順だけを保存する。
Private Hostのコード・binary・権限・アバター・内部URL・認証情報・計測画面は公開しない。
Privateとの結合テスト・社内レビュー・性能ログはPrivate側へ保存し、
Publicには匿名化可能で再利用価値のある仕様差分だけ戻す。
