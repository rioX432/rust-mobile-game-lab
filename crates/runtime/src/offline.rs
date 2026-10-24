use game_core::{
    Probe, Scene,
    game::{self, GameCommand, GameKind, Room, Snapshot},
};

pub(crate) const HOST: u64 = 1;
pub(crate) const VIEWER: u64 = 2;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Action {
    Reset,
    Mode { race: bool },
    Start,
    Choose { option: u8 },
    Close,
}

impl Action {
    pub(crate) fn parse(action: u32, argument: u32) -> Result<Self, i32> {
        match (action, argument) {
            (0, _) => Ok(Self::Reset),
            (1, 0..=1) => Ok(Self::Mode {
                race: argument == 1,
            }),
            (2, 0) => Ok(Self::Start),
            (3, 0..=2) => Ok(Self::Choose {
                option: argument as u8,
            }),
            (4, 0) => Ok(Self::Close),
            _ => Err(crate::INVALID_ARGUMENT),
        }
    }
}

pub(crate) struct Offline {
    pub(crate) probe: Probe,
    room: Room,
    race: bool,
    pub(crate) last_action_error: Option<String>,
}

impl Offline {
    pub(crate) fn new(seed: u64) -> Self {
        let mut room = Room::new(GameKind::Race, HOST, seed);
        room.join(VIEWER).expect("the offline room has two players");
        Self {
            probe: Probe::default(),
            room,
            race: false,
            last_action_error: None,
        }
    }

    pub(crate) fn snapshot(&self) -> Option<Snapshot> {
        self.race.then(|| self.room.snapshot())
    }

    pub(crate) fn advance(&mut self, now_ms: u64) {
        self.room.advance(now_ms);
    }

    pub(crate) fn apply(&mut self, action: Action, round_id: Option<u64>, now_ms: u64) {
        let command = match action {
            Action::Reset => {
                self.probe.selected_lane = None;
                self.last_action_error = None;
                return;
            }
            Action::Mode { race } => {
                self.race = race;
                self.last_action_error = None;
                return;
            }
            Action::Start => (HOST, GameCommand::Start),
            Action::Choose { option } => (VIEWER, GameCommand::Choose { option }),
            Action::Close => (HOST, GameCommand::Close),
        };
        if !self.race {
            self.last_action_error = Some("Race mode is not selected".into());
            return;
        }
        // Choices keep their observed round so queued input cannot enter a later round.
        let round_id = if matches!(action, Action::Choose { .. }) {
            let Some(round_id) = round_id else {
                self.last_action_error = Some("Race input has no observed round".into());
                return;
            };
            round_id
        } else {
            self.room.snapshot().round_id
        };
        self.last_action_error = self
            .room
            .apply(command.0, round_id, command.1, now_ms)
            .err()
            .map(|error| format!("{error:?}"));
    }

    pub(crate) fn touch(
        &mut self,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        round_id: Option<u64>,
        now_ms: u64,
    ) {
        if self.probe.touch(x, y, width as f32, height as f32) && self.race {
            let option = self
                .probe
                .selected_lane
                .expect("valid touch selects a lane") as u8;
            self.apply(Action::Choose { option }, round_id, now_ms);
        }
    }

    pub(crate) fn scene(&self, width: u32, height: u32, scale: f32, now_ms: u64) -> Scene {
        if let Some(snapshot) = self.snapshot() {
            game::scene(&snapshot, VIEWER, width, height, scale, now_ms)
        } else {
            self.probe.scene(width, height, scale)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::game::RoundPhase;

    fn race(seed: u64) -> Offline {
        let mut offline = Offline::new(seed);
        assert!(offline.snapshot().is_none());
        offline.apply(Action::Mode { race: true }, None, 0);
        offline
    }

    #[test]
    fn offline_round_uses_host_commands_viewer_inputs_and_the_supplied_seed() {
        let mut offline = race(42);
        let initial = offline.snapshot().unwrap();
        assert_eq!(initial.seed, 42);
        assert_eq!(initial.phase, RoundPhase::Waiting);
        assert_eq!(
            initial.players.iter().map(|p| p.id).collect::<Vec<_>>(),
            [HOST, VIEWER]
        );
        offline.apply(Action::Start, None, 10);
        offline.touch(250.0, 100.0, 300, 400, Some(1), 20);
        offline.apply(Action::Choose { option: 0 }, Some(1), 30);
        let selecting = offline.snapshot().unwrap();
        assert_eq!(selecting.players[0].choice, None);
        assert_eq!(selecting.players[1].choice, Some(0));
        offline.apply(Action::Close, None, 40);
        assert!(offline.last_action_error.is_none());
        assert_eq!(offline.snapshot().unwrap().phase, RoundPhase::Animating);
        offline.advance(1_840);
        let result = offline.snapshot().unwrap();
        assert_eq!(result.phase, RoundPhase::Result);
        let outcome = result.result.unwrap();
        assert_eq!(outcome.counts, [1, 0, 0]);
        assert_eq!(outcome.lane_order, [0, 2, 1]);
        assert_eq!(outcome.winners, [VIEWER]);
        offline.apply(Action::Start, None, 1_850);
        let next = offline.snapshot().unwrap();
        assert_eq!(next.round_id, 2);
        assert_eq!(next.phase, RoundPhase::Selecting);
        assert!(next.result.is_none());
        assert!(next.players.iter().all(|p| p.choice.is_none()));
    }

    #[test]
    fn seed_and_inputs_reproduce_the_core_result_without_rendering_deciding_it() {
        for seed in [0, 1, 42, u64::MAX] {
            let mut offline = race(seed);
            let mut room = Room::new(GameKind::Race, HOST, seed);
            room.join(VIEWER).unwrap();
            offline.apply(Action::Start, None, 0);
            room.apply(HOST, 0, GameCommand::Start, 0).unwrap();
            offline.apply(Action::Choose { option: 2 }, Some(1), 100);
            room.apply(VIEWER, 1, GameCommand::Choose { option: 2 }, 100)
                .unwrap();
            let before_scene = offline.snapshot().unwrap();
            assert_eq!(
                offline.scene(600, 800, 2.0, 200),
                game::scene(&before_scene, VIEWER, 600, 800, 2.0, 200)
            );
            assert_eq!(offline.snapshot().unwrap(), before_scene);
            offline.advance(90_000);
            room.advance(90_000);
            assert_eq!(offline.snapshot().unwrap(), room.snapshot());
        }
    }

    #[test]
    fn mode_and_probe_reset_preserve_room_while_background_time_advances() {
        let mut offline = race(9);
        offline.apply(Action::Start, None, 0);
        offline.touch(150.0, 200.0, 300, 400, Some(1), 1);
        let selecting = offline.snapshot().unwrap();
        offline.apply(Action::Reset, None, 2);
        assert_eq!(offline.probe.selected_lane, None);
        assert_eq!(offline.probe.touches, 1);
        assert_eq!(offline.snapshot().unwrap(), selecting);
        offline.apply(Action::Mode { race: false }, None, 3);
        assert_eq!(
            offline.scene(300, 400, 1.0, 3),
            offline.probe.scene(300, 400, 1.0)
        );
        offline.advance(90_000);
        offline.apply(Action::Mode { race: true }, None, 90_000);
        let resumed = offline.snapshot().unwrap();
        assert_eq!(resumed.phase, RoundPhase::Result);
        assert_eq!(resumed.round_id, selecting.round_id);
        assert_eq!(resumed.players, selecting.players);
    }

    #[test]
    fn rejected_and_late_choices_do_not_mutate_a_new_round() {
        let mut offline = race(42);
        offline.apply(Action::Choose { option: 1 }, Some(0), 0);
        assert_eq!(offline.last_action_error.as_deref(), Some("InvalidPhase"));
        offline.apply(Action::Start, None, 1);
        offline.apply(Action::Choose { option: 3 }, Some(1), 2);
        assert_eq!(offline.last_action_error.as_deref(), Some("InvalidOption"));
        offline.apply(Action::Choose { option: 1 }, Some(1), 3_001);
        assert_eq!(offline.last_action_error.as_deref(), Some("InvalidPhase"));
        assert_eq!(
            offline.snapshot().unwrap().result.unwrap().counts,
            [0, 0, 0]
        );
        offline.advance(10_000);
        offline.apply(Action::Start, None, 10_000);
        offline.apply(Action::Choose { option: 1 }, Some(1), 10_001);
        assert_eq!(offline.last_action_error.as_deref(), Some("WrongRound"));
        assert!(
            offline
                .snapshot()
                .unwrap()
                .players
                .iter()
                .all(|p| p.choice.is_none())
        );
        offline.apply(Action::Choose { option: 1 }, None, 10_002);
        assert!(offline.last_action_error.is_some());
        offline.apply(Action::Choose { option: 1 }, Some(2), 10_003);
        assert!(offline.last_action_error.is_none());
    }
}
