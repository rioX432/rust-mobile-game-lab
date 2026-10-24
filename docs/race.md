# 純粋処理の3レーンレース

[Issue #3](https://github.com/rioX432/rust-mobile-game-lab/issues/3)のオフラインコア。APIは[共有契約](race-contract.md)に従う。OS・GPU・ネットワーク・実時計に依存せず、追加依存はない。投票は含まない。図形は既存の`Scene`へ矩形と円で渡し、他作品の画像・コードは使っていない。

## 公開API

`game_core::game`で以下を公開する。

- `PlayerId = u64`
- `GameKind::Race`
- `RoundPhase::{Waiting, Selecting, Animating, Result}`
- `GameCommand::{Start, Choose { option: u8 }, Close}`
- `Player { id: PlayerId, choice: Option<u8> }`
- `Outcome { lane_order: [u8; 3], counts: [u32; 3], winners: Vec<PlayerId> }`
- `Snapshot { game, round_id, revision, phase, seed, players, result, closes_at_ms, animation_ends_at_ms }`。各フィールドの型・公開範囲は共有契約と同じ。
- `Room::new(game, host, seed)`、`join(player)`、`apply(actor, round_id, command, now_ms)`、`advance(now_ms) -> bool`、`snapshot() -> Snapshot`
- `scene(&Snapshot, local_player, width, height, scale, now_ms) -> Scene`

コマンド・スナップショット・結果・エラーと、それらが含む値の型は`Clone/Debug/PartialEq/Eq/Serialize/Deserialize`を持つ。Roomの内部は非公開。snapshotは独立したコピーであり、書き換えてもRoomへ反映されない。SnapshotからRoomを再構築するAPIはこの契約に含まない。

## 時刻と状態

最初は`round_id=0`、`revision=0`、ホストのみ登録済みのWaiting。ホストを含め20人まで、参加順を保持する。全フェーズでjoinでき、同じIDの再joinは満員でも成功し、revisionを増やさない。

Startは現在のround_idを指定する。Waiting/Resultからホストが開始するとround_idとrevisionが1増え、選択と前回結果を消し、Selectingへ進む。受付期限は開始時刻+3,000ms。未選択者は集計せず、ホストの選択も必須ではない。受付中の途中参加者も選択できる。同選択の再送は成功して状態を変えず、選び直しは最新選択で置き換える。

自然締切は`now_ms >= closes_at_ms`。applyは検証前にadvanceするため、締切ちょうどのChooseは`InvalidPhase`で拒否するが、時間による状態遷移は残る。Closeはホストが受付中に早期締切するときだけ成功する。締切時に結果を整数演算で固定し、Animatingへ進む。演出終了は締切時刻+1,800ms。早期Closeでは呼出時刻を使い、自然締切では予定期限を使う。closes_at_msは早期Close後も元の予定期限を保持する。

`advance`は観測可能な状態が変わったときだけtrueを返す。自然締切と演出終了を飛び越す時刻なら、1回でResultまで進む。revisionはjoin・Start・選択変更・締切・結果表示ごとに1増える。2段階を一度に進める場合は2増える。結果確定後の途中参加はplayersだけを追加し、counts/winnersを変更しない。次ラウンドでは全参加者が選択できる。

時刻はRoomごとに同じ基準の整数msで注入する。直前に観測した時刻より小さい値は直前値へ丸め、時間を巻き戻さない。拒否された入力の時刻も観測時刻として保持する。Startでは受付と演出の全期間をu64で表せることを確認する。sleepや実時計の取得はない。

## 拒否理由

| GameError | 条件 |
|---|---|
| `RoomFull` | 未登録IDのjoinで20人を超える |
| `WrongRound` | コマンドのround_idが現在値と異なる（旧・未来とも拒否） |
| `UnknownPlayer` | コマンドのactorが未登録 |
| `HostOnly` | 登録済みの非ホストがStart/Closeする |
| `InvalidPhase` | フェーズ上許されない操作。締切ちょうど以後のChooseを含む |
| `InvalidOption` | Selecting中のChooseが0〜2以外 |
| `TimeOverflow` | Start時刻+4,800msをu64で表せない |
| `RoundIdExhausted` | Startでround_idを増やせない |

applyの判定順は時刻更新→round_id→参加者→コマンドごとの権限→フェーズ→選択範囲または数値上限。拒否時のエラーはこの順序で1件返す。時間による遷移以外は拒否によってSnapshotを変えない。通信のcommand_id重複排除・認証・クライアント時計の検証は後続サーバー層の責務。

## 結果の再現規則

レーン順位はseedとラウンド番号だけで決まり、参加者の選択はcountsとwinnersを決める。次の演算はすべてu64で、加算・乗算は2^64を法として折り返す。

```text
v = seed + round_id * 0x9e3779b97f4a7c15
v = (v XOR (v >> 30)) * 0xbf58476d1ce4e5b9
v = (v XOR (v >> 27)) * 0x94d049bb133111eb
v = v XOR (v >> 31)
```

`v % 6`を`[[0,1,2], [0,2,1], [1,0,2], [1,2,0], [2,0,1], [2,1,0]]`の添字とする。先頭レーンが勝者。countsはレーン0/1/2の最新選択人数、winnersは勝者を選んだ参加者IDを参加順で並べる。これは再現性のための規則であり、暗号学的乱数や厳密な等確率を保証するものではない。異なるseedやラウンドが同じ順位になる場合もある。

sceneのfloatは演出の補間・座標変換だけで使う。Snapshotのphase/result/revisionには書き込まない。ローカル参加者の選択レーンを強調し、結果の順位に合わせた円の移動を返す。width/heightは物理ピクセル、scaleで論理ピクセルへ変換する。不正なscale、または変換が有限値にならないscaleは1へ戻す。

## 実行と確認結果

リポジトリのルートで実行する。対象はcoreのみ。

```sh
scripts/cargo.sh test -p game-core --locked
scripts/cargo.sh run -p game-core --example offline_race --locked
scripts/cargo.sh clippy -p game-core --all-targets --locked -- -D warnings
scripts/cargo.sh fmt -p game-core -- --check
```

2026-10-24の確認: テストはレース13件・既存3件が成功。ホスト権限、未登録者、不正レーン、旧/未来round、同選択再送、締切/演出終了の境界、時間の飛び越し・巻き戻し、途中参加、次ラウンド、20人の合成入力、無選択、時刻上限、固定結果の再現、表示補間でSnapshotが変わらないことを公開APIで検証する。

offline例はseed=42、host=10、join=[20,30,40]。入力はStart(100ms)、10→0(200ms)、20→1(300ms)、30→0(400ms)、20→2(500ms)。3,100msで締切、4,900msでResult、5,000msで次ラウンドを開始した。観測値:

```text
round=1 revision=10 phase=Result
lane_order=[0,2,1] counts=[2,0,1] winners=[10,30]
next round=2 revision=11 phase=Selecting choices=[None,None,None,None]
```

実行ログは`artifacts/core-test.log`、`artifacts/offline-race.log`、`artifacts/core-clippy.log`。実効権限は`artifacts/worker-permissions.txt`。artifactsはGit管理対象外。固定結果はテストとexampleにも残している。

モバイル組込み、GPUでの実表示・タッチ、通信、Emulator/Simulator、実機、実機性能はこの実装では未確認。コアの成功で代用しない。root manifest・Cargo.lock・runtime・renderer・ネイティブ・共有文書は変更していない。
