# レース実装の共有契約

対象は[レース状態遷移 Issue #3](https://github.com/rioX432/rust-mobile-game-lab/issues/3)。この契約は次段階のAPIであり、ネイティブ描画プローブの動作を変更しない。

`game-core`の`game`モジュールにOS/GPU/通信なしで実装する。serde以外の依存を増やさず、時刻はすべて呼び出し引数で注入する。

```rust
pub type PlayerId = u64;
pub enum GameKind { Race } // 投票は主サンプル成立後に追加
pub enum RoundPhase { Waiting, Selecting, Animating, Result }
pub enum GameCommand { Start, Choose { option: u8 }, Close }
pub struct Player { pub id: PlayerId, pub choice: Option<u8> }
pub struct Outcome {
    pub lane_order: [u8; 3], // 先頭が勝者
    pub counts: [u32; 3],
    pub winners: Vec<PlayerId>,
}
pub struct Snapshot {
    pub game: GameKind,
    pub round_id: u64,
    pub revision: u64,
    pub phase: RoundPhase,
    pub seed: u64,
    pub players: Vec<Player>,
    pub result: Option<Outcome>,
    pub closes_at_ms: Option<u64>,
    pub animation_ends_at_ms: Option<u64>,
}
impl Room {
    pub fn new(game: GameKind, host: PlayerId, seed: u64) -> Self;
    pub fn join(&mut self, player: PlayerId) -> Result<(), GameError>;
    pub fn apply(&mut self, actor: PlayerId, round_id: u64,
                 command: GameCommand, now_ms: u64) -> Result<(), GameError>;
    pub fn advance(&mut self, now_ms: u64) -> bool;
    pub fn snapshot(&self) -> Snapshot;
}
pub fn scene(snapshot: &Snapshot, local_player: PlayerId,
             width: u32, height: u32, scale: f32, now_ms: u64) -> crate::Scene;
```

- ホストを最初の参加者として登録する。ホストを含め20人まで。重複joinは人数やrevisionを増やさない。
- WaitingまたはResultからホストだけがStartでき、round_idを増やす。受付3秒、演出1.8秒を初期値とし、整数msで管理する。
- 入力は登録済み参加者の0〜2のみ。受付中の最新選択を使い、同じ選択の繰り返しで状態を重複加算しない。旧round_id・締切ちょうど以後・演出/結果中の入力は拒否する。
- Closeはホストだけが受付中に実行できる。自然締切はadvanceが処理する。時刻が演出終了を飛び越えた場合も1回のadvanceでResultへ進む。
- Roomが状態と結果の正本。seedとラウンド番号に基づく明示した整数演算でレーン順位を確定し、同じseed・入力列から再現できる。floatや描画結果で勝敗を決めない。
- Snapshotは安定した参加者順、単調増加revisionを持つ。join/受付/結果/次ラウンドの途中状態を保持できる。applyは入力前に時刻を進めてよく、入力が拒否されても時間による状態遷移は起こり得る。
- sceneはSnapshotを表示用データへ変換するだけ。演出の補間が状態・結果・revisionを変更しない。
- `GameError`の具体的なvariantは担当実装で確定し、拒否理由が区別できるよう文書化する。通信のcommand_id重複処理は次のサーバー層が所有する。
