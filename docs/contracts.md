# 最小共有契約

最初に図形表示・タッチ・Surface寿命を両OSで確認してから、同じ境界へオフラインレースを接続した。ゲーム操作と状態の追加契約は[offline-native.md](offline-native.md)。通信は後続の未完了範囲。

## 描画と入力

`game-core::Scene { size: [f32;2], shapes: Vec<Shape> }`。座標は論理ピクセル、矩形はx/y/幅/高さ、円は中心x/y/半径。色はRGBA。ネイティブからはSurface内の物理ピクセル座標を渡し、Rustが密度に応じて扱う。ネイティブ領域の入力は渡さない。

`game-renderer::Renderer::new(instance: &wgpu::Instance, surface: wgpu::Surface<'static>, width: u32, height: u32)`はasyncで`Result<Renderer,String>`を返す。`resize(width,height)`と`render(&Scene) -> Result<(),String>`を提供する。ゼロサイズは描画せず、失効Surfaceは再構成する。ゲームの勝敗は変更しない。

ネイティブランタイムは`prepare_frame(&Scene)`で描画先取得とコマンド準備をgate外で行い、取得後に世代・paused・destroyedを再検査して`submit_frame`だけをgate内で実行する。失効した取得結果はgate外で破棄する。Metalではwgpuがconfigure時に無期限取得へ変更するため、固定版の公開HAL Surfaceからlayerの取得timeoutを毎回有効化し、getterで確認する。

## ネイティブ境界

C ABIの正本は`include/game_runtime.h`。ライブラリ名は`game_runtime`。iOSはCAMetalLayerを渡す。Androidは`lab.rustmobile.NativeBridge`の`@JvmStatic external`メソッドをJNIで提供する:

```text
create(seed: Long): Long
destroy(handle: Long): Int
attach(handle: Long, surface: android.view.Surface, width: Int, height: Int, scale: Float): Int
resize(handle: Long, width: Int, height: Int, scale: Float): Int
touch(handle: Long, xPixels: Float, yPixels: Float): Int
pause(handle: Long, paused: Boolean): Int
detach(handle: Long): Int
action(handle: Long, action: Int, argument: Int): Int
status(handle: Long): String
```

初期化は非同期。状態JSONは`phase`（Initializing/Running/Paused/Detached）、`frames`、`touches`、`selected_lane`、`live_instances`、`surface_generation`、`last_error`を持つ。Native UIに状態を表示する。`mg_destroy`後は無効ハンドルとなる。GPU初期化/描画/解放はRustワーカーが所有し、入力とコマンドは上限付きで受け付ける。ネイティブの画面を閉じるときにdetach→destroy。古いSurfaceコールバックが新しい接続に作用しないよう、ホスト側も画面/接続世代を識別する。

detach/destroyは描画の排他区間を通してそのSurfaceへの新しい描画を無効化し、GPU初期化の終了をUIで待たせない。Rustが取得したANativeWindow参照/retainしたCAMetalLayerは、Rust Surfaceを破棄した後に解放する。破棄時の遅延GPU初期化は接続世代の不一致で採用しない。

## 後続ゲーム/通信の境界

ゲーム共通状態はWaiting → Selecting → Animating → Result。レースはseedと受理入力からサーバーが結果確定し、描画補間は結果を変えない。投票も選択/締切/集計を同じ経路で扱う。

通信はWebSocket JSON、protocol_version=1、room_id/session_id/round_id/command_idを持つコマンドと、単調増加revision付きスナップショット。初期上限は20参加者、メッセージ16 KiB、送信待ち64件。ホスト資格は部屋作成時の不透明token。ホスト切断は待機し再接続、移譲しない。ゲーム規則/時刻/認可はサーバー正本。詳細の型と重複処理は通信実装前に確定する。
