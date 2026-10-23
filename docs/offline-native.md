# オフラインレースのネイティブ接続

対象: [レース #3](https://github.com/rioX432/rust-mobile-game-lab/issues/3)、[Android #6](https://github.com/rioX432/rust-mobile-game-lab/issues/6)、[iOS #7](https://github.com/rioX432/rust-mobile-game-lab/issues/7)のオフライン部分。
ブランチは`feat/offline-native-race`、基点は`b7721a8`。ネットワーク、core/game、renderer、共有manifest/lock、scriptsは変更していない。共通の進捗は統合担当が`docs/progress.md`へ反映する。

## 実装

既存の非同期FFIキューを通して、両OSから次の操作を送る。作成時の既定モードはProbeのまま。

| `mg_action` | argument | 意味 |
| --- | --- | --- |
| 0 | 任意（無視） | 従来のProbe選択リセット |
| 1 | 0 / 1 | Probe / Race切替 |
| 2 | 0 | ホストのStart・次ラウンド |
| 3 | 0..2 | 視聴者の3択入力 |
| 4 | 0 | ホストの早期Close |

`mg_create(seed)`から作るRust `game::Room`にhost 1 / viewer 2を登録する。Start/Closeはhost 1、タッチとNative Chooseはviewer 2。画面のChoose 1〜3はRustのoption 0〜2に対応する。ゲーム規則・締切・結果はRoomだけが決め、同じRendererへ`game::scene`を渡す。

`Status.game`はProbeではnull、Raceでは共通`Snapshot`。両OSでround/phase/result、選択肢の順位・人数・勝者を文字表示し、既存のJSONも残す。Probeの既存fieldsは維持する。

戻り値0はキューへの受理であり、適用成功ではない。不正action/argumentは直ちに`-2`、キュー満杯や選択入力不能なSurfaceは`-3`。ゲーム・遅延Native Chooseの拒否は`last_action_error`に出し、GPU/workerの`last_error`とは分離する。成功したゲーム操作・モード切替・Resetは`last_action_error`をクリアする。遅延タッチは状態を変えず破棄する。

選択入力は受付時のround ID、Surface世代、入力世代を保持する。pauseから復帰した後も古い入力は適用しない。detach/再接続、destroy、モード切替でも古い選択入力を拒否する。Start直後など表示Snapshotがまだ更新されていない入力は旧roundとして拒否され得るため、Selecting表示を待って選ぶ。

RoomはSurfaceより長命で、再生成やProbeへの切替でも維持する。経過`Instant`から整数`now_ms`を作って`Room.advance`へ渡す。背景中も締切・演出時計は進み、復帰時に停止時間をdelta timeとして投入しない。GPU初期化・フレーム取得のgate分離とMetal timeoutは変更していない。

## 検証（2026-10-24 JST）

作業セッションの`turn_context`で、専用cwdと実効`workspace-write / on-request / auto_review`を確認し、秘密を含まない抜粋を`artifacts/worker-permissions.txt`へ保存した。Git作者はローカル設定の`RIO(Ryosuke Shimizu) <34413373+rioX432@users.noreply.github.com>`。GUIのPermissions表示は未確認。

| 層 | 結果・証拠 |
| --- | --- |
| Rust実行テスト | core 16 / runtime 16成功。seed、オフライン一巡、次round、タッチとNative Choose、締切、不正入力、pause/復帰・世代変更・destroy後の遅延入力、workerのStatus公開を確認。`artifacts/rust-tests.log` |
| Rust静的検査 | `fmt --all -- --check`、対象2crateの`clippy --all-targets -- -D warnings`成功。`artifacts/rust-fmt.log`、`artifacts/rust-clippy.log` |
| Androidホスト | SDK35のandroid.jarで全Kotlinホストをコンパイルし、RecordingRuntimeを使うJUnit 4件成功。リソースIDはテスト用スタブ。APK生成・JNI実行ではない。`artifacts/android-purehost.log` |
| iOSソース | SDK26.5、arm64 iOS16 Simulator向けSwiftオブジェクト生成成功。Rustリンクなし。`artifacts/ios-source-check.log` |
| Swift文字表示 | macOSで`RuntimeStatusTests`実行成功。Probe、Selecting、Result、操作拒否、壊れたJSONを確認。iOS実行ではない。`artifacts/ios-status-test.log` |
| 両OSアプリビルド・端末・性能 | 今回は未実施・未確認。統合担当が実施する |

Rustは専用`artifacts/target-race-host`だけを使用し、debug情報とincrementalを無効にした。テストのビルドにはtoolchainの`rust-objcopy`が`libLLVM.dylib`を見つけられない警告があるが、32件成功しClippyも通過した。toolchain自体は変更していない。Androidのコンパイル警告は既存のInsets/Back APIの非推奨警告。

## 統合後のコマンドと端末確認

```sh
export CARGO_TARGET_DIR="$PWD/artifacts/target-race-host"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
scripts/cargo.sh test -p game-runtime -p game-core --locked
scripts/cargo.sh fmt --all -- --check
scripts/cargo.sh clippy -p game-runtime -p game-core --all-targets --locked -- -D warnings
scripts/ios-build.sh check-sources
xcrun swiftc -module-cache-path "$PWD/artifacts/swift-module-cache" \
  apps/ios/Sources/RuntimeStatus.swift apps/ios/Tests/RuntimeStatusTests.swift \
  -o artifacts/ios-status-test
artifacts/ios-status-test
```

アプリビルドは統合担当のビルド用targetと容量管理に従う。既存の`scripts/ios-build.sh simulator`はtargetを上書きするため、上記専用target設定だけでは分離できない。

1. 統合ブランチで`scripts/android-build.sh`、`scripts/ios-build.sh simulator`を実行する。
2. 両OSでProbeを開き、既存の開閉10回・背景復帰5回・サイズ変更・Surface/Layer再生成後の入力を再確認する。
3. Race → Waiting → Start → Selecting表示 → タッチまたはChoose → Close roundまたは自然締切 → Result → Start/nextで次roundへ進める。viewer 2のchoiceと結果人数をJSONでも確認する。
4. 選択中にSurface/Layer再生成、背景移行・復帰、Race/Probe往復を行い、同じroundとRoom状態が継続し、古い入力が次roundへ入らないことを確認する。
5. ネイティブUIの全操作・状態表示・描画領域が縦画面に収まり操作できることを実画面で確認する。Androidは描画領域を残り高さに配分し、iOSは高さの35%（Resize時25%）に制約、JSONはスクロール領域へ置いた。実画面の収まりは未確認。

オフライン一巡のEmulator/Simulator証拠、実機、性能、ネットワーク条件は未確認。Issueは閉じず、PoC完了としない。
