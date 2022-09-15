#![no_std]
#![allow(clippy::missing_safety_doc)]

pub mod action;
pub mod event;
pub mod state;
pub mod state_query;

use action::*;
use event::*;
use gstd::{msg, prelude::*};
use state::*;
use state_query::*;

#[derive(Debug, Default)]
pub struct TicTacToe {
    pub games: BTreeMap<GameID, Game>,
    pub nonce: GameID,
}

static mut TIC_TAC_TOE: Option<TicTacToe> = None;

gstd::metadata! {
    title: "TicTacToe",
    handle:
        input: Action,
        output: Event,
    state:
        input: StateQuery,
        output: StateQueryReply,
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let tic_tac_toe = TicTacToe {
        nonce: 0,
        ..Default::default()
    };

    TIC_TAC_TOE = Some(tic_tac_toe);
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Invalid Action data!");
    let ttt: &mut TicTacToe = TIC_TAC_TOE.get_or_insert(TicTacToe::default());

    match action {
        Action::Create(opponent) => {
            ttt.nonce = ttt.nonce.checked_add(1).expect("Math overflow!");
            let id = ttt.nonce;

            let player_0 = msg::source();
            let player_1 = opponent;

            ttt.games.insert(id, Game::init(player_0, player_1));

            msg::reply(
                Event::Created {
                    id,
                    player_0,
                    player_1,
                },
                0,
            )
            .unwrap();
        }
        Action::Cancel(id) => {
            let game = ttt.games.get_mut(&id).expect("Game not found!");
            game.cancel(&msg::source());

            msg::reply(Event::Canceled(id), 0).unwrap();
        }
        Action::Turn { id, x, y } => {
            let game = ttt.games.get_mut(&id).expect("Game not found!");
            let player = msg::source();

            let is_game_finished = game.turn(
                &player,
                x.try_into().expect("TryInto overflow!"),
                y.try_into().expect("TryInto overflow!"),
            );
            let maybe_winner = game.get_winner();

            if is_game_finished {
                msg::reply(
                    Event::Finished {
                        id,
                        winner: maybe_winner,
                    },
                    0,
                )
                .unwrap();
            } else {
                msg::reply(Event::NewTurn { id, x, y, player }, 0).unwrap();
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: StateQuery = msg::load().expect("Invalid StateQuery data!");
    let ttt: &mut TicTacToe = TIC_TAC_TOE.get_or_insert(TicTacToe::default());

    let encoded = match query {
        StateQuery::GetNonce => StateQueryReply::Nonce(ttt.nonce),
        StateQuery::GetGamesLen => StateQueryReply::GamesLen(ttt.games.len() as GameID),
        StateQuery::IsEnded(id) => {
            let game = ttt.games.get_mut(&id).expect("Game not found!");
            StateQueryReply::IsEnded(game.is_ended())
        }
        StateQuery::IsBoardFilled(id) => {
            let game = ttt.games.get_mut(&id).expect("Game not found!");
            StateQueryReply::IsBoardFilled(game.is_board_filled())
        }
        StateQuery::GetBoardMark((id, player)) => {
            let game = ttt.games.get_mut(&id).expect("Game not found!");
            StateQueryReply::BoardMark(game.get_board_mark(&player))
        }
        StateQuery::GetPlayer((id, board_mark)) => {
            let game = ttt.games.get_mut(&id).expect("Game not found!");
            StateQueryReply::Player(game.get_player(board_mark))
        }
        StateQuery::GetNextTurn(id) => {
            let game = ttt.games.get_mut(&id).expect("Game not found!");
            let (player, board_mark) = game.get_next_turn();
            StateQueryReply::NextTurn { player, board_mark }
        }
        StateQuery::GetWinner(id) => {
            let game = ttt.games.get_mut(&id).expect("Game not found!");
            StateQueryReply::Winner(game.get_winner())
        }
    }
    .encode();

    gstd::util::to_leak_ptr(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtest::{Program, System};

    #[test]
    fn success_create_action() {
        let sys = System::new();
        sys.init_logger();

        let owner: u64 = 3;
        let player_0: u64 = 4;
        let player_1: u64 = 5;

        let tic_tac_toe = Program::current(&sys);
        let result = tic_tac_toe.send_bytes(owner, [0u8; 1]);
        assert!(result.log().is_empty());

        let result = tic_tac_toe.send(player_0, Action::Create(player_1.into()));
        assert!(result.contains(&(
            player_0,
            Event::Created {
                id: 1,
                player_0: player_0.into(),
                player_1: player_1.into()
            }
            .encode()
        )));
    }

    #[test]
    fn success_cancel_action() {
        let sys = System::new();
        sys.init_logger();

        let owner: u64 = 3;
        let player_0: u64 = 4;
        let player_1: u64 = 5;

        let tic_tac_toe = Program::current(&sys);
        let result = tic_tac_toe.send_bytes(owner, [0u8; 1]);
        assert!(result.log().is_empty());

        let result = tic_tac_toe.send(player_0, Action::Create(player_1.into()));
        assert!(result.contains(&(
            player_0,
            Event::Created {
                id: 1,
                player_0: player_0.into(),
                player_1: player_1.into()
            }
            .encode()
        )));

        let result = tic_tac_toe.send(player_1, Action::Cancel(1));
        assert!(result.contains(&(player_1, Event::Canceled(1).encode())));
    }

    #[test]
    fn success_turn_action() {
        let sys = System::new();
        sys.init_logger();

        let owner: u64 = 3;
        let player_0: u64 = 4;
        let player_1: u64 = 5;
        let game_id = 1;

        let tic_tac_toe = Program::current(&sys);
        let result = tic_tac_toe.send_bytes(owner, [0u8; 1]);
        assert!(result.log().is_empty());

        let result = tic_tac_toe.send(player_0, Action::Create(player_1.into()));
        assert!(result.contains(&(
            player_0,
            Event::Created {
                id: game_id,
                player_0: player_0.into(),
                player_1: player_1.into()
            }
            .encode()
        )));

        let result = tic_tac_toe.send(
            player_0,
            Action::Turn {
                id: game_id,
                x: 0,
                y: 0,
            },
        );
        assert!(result.contains(&(
            player_0,
            Event::NewTurn {
                id: game_id,
                x: 0,
                y: 0,
                player: player_0.into()
            }
            .encode()
        )));
    }

    #[test]
    fn success_turn_finished_action() {
        let sys = System::new();
        sys.init_logger();

        let owner: u64 = 3;
        let player_0: u64 = 4;
        let player_1: u64 = 5;
        let game_id = 1;

        let tic_tac_toe = Program::current(&sys);
        let result = tic_tac_toe.send_bytes(owner, [0u8; 1]);
        assert!(result.log().is_empty());

        let result = tic_tac_toe.send(player_0, Action::Create(player_1.into()));
        assert!(result.contains(&(
            player_0,
            Event::Created {
                id: game_id,
                player_0: player_0.into(),
                player_1: player_1.into()
            }
            .encode()
        )));

        let result = tic_tac_toe.send(
            player_0,
            Action::Turn {
                id: game_id,
                x: 0,
                y: 0,
            },
        );
        assert!(result.contains(&(
            player_0,
            Event::NewTurn {
                id: game_id,
                x: 0,
                y: 0,
                player: player_0.into()
            }
            .encode()
        )));

        let result = tic_tac_toe.send(
            player_1,
            Action::Turn {
                id: game_id,
                x: 1,
                y: 1,
            },
        );
        assert!(result.contains(&(
            player_1,
            Event::NewTurn {
                id: game_id,
                x: 1,
                y: 1,
                player: player_1.into()
            }
            .encode()
        )));

        let result = tic_tac_toe.send(
            player_0,
            Action::Turn {
                id: game_id,
                x: 0,
                y: 1,
            },
        );
        assert!(result.contains(&(
            player_0,
            Event::NewTurn {
                id: game_id,
                x: 0,
                y: 1,
                player: player_0.into()
            }
            .encode()
        )));

        let result = tic_tac_toe.send(
            player_1,
            Action::Turn {
                id: game_id,
                x: 2,
                y: 2,
            },
        );
        assert!(result.contains(&(
            player_1,
            Event::NewTurn {
                id: game_id,
                x: 2,
                y: 2,
                player: player_1.into()
            }
            .encode()
        )));

        let result = tic_tac_toe.send(
            player_0,
            Action::Turn {
                id: game_id,
                x: 0,
                y: 2,
            },
        );
        assert!(result.contains(&(
            player_0,
            Event::Finished {
                id: game_id,
                winner: Some(player_0.into())
            }
            .encode()
        )));
    }
}
