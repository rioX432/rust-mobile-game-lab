> **履歴の実装記録 (2026-10-24以前)。** 旧2D/3択オフライン技術PoCの証拠。ここに書かれた未着手・「次の段階」・旧Issue依存は現行開発指示ではない。SEED Race v1の現行方針は[PROJECT](PROJECT.md)、[plan](plan.md)。実行当時の結果・ソース等は変更せず保持する。

# オフラインレースのネイティブ接続

対象: [レース #3](https://github.com/rioX432/rust-mobile-game-lab/issues/3)、[Android #6](https://github.com/rioX432/rust-mobile-game-lab/issues/6)、[iOS #7](https://github.com/rioX432/rust-mobile-game-lab/issues/7)のオフライン部分。
統合ブランチは`feat/offline-race`。coreと両OSホストを統合し、Emulator/Simulatorで実行した。今回の完了範囲はオフラインレースまでで、PoC全体の完了ではない。

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
| Swift文字表示 | 下記のSimulator実操作と画像で確認。表示文言一致だけの補助テストは受入に採用せず、削除した |
| 統合検証 | Rust core16 / renderer4 / runtime16、format、strict Clippy、Android JUnit4 / lint / APK、iOS Simulatorアプリの実リンクビルドが成功。両OSの実操作は下記。実機・性能は未確認 |

Rustは専用`artifacts/target-race-host`だけを使用し、debug情報とincrementalを無効にした。テストのビルドにはtoolchainの`rust-objcopy`が`libLLVM.dylib`を見つけられない警告があるが、32件成功しClippyも通過した。toolchain自体は変更していない。Androidのコンパイル警告は既存のInsets/Back APIの非推奨警告。

## 統合後のコマンドと端末確認

```sh
export CARGO_TARGET_DIR="$PWD/artifacts/target-race-host"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
scripts/cargo.sh test -p game-runtime -p game-core --locked
scripts/cargo.sh fmt --all -- --check
scripts/cargo.sh clippy -p game-runtime -p game-core --all-targets --locked -- -D warnings
scripts/ios-build.sh check-sources
```

アプリビルドは統合担当のビルド用targetと容量管理に従う。既存の`scripts/ios-build.sh simulator`はtargetを上書きするため、上記専用target設定だけでは分離できない。

1. 統合ブランチで`scripts/android-build.sh`、`scripts/ios-build.sh simulator`を実行する。
2. 両OSでProbeを開き、既存の開閉10回・背景復帰5回・サイズ変更・Surface/Layer再生成後の入力を再確認する。
3. Race → Waiting → Start → Selecting表示 → タッチまたはChoose → Close roundまたは自然締切 → Result → Start/nextで次roundへ進める。viewer 2のchoiceと結果人数をJSONでも確認する。
4. 選択中にSurface/Layer再生成、背景移行・復帰、Race/Probe往復を行い、同じroundとRoom状態が継続し、古い入力が次roundへ入らないことを確認する。
5. ネイティブUIの全操作・状態表示・描画領域が縦画面に収まり操作できることを実画面で確認する。Androidは描画領域を残り高さに配分し、iOSは高さの35%（Resize時25%）に制約、JSONはスクロール領域へ置いた。実画面の収まりは未確認。

## 統合担当による実行結果

対象コードは`35a7354`のRust/core/両OSホスト。以後の変更は表示文言一致テストの除外と記録のみ。2026-10-24、専用Android Emulator（API35 arm64 / 1080×2340）とiPhone17 Simulator（iOS26.5 / scale3）で実行した。AndroidはDebug APK+Rust release、iOSはDebug app+Rust debug。実機性能の条件ではない。

- 両OSそれぞれ、画面を開く→RaceのWaiting/round0→閉じるを10回。各回で管理インスタンス・worker・NativeTargetが各1、閉じた後はゲーム領域が消えることを確認。
- 最終再オープン後、seed42のround1でRust領域の左レーンをタッチ。両OSとも`lane_order=[0,2,1] / counts=[1,0,0] / winners=[2]`でResult。
- Start/nextでround2を開始し、ネイティブChoose2から入力。両OSとも`counts=[0,1,0]`でResult。Probe/Race往復後も同じRoom/結果を保持。
- ResultでHome→アプリ復帰を5回。round/参加者/結果を保持し、Rustの停止中描画提出増分は各回0。各回の管理インスタンス・worker・NativeTargetは各1。
- Resize、iOSはさらにNew layerを実行。同じRoomを保持し、round3で再生成後の右レーンタッチが成功。両OSとも`lane_order=[0,1,2] / counts=[0,0,1] / winners=[]`でResult、GPU/ゲーム操作エラーなし。
- 両OSの上下ネイティブUI、描画領域、入力、結果表示が収まることを実画面の画像で確認した。

[Android実行画像](evidence/android-offline-race.png)、[iOS実行画像](evidence/ios-offline-race.png)、[検証記録](evidence/offline-mobile-acceptance.json)。iOSのアクセシビリティ取得値には長さ制限があり、長いResult JSONは途中までしか取得できない。記録の`snapshot_complete=false`は、実際に見えたJSON先頭と結果要約の項目だけを採取したことを表し、未取得の締切時刻等は補っていない。iOSの実行画像では、取得値から切れた時刻等を含む全JSONが画面に表示されている。

初回の個別ツール往復では3秒の受付を過ぎ、入力がInvalidPhaseとして拒否された。受付時間は変更せず、既知のStart/タッチ操作をbatchで送って再オープンから検証し直した。成功記録は上記の専用ファイルに保存。フレーム数はGPUへの提出カウンタであり、FPS/実提示間隔ではない。

未確認は動的な画面密度変更、実機、実GPU/OSハング、実Surface/Device Lostの故障注入、通信・投票・SDK最終取り込み・性能/サイズ測定。バックグラウンド5回のゲーム状態はResultで検証しており、全phaseの総当たりではない。未検証条件を持つIssueと親Issueは閉じず、PoC完了としない。
