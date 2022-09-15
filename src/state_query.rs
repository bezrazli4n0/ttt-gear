use crate::state::{BoardMark, GameID};
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateQuery {
    GetNonce,
    GetGamesLen,
    IsEnded(GameID),
    IsBoardFilled(GameID),
    GetBoardMark((GameID, ActorId)),
    GetPlayer((GameID, BoardMark)),
    GetNextTurn(GameID),
    GetWinner(GameID),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateQueryReply {
    Nonce(GameID),
    GamesLen(GameID),
    IsEnded(bool),
    IsBoardFilled(bool),
    BoardMark(BoardMark),
    Player(ActorId),
    NextTurn {
        player: ActorId,
        board_mark: BoardMark,
    },
    Winner(Option<ActorId>),
}
