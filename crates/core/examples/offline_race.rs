use game_core::game::{GameCommand, GameError, GameKind, Outcome, Room, RoundPhase};

fn main() -> Result<(), GameError> {
    let seed = 42;
    let mut room = Room::new(GameKind::Race, 10, seed);
    for id in [20, 30, 40] {
        room.join(id)?;
    }
    println!("seed={seed} host=10 join=[20, 30, 40]");
    let inputs = [
        (10, 0, GameCommand::Start, 100),
        (10, 1, GameCommand::Choose { option: 0 }, 200),
        (20, 1, GameCommand::Choose { option: 1 }, 300),
        (30, 1, GameCommand::Choose { option: 0 }, 400),
        (20, 1, GameCommand::Choose { option: 2 }, 500),
    ];
    for (actor, round, command, now_ms) in inputs {
        println!("input actor={actor} round={round} command={command:?} now_ms={now_ms}");
        room.apply(actor, round, command, now_ms)?;
    }
    assert!(room.advance(3_100));
    println!("deadline snapshot={:?}", room.snapshot());
    assert!(room.advance(4_900));
    let result = room.snapshot();
    assert_eq!(result.phase, RoundPhase::Result);
    assert_eq!(
        result.result,
        Some(Outcome {
            lane_order: [0, 2, 1],
            counts: [2, 0, 1],
            winners: vec![10, 30],
        })
    );
    println!("result snapshot={result:?}");
    room.apply(10, 1, GameCommand::Start, 5_000)?;
    let next = room.snapshot();
    assert_eq!(next.round_id, 2);
    assert_eq!(next.phase, RoundPhase::Selecting);
    assert!(next.players.iter().all(|player| player.choice.is_none()));
    println!("next_round snapshot={next:?}");
    Ok(())
}
