use serde::{Deserialize, Serialize};

use crate::{Scene, Shape, ShapeKind};

const SELECTION_MS: u64 = 3_000;
const ANIMATION_MS: u64 = 1_800;
const MAX_PLAYERS: usize = 20;

pub type PlayerId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameKind {
    Race,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundPhase {
    Waiting,
    Selecting,
    Animating,
    Result,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameCommand {
    Start,
    Choose { option: u8 },
    Close,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub choice: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub lane_order: [u8; 3],
    pub counts: [u32; 3],
    pub winners: Vec<PlayerId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameError {
    RoomFull,
    UnknownPlayer,
    HostOnly,
    WrongRound,
    InvalidPhase,
    InvalidOption,
    TimeOverflow,
    RoundIdExhausted,
}

#[derive(Debug)]
pub struct Room {
    host: PlayerId,
    state: Snapshot,
    last_now_ms: u64,
}

impl Room {
    pub fn new(game: GameKind, host: PlayerId, seed: u64) -> Self {
        Self {
            host,
            state: Snapshot {
                game,
                round_id: 0,
                revision: 0,
                phase: RoundPhase::Waiting,
                seed,
                players: vec![Player {
                    id: host,
                    choice: None,
                }],
                result: None,
                closes_at_ms: None,
                animation_ends_at_ms: None,
            },
            last_now_ms: 0,
        }
    }

    pub fn join(&mut self, player: PlayerId) -> Result<(), GameError> {
        if self.state.players.iter().any(|p| p.id == player) {
            return Ok(());
        }
        if self.state.players.len() >= MAX_PLAYERS {
            return Err(GameError::RoomFull);
        }
        self.state.players.push(Player {
            id: player,
            choice: None,
        });
        self.state.revision += 1;
        Ok(())
    }

    pub fn apply(
        &mut self,
        actor: PlayerId,
        round_id: u64,
        command: GameCommand,
        now_ms: u64,
    ) -> Result<(), GameError> {
        self.advance(now_ms);
        let now_ms = self.last_now_ms;
        if round_id != self.state.round_id {
            return Err(GameError::WrongRound);
        }
        let index = self
            .state
            .players
            .iter()
            .position(|p| p.id == actor)
            .ok_or(GameError::UnknownPlayer)?;
        match command {
            GameCommand::Start => {
                if actor != self.host {
                    return Err(GameError::HostOnly);
                }
                if !matches!(self.state.phase, RoundPhase::Waiting | RoundPhase::Result) {
                    return Err(GameError::InvalidPhase);
                }
                let round = self
                    .state
                    .round_id
                    .checked_add(1)
                    .ok_or(GameError::RoundIdExhausted)?;
                // Reserve the complete timeline so automatic closure cannot overflow.
                now_ms
                    .checked_add(SELECTION_MS + ANIMATION_MS)
                    .ok_or(GameError::TimeOverflow)?;
                self.state.round_id = round;
                self.state.phase = RoundPhase::Selecting;
                self.state.result = None;
                self.state.closes_at_ms = Some(now_ms + SELECTION_MS);
                self.state.animation_ends_at_ms = None;
                for player in &mut self.state.players {
                    player.choice = None;
                }
                self.state.revision += 1;
            }
            GameCommand::Choose { option } => {
                if self.state.phase != RoundPhase::Selecting {
                    return Err(GameError::InvalidPhase);
                }
                if option > 2 {
                    return Err(GameError::InvalidOption);
                }
                if self.state.players[index].choice != Some(option) {
                    self.state.players[index].choice = Some(option);
                    self.state.revision += 1;
                }
            }
            GameCommand::Close => {
                if actor != self.host {
                    return Err(GameError::HostOnly);
                }
                if self.state.phase != RoundPhase::Selecting {
                    return Err(GameError::InvalidPhase);
                }
                self.close(now_ms);
            }
        }
        Ok(())
    }

    pub fn advance(&mut self, now_ms: u64) -> bool {
        self.last_now_ms = self.last_now_ms.max(now_ms);
        let before = self.state.revision;
        if self.state.phase == RoundPhase::Selecting
            && let Some(deadline) = self.state.closes_at_ms
            && self.last_now_ms >= deadline
        {
            // Use the scheduled deadline even when the caller skips several frames.
            self.close(deadline);
        }
        if self.state.phase == RoundPhase::Animating
            && let Some(end) = self.state.animation_ends_at_ms
            && self.last_now_ms >= end
        {
            self.state.phase = RoundPhase::Result;
            self.state.revision += 1;
        }
        before != self.state.revision
    }

    pub fn snapshot(&self) -> Snapshot {
        self.state.clone()
    }

    fn close(&mut self, now_ms: u64) {
        let lane_order = lane_order(self.state.seed, self.state.round_id);
        let mut counts = [0; 3];
        let mut winners = Vec::new();
        for player in &self.state.players {
            if let Some(choice) = player.choice {
                counts[choice as usize] += 1;
                if choice == lane_order[0] {
                    winners.push(player.id);
                }
            }
        }
        self.state.result = Some(Outcome {
            lane_order,
            counts,
            winners,
        });
        self.state.phase = RoundPhase::Animating;
        self.state.animation_ends_at_ms = Some(now_ms + ANIMATION_MS);
        self.state.revision += 1;
    }
}

fn lane_order(seed: u64, round_id: u64) -> [u8; 3] {
    let mut value = seed.wrapping_add(round_id.wrapping_mul(0x9e3779b97f4a7c15));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^= value >> 31;
    [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ][(value % 6) as usize]
}

pub fn scene(
    snapshot: &Snapshot,
    local_player: PlayerId,
    width: u32,
    height: u32,
    scale: f32,
    now_ms: u64,
) -> Scene {
    let scale = if scale.is_finite()
        && scale > 0.0
        && (width as f32 / scale).is_finite()
        && (height as f32 / scale).is_finite()
    {
        scale
    } else {
        1.0
    };
    let w = width as f32 / scale;
    let h = height as f32 / scale;
    let choice = snapshot
        .players
        .iter()
        .find(|p| p.id == local_player)
        .and_then(|p| p.choice);
    let progress = match snapshot.phase {
        RoundPhase::Animating => snapshot.animation_ends_at_ms.map_or(0.0, |end| {
            now_ms
                .saturating_sub(end.saturating_sub(ANIMATION_MS))
                .min(ANIMATION_MS) as f32
                / ANIMATION_MS as f32
        }),
        RoundPhase::Result => 1.0,
        _ => 0.0,
    };
    let mut shapes = Vec::new();
    for lane in 0..3u8 {
        shapes.push(Shape {
            kind: ShapeKind::Rectangle,
            bounds: [(lane as f32 / 3.0) * w, 0.0, w / 3.0, h],
            color: if choice == Some(lane) {
                [0.15, 0.45, 0.55, 1.0]
            } else {
                [0.12, 0.16, 0.25, 1.0]
            },
        });
        let rank = snapshot
            .result
            .as_ref()
            .and_then(|result| result.lane_order.iter().position(|&item| item == lane))
            .unwrap_or(0);
        shapes.push(Shape {
            kind: ShapeKind::Circle,
            bounds: [
                ((lane as f32 + 0.5) / 3.0) * w,
                h * (0.85 - progress * (0.70 - rank as f32 * 0.10)),
                w.min(h) / 14.0,
                0.0,
            ],
            color: [
                [0.98, 0.55, 0.38, 1.0],
                [0.48, 0.85, 0.70, 1.0],
                [0.70, 0.60, 0.98, 1.0],
            ][lane as usize],
        });
    }
    Scene {
        size: [w, h],
        shapes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selecting() -> Room {
        let mut room = Room::new(GameKind::Race, 10, 42);
        room.join(20).unwrap();
        room.apply(10, 0, GameCommand::Start, 100).unwrap();
        room
    }

    #[test]
    fn host_is_registered_and_duplicate_join_is_idempotent_even_when_full() {
        let mut room = Room::new(GameKind::Race, 10, 42);
        let initial = room.snapshot();
        assert_eq!(initial.phase, RoundPhase::Waiting);
        assert_eq!(initial.round_id, 0);
        assert_eq!(
            initial.players,
            vec![Player {
                id: 10,
                choice: None
            }]
        );
        assert_eq!(initial.result, None);
        assert_eq!(initial.closes_at_ms, None);
        assert_eq!(initial.animation_ends_at_ms, None);
        room.join(10).unwrap();
        assert_eq!(room.snapshot(), initial);
        for id in 11..30 {
            room.join(id).unwrap();
        }
        let full = room.snapshot();
        assert_eq!(
            full.players.iter().map(|p| p.id).collect::<Vec<_>>(),
            (10..30).collect::<Vec<_>>()
        );
        assert_eq!(full.revision, 19);
        room.join(20).unwrap();
        assert_eq!(room.join(30), Err(GameError::RoomFull));
        assert_eq!(room.snapshot(), full);
    }

    #[test]
    fn host_privileges_membership_round_and_phase_are_enforced() {
        let mut room = Room::new(GameKind::Race, 10, 42);
        room.join(20).unwrap();
        for command in [GameCommand::Start, GameCommand::Close] {
            let before = room.snapshot();
            assert_eq!(room.apply(20, 0, command, 0), Err(GameError::HostOnly));
            assert_eq!(room.apply(99, 0, command, 0), Err(GameError::UnknownPlayer));
            assert_eq!(room.snapshot(), before);
        }
        assert_eq!(
            room.apply(10, 0, GameCommand::Close, 0),
            Err(GameError::InvalidPhase)
        );
        assert_eq!(
            room.apply(20, 0, GameCommand::Choose { option: 0 }, 0),
            Err(GameError::InvalidPhase)
        );
        room.apply(10, 0, GameCommand::Start, 100).unwrap();
        let before = room.snapshot();
        assert_eq!(
            room.apply(10, 1, GameCommand::Start, 100),
            Err(GameError::InvalidPhase)
        );
        assert_eq!(
            room.apply(20, 1, GameCommand::Close, 100),
            Err(GameError::HostOnly)
        );
        assert_eq!(
            room.apply(99, 1, GameCommand::Choose { option: 0 }, 100),
            Err(GameError::UnknownPlayer)
        );
        assert_eq!(
            room.apply(20, 0, GameCommand::Choose { option: 0 }, 100),
            Err(GameError::WrongRound)
        );
        assert_eq!(
            room.apply(10, 2, GameCommand::Close, 100),
            Err(GameError::WrongRound)
        );
        assert_eq!(room.snapshot(), before);
    }

    #[test]
    fn latest_choice_wins_and_repeated_choice_does_not_change_revision() {
        let mut room = selecting();
        let before = room.snapshot();
        for option in [3, u8::MAX] {
            assert_eq!(
                room.apply(20, 1, GameCommand::Choose { option }, 101),
                Err(GameError::InvalidOption)
            );
        }
        assert_eq!(room.snapshot(), before);
        room.apply(20, 1, GameCommand::Choose { option: 1 }, 102)
            .unwrap();
        let once = room.snapshot();
        room.apply(20, 1, GameCommand::Choose { option: 1 }, 103)
            .unwrap();
        assert_eq!(room.snapshot(), once);
        room.apply(20, 1, GameCommand::Choose { option: 2 }, 104)
            .unwrap();
        assert_eq!(room.snapshot().revision, once.revision + 1);
        room.apply(10, 1, GameCommand::Close, 105).unwrap();
        assert_eq!(room.snapshot().result.unwrap().counts, [0, 0, 1]);
    }

    #[test]
    fn deadline_and_animation_end_are_inclusive_boundaries() {
        let mut room = selecting();
        assert_eq!(room.snapshot().closes_at_ms, Some(3_100));
        assert!(!room.advance(3_099));
        room.apply(20, 1, GameCommand::Choose { option: 0 }, 3_099)
            .unwrap();
        let before = room.snapshot();
        assert_eq!(
            room.apply(20, 1, GameCommand::Choose { option: 2 }, 3_100),
            Err(GameError::InvalidPhase)
        );
        let closed = room.snapshot();
        assert_eq!(closed.phase, RoundPhase::Animating);
        assert_eq!(closed.revision, before.revision + 1);
        assert_eq!(closed.animation_ends_at_ms, Some(4_900));
        assert_eq!(closed.players[1].choice, Some(0));
        assert_eq!(closed.result.as_ref().unwrap().counts, [1, 0, 0]);
        assert_eq!(
            room.apply(10, 1, GameCommand::Close, 3_101),
            Err(GameError::InvalidPhase)
        );
        assert_eq!(
            room.apply(10, 1, GameCommand::Start, 3_101),
            Err(GameError::InvalidPhase)
        );
        assert!(!room.advance(4_899));
        assert!(room.advance(4_900));
        let result = room.snapshot();
        assert_eq!(result.phase, RoundPhase::Result);
        assert_eq!(result.result, closed.result);
        assert_eq!(
            room.apply(20, 1, GameCommand::Choose { option: 0 }, 4_900),
            Err(GameError::InvalidPhase)
        );
        assert!(!room.advance(50_000));
        assert_eq!(room.snapshot(), result);
    }

    #[test]
    fn one_advance_can_skip_both_timed_phases_without_extending_deadline() {
        let mut room = selecting();
        let before = room.snapshot();
        assert!(room.advance(50_000));
        let after = room.snapshot();
        assert_eq!(after.phase, RoundPhase::Result);
        assert_eq!(after.animation_ends_at_ms, Some(4_900));
        assert_eq!(after.revision, before.revision + 2);
        assert_eq!(after.result.unwrap().counts, [0, 0, 0]);
    }

    #[test]
    fn manual_close_starts_animation_at_host_time_and_clock_cannot_reopen_input() {
        let mut room = selecting();
        assert!(!room.advance(500));
        room.apply(10, 1, GameCommand::Close, 400).unwrap();
        let closed = room.snapshot();
        assert_eq!(closed.phase, RoundPhase::Animating);
        assert_eq!(closed.animation_ends_at_ms, Some(2_300));
        assert!(!room.advance(200));
        assert_eq!(
            room.apply(20, 1, GameCommand::Choose { option: 1 }, 200),
            Err(GameError::InvalidPhase)
        );
        assert_eq!(room.snapshot(), closed);
        assert!(!room.advance(2_299));
        assert!(room.advance(2_300));
    }

    #[test]
    fn late_join_keeps_result_frozen_and_next_round_resets_all_choices() {
        let mut room = selecting();
        room.join(30).unwrap();
        room.apply(30, 1, GameCommand::Choose { option: 0 }, 200)
            .unwrap();
        room.apply(10, 1, GameCommand::Close, 300).unwrap();
        let closed = room.snapshot();
        room.join(40).unwrap();
        assert_eq!(room.snapshot().result, closed.result);
        assert_eq!(
            room.snapshot().players[3],
            Player {
                id: 40,
                choice: None
            }
        );
        assert_eq!(
            room.apply(40, 1, GameCommand::Choose { option: 0 }, 301),
            Err(GameError::InvalidPhase)
        );
        room.advance(2_100);
        room.join(50).unwrap();
        let result = room.snapshot();
        assert_eq!(result.result.as_ref().unwrap().winners, vec![30]);
        let mut detached = result.clone();
        detached.players.clear();
        detached.result = None;
        assert_eq!(room.snapshot(), result);
        room.apply(10, 1, GameCommand::Start, 2_200).unwrap();
        let next = room.snapshot();
        assert_eq!(next.phase, RoundPhase::Selecting);
        assert_eq!(next.round_id, 2);
        assert_eq!(next.revision, result.revision + 1);
        assert_eq!(next.result, None);
        assert_eq!(next.animation_ends_at_ms, None);
        assert_eq!(next.closes_at_ms, Some(5_200));
        assert!(next.players.iter().all(|p| p.choice.is_none()));
        assert_eq!(
            next.players.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![10, 20, 30, 40, 50]
        );
        assert_eq!(
            room.apply(30, 1, GameCommand::Choose { option: 1 }, 2_201),
            Err(GameError::WrongRound)
        );
        assert_eq!(
            room.apply(10, 1, GameCommand::Start, 2_201),
            Err(GameError::WrongRound)
        );
        assert_eq!(room.snapshot(), next);
        room.apply(40, 2, GameCommand::Choose { option: 1 }, 2_202)
            .unwrap();
        assert_eq!(room.snapshot().players[3].choice, Some(1));
    }

    fn replay(seed: u64) -> Snapshot {
        let mut room = Room::new(GameKind::Race, 10, seed);
        for id in [20, 30, 40] {
            room.join(id).unwrap();
        }
        room.apply(10, 0, GameCommand::Start, 100).unwrap();
        for (actor, option, at) in [(10, 0, 200), (20, 1, 300), (30, 0, 400), (20, 2, 500)] {
            room.apply(actor, 1, GameCommand::Choose { option }, at)
                .unwrap();
        }
        room.advance(4_900);
        room.snapshot()
    }

    #[test]
    fn seed_and_input_fixture_replays_to_known_result_in_join_order() {
        let first = replay(42);
        assert_eq!(first, replay(42));
        assert_eq!(
            first.result,
            Some(Outcome {
                lane_order: [0, 2, 1],
                counts: [2, 0, 1],
                winners: vec![10, 30],
            })
        );
        assert_eq!(first.revision, 10);
        assert_eq!(
            replay(u64::MAX).result,
            Some(Outcome {
                lane_order: [1, 0, 2],
                counts: [2, 0, 1],
                winners: vec![],
            })
        );
    }

    #[test]
    fn twenty_synthetic_players_are_counted_once_per_latest_choice() {
        let mut room = Room::new(GameKind::Race, 0, 42);
        for id in 1..20 {
            room.join(id).unwrap();
        }
        room.apply(0, 0, GameCommand::Start, 0).unwrap();
        for id in 0..20 {
            for _ in 0..2 {
                room.apply(
                    id,
                    1,
                    GameCommand::Choose {
                        option: (id % 3) as u8,
                    },
                    1,
                )
                .unwrap();
            }
        }
        room.advance(4_800);
        assert_eq!(
            room.snapshot().result,
            Some(Outcome {
                lane_order: [0, 2, 1],
                counts: [7, 7, 6],
                winners: vec![0, 3, 6, 9, 12, 15, 18],
            })
        );
    }

    #[test]
    fn round_number_changes_the_seeded_order_without_changing_room_seed() {
        let mut room = Room::new(GameKind::Race, 10, u64::MAX);
        for (round, order) in [(1, [1, 0, 2]), (2, [1, 2, 0])] {
            let start = (round - 1) * 5_000;
            room.apply(10, round - 1, GameCommand::Start, start)
                .unwrap();
            room.advance(start + 4_800);
            let snapshot = room.snapshot();
            assert_eq!(snapshot.phase, RoundPhase::Result);
            assert_eq!(snapshot.round_id, round);
            assert_eq!(snapshot.seed, u64::MAX);
            assert_eq!(snapshot.result.unwrap().lane_order, order);
        }
    }

    #[test]
    fn no_selections_produces_empty_counts_and_winners() {
        let mut room = Room::new(GameKind::Race, 0, 0);
        room.apply(0, 0, GameCommand::Start, 0).unwrap();
        room.advance(4_800);
        assert_eq!(
            room.snapshot().result,
            Some(Outcome {
                lane_order: [0, 2, 1],
                counts: [0, 0, 0],
                winners: vec![],
            })
        );
    }

    #[test]
    fn timeline_overflow_is_rejected_and_largest_valid_timeline_completes() {
        let mut room = Room::new(GameKind::Race, 0, 42);
        let before = room.snapshot();
        assert_eq!(
            room.apply(0, 0, GameCommand::Start, u64::MAX - 4_799),
            Err(GameError::TimeOverflow)
        );
        assert_eq!(room.snapshot(), before);
        let mut room = Room::new(GameKind::Race, 0, 42);
        room.apply(0, 0, GameCommand::Start, u64::MAX - 4_800)
            .unwrap();
        assert!(room.advance(u64::MAX));
        assert_eq!(room.snapshot().phase, RoundPhase::Result);
        assert_eq!(room.snapshot().animation_ends_at_ms, Some(u64::MAX));
    }

    #[test]
    fn display_interpolates_and_scales_without_mutating_authoritative_snapshot() {
        let mut room = selecting();
        room.apply(20, 1, GameCommand::Choose { option: 0 }, 200)
            .unwrap();
        room.apply(10, 1, GameCommand::Close, 300).unwrap();
        let snapshot = room.snapshot();
        let start = scene(&snapshot, 20, 300, 600, 1.0, 300);
        let middle = scene(&snapshot, 20, 300, 600, 1.0, 1_200);
        let end = scene(&snapshot, 20, 300, 600, 1.0, 2_100);
        assert_ne!(start, middle);
        assert_ne!(middle, end);
        assert_eq!(end, scene(&snapshot, 20, 300, 600, 1.0, u64::MAX));
        assert_eq!(start, scene(&snapshot, 20, 600, 1_200, 2.0, 300));
        assert_ne!(start, scene(&snapshot, 99, 300, 600, 1.0, 300));
        for scale in [f32::NAN, f32::INFINITY, 0.0, -1.0, f32::from_bits(1)] {
            assert_eq!(start, scene(&snapshot, 20, 300, 600, scale, 300));
        }
        assert_eq!(scene(&snapshot, 20, 0, 0, 1.0, 300).size, [0.0, 0.0]);
        let large = scene(&snapshot, 20, 300, 600, 2e-36, 300);
        assert!(large.size.iter().all(|value| value.is_finite()));
        assert!(
            large
                .shapes
                .iter()
                .all(|shape| shape.bounds.iter().all(|value| value.is_finite()))
        );
        // Lane zero is the fixture winner and must be furthest along at the end.
        assert!(end.shapes[1].bounds[1] < end.shapes[3].bounds[1]);
        assert!(end.shapes[1].bounds[1] < end.shapes[5].bounds[1]);
        assert_eq!(room.snapshot(), snapshot);
    }
}
