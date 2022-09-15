use crate::GameID;
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    Created {
        id: GameID,
        player_0: ActorId,
        player_1: ActorId,
    },
    Canceled(GameID),
    NewTurn {
        id: GameID,
        x: u64,
        y: u64,
        player: ActorId,
    },
    Finished {
        id: GameID,
        winner: Option<ActorId>,
    },
}
