use crate::GameID;
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Action {
    Create(ActorId),
    Cancel(GameID),
    Turn { id: GameID, x: u64, y: u64 },
}
