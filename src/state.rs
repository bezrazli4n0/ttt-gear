use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};

pub const BOARD_SIZE: usize = 3;
pub type GameID = u128;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum BoardMark {
    X,
    O,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GameStatus {
    Created,
    Canceled,
    Finished { winner: Option<ActorId> },
}

#[derive(Debug)]
pub struct Game {
    pub board: [[Option<BoardMark>; BOARD_SIZE]; BOARD_SIZE],
    pub player_0: ActorId,
    pub player_1: ActorId,
    pub next_turn: (ActorId, BoardMark),
    pub player_to_board_mark: BTreeMap<ActorId, BoardMark>,
    pub status: GameStatus,
}

impl Game {
    pub fn init(player_0: ActorId, player_1: ActorId) -> Self {
        if player_0 == player_1 {
            panic!("You must have friends ;(");
        }

        // Custom first turn logic can be applied here:
        let next_turn = (player_0, BoardMark::X);

        let mut player_to_board_mark = BTreeMap::new();
        player_to_board_mark.insert(player_0, BoardMark::X);
        player_to_board_mark.insert(player_1, BoardMark::O);

        Game {
            board: Default::default(),
            player_0,
            player_1,
            next_turn,
            player_to_board_mark,
            status: GameStatus::Created,
        }
    }

    pub fn is_ended(&self) -> bool {
        matches!(
            self.status,
            GameStatus::Canceled | GameStatus::Finished { winner: _ }
        )
    }

    pub fn is_board_filled(&self) -> bool {
        for y_axis in &self.board {
            for x_axis in y_axis {
                if x_axis.is_none() {
                    return false;
                }
            }
        }

        true
    }

    pub fn get_board_mark(&self, player: &ActorId) -> BoardMark {
        self.player_to_board_mark
            .get(player)
            .expect("Player not found")
            .clone()
    }

    pub fn get_player(&self, board_mark: BoardMark) -> ActorId {
        let actor_id = self
            .player_to_board_mark
            .iter()
            .find_map(|(actor_id, actor_mark)| {
                if actor_mark == &board_mark {
                    Some(actor_id)
                } else {
                    None
                }
            })
            .expect("Invalid data");

        *actor_id
    }

    /// Returns `next_turn` based on current `next_turn`.
    pub fn get_next_turn(&self) -> (ActorId, BoardMark) {
        let (last_player, last_board_mark) = &self.next_turn;

        let next_player = if last_player == &self.player_0 {
            self.player_1
        } else {
            self.player_0
        };

        let next_board_mark = if last_board_mark == &BoardMark::X {
            BoardMark::O
        } else {
            BoardMark::X
        };

        (next_player, next_board_mark)
    }

    pub fn get_winner(&self) -> Option<ActorId> {
        match self.status {
            GameStatus::Finished { winner } => winner,
            _ => None,
        }
    }

    pub fn check_winner_row(
        &self,
        x_indexes: [usize; BOARD_SIZE],
        y_indexes: [usize; BOARD_SIZE],
    ) -> Option<BoardMark> {
        if self.board[y_indexes[0]][x_indexes[0]].is_some()
            && self.board[y_indexes[1]][x_indexes[1]].is_some()
            && self.board[y_indexes[2]][x_indexes[2]].is_some()
        {
            let a = self.board[y_indexes[0]][x_indexes[0]].as_ref().unwrap();
            let b = self.board[y_indexes[1]][x_indexes[1]].as_ref().unwrap();
            let c = self.board[y_indexes[2]][x_indexes[2]].as_ref().unwrap();

            if a == b && a == c {
                return Some(a.clone());
            }
        }

        None
    }

    pub fn check_winner(&self) -> Option<BoardMark> {
        /*
            +++
            ---
            ---
        */
        let res = self.check_winner_row([0, 1, 2], [0, 0, 0]);
        if res.is_some() {
            return res;
        }

        /*
            ---
            +++
            ---
        */
        let res = self.check_winner_row([0, 1, 2], [1, 1, 1]);
        if res.is_some() {
            return res;
        }

        /*
            ---
            ---
            +++
        */
        let res = self.check_winner_row([0, 1, 2], [2, 2, 2]);
        if res.is_some() {
            return res;
        }

        /*
            +--
            +--
            +--
        */
        let res = self.check_winner_row([0, 0, 0], [0, 1, 2]);
        if res.is_some() {
            return res;
        }

        /*
            -+-
            -+-
            -+-
        */
        let res = self.check_winner_row([1, 1, 1], [0, 1, 2]);
        if res.is_some() {
            return res;
        }

        /*
            --+
            --+
            --+
        */
        let res = self.check_winner_row([2, 2, 2], [0, 1, 2]);
        if res.is_some() {
            return res;
        }

        /*
            --+
            -+-
            +--
        */
        let res = self.check_winner_row([2, 1, 0], [0, 1, 2]);
        if res.is_some() {
            return res;
        }

        /*
            +--
            -+-
            --+
        */
        self.check_winner_row([0, 1, 2], [0, 1, 2])
    }

    /// Returns condition which indicates
    /// end of the game, when:
    ///
    /// - Win combination is found.
    ///
    /// - Game board is filled.
    fn handle_game_round(&mut self) -> bool {
        // 1. Check gaming board for winning combination
        if let Some(winner_mark) = self.check_winner() {
            let winner = self.get_player(winner_mark);

            self.status = GameStatus::Finished {
                winner: Some(winner),
            };
            return true;
        }

        // 2. Check if gaming board is filled(We have a tie in the game)
        if self.is_board_filled() {
            self.status = GameStatus::Finished { winner: None };
            return true;
        }

        false
    }

    /// Handle current `player` turn.
    ///
    /// Returns `true` if game is finished at this turn.
    pub fn turn(&mut self, player: &ActorId, x: usize, y: usize) -> bool {
        self.assert_not_ended();
        self.assert_player_in_game(player);

        let (current_player, current_mark) = self.next_turn.clone();

        // 1. Handle possible ending state before turn
        if self.handle_game_round() {
            return true;
        }

        // 2. Check if `player` can do current turn
        if player != &current_player {
            panic!("It's not your turn!");
        }

        // 3. Place `player` mark
        let y_cell = self.board.get_mut(y).expect("Invalid y index!");
        let x_cell = y_cell.get_mut(x).expect("Invalid x index!");

        if x_cell.is_some() {
            panic!("Location is not empty!");
        }

        *x_cell = Some(current_mark);

        // 4. Handle possible ending state after turn
        if self.handle_game_round() {
            return true;
        }

        // 5. Update next turn
        self.next_turn = self.get_next_turn();
        false
    }

    fn assert_not_ended(&self) {
        if self.is_ended() {
            panic!("Game is ended!");
        }
    }

    fn assert_player_in_game(&self, player: &ActorId) {
        if &self.player_0 != player && &self.player_1 != player {
            panic!("Player not found in this game!");
        }
    }

    pub fn cancel(&mut self, player: &ActorId) {
        self.assert_not_ended();
        self.assert_player_in_game(player);

        self.status = GameStatus::Canceled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gstd::ActorId;

    fn setup() -> (ActorId, ActorId, Game) {
        let player_0 = ActorId::new([0u8; 32]);
        let player_1 = ActorId::new([1u8; 32]);

        (player_0, player_1, Game::init(player_0, player_1))
    }

    #[test]
    fn success_init() {
        let (player_0, player_1, game) = setup();

        assert_eq!(game.player_0, player_0);
        assert_eq!(game.player_1, player_1);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.status, GameStatus::Created);
    }

    #[test]
    fn success_turn_handle_game_round_winner() {
        let (player_0, player_1, mut game) = setup();
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        game.turn(&player_0, 0, 0);
        assert_eq!(game.next_turn, (player_1, BoardMark::O));
        assert_eq!(game.get_next_turn(), (player_0, BoardMark::X));

        game.turn(&player_1, 2, 2);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        game.turn(&player_0, 0, 1);
        assert_eq!(game.next_turn, (player_1, BoardMark::O));
        assert_eq!(game.get_next_turn(), (player_0, BoardMark::X));

        game.turn(&player_1, 1, 1);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        let turn_result = game.turn(&player_0, 0, 2);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        assert!(turn_result);
        assert!(game.is_ended());
        assert!(!game.is_board_filled());
        assert_eq!(game.get_winner(), Some(player_0));
        assert_eq!(
            game.status,
            GameStatus::Finished {
                winner: Some(player_0)
            }
        );
    }

    #[test]
    fn success_turn_handle_game_round_filled() {
        let (player_0, player_1, mut game) = setup();
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        game.turn(&player_0, 1, 1);
        assert_eq!(game.next_turn, (player_1, BoardMark::O));
        assert_eq!(game.get_next_turn(), (player_0, BoardMark::X));

        game.turn(&player_1, 0, 0);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        game.turn(&player_0, 2, 2);
        assert_eq!(game.next_turn, (player_1, BoardMark::O));
        assert_eq!(game.get_next_turn(), (player_0, BoardMark::X));

        game.turn(&player_1, 2, 1);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        game.turn(&player_0, 2, 0);
        assert_eq!(game.next_turn, (player_1, BoardMark::O));
        assert_eq!(game.get_next_turn(), (player_0, BoardMark::X));

        game.turn(&player_1, 0, 2);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        game.turn(&player_0, 0, 1);
        assert_eq!(game.next_turn, (player_1, BoardMark::O));
        assert_eq!(game.get_next_turn(), (player_0, BoardMark::X));

        game.turn(&player_1, 1, 0);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        let turn_result = game.turn(&player_0, 1, 2);
        assert_eq!(game.next_turn, (player_0, BoardMark::X));
        assert_eq!(game.get_next_turn(), (player_1, BoardMark::O));

        assert!(turn_result);
        assert!(game.is_ended());
        assert!(game.is_board_filled());
        assert_eq!(game.get_winner(), None);
        assert_eq!(game.status, GameStatus::Finished { winner: None });
    }

    #[test]
    fn success_turn() {
        let (player_0, player_1, mut game) = setup();

        let turn_result = game.turn(&player_0, 0, 0);

        assert!(!turn_result);
        assert!(!game.is_ended());
        assert!(!game.is_board_filled());
        assert!(game.board[0][0].is_some());
        assert_eq!(game.next_turn, (player_1, BoardMark::O));
    }

    #[test]
    fn success_cancel() {
        let (player_0, _, mut game) = setup();

        game.cancel(&player_0);

        assert!(game.is_ended());
        assert!(!game.is_board_filled());
        assert_eq!(game.status, GameStatus::Canceled);
    }

    #[test]
    #[should_panic(expected = "You must have friends ;(")]
    fn panic_init_players_eq() {
        let player_0 = ActorId::new([0u8; 32]);
        let _game = Game::init(player_0, player_0);
    }

    #[test]
    #[should_panic(expected = "Game is ended!")]
    fn panic_turn_ended() {
        let (player_0, _, mut game) = setup();

        game.cancel(&player_0);
        game.turn(&player_0, 0, 0);
    }

    #[test]
    #[should_panic(expected = "Player not found in this game!")]
    fn panic_turn_player_not_exists() {
        let (_, _, mut game) = setup();
        let player_2 = ActorId::new([2u8; 32]);

        game.turn(&player_2, 0, 0);
    }

    #[test]
    #[should_panic(expected = "It's not your turn!")]
    fn panic_turn_invalid_sequence() {
        let (_, player_1, mut game) = setup();
        game.turn(&player_1, 0, 0);
    }

    #[test]
    #[should_panic(expected = "Location is not empty!")]
    fn panic_turn_location_not_empty() {
        let (player_0, player_1, mut game) = setup();
        game.turn(&player_0, 0, 0);
        game.turn(&player_1, 0, 0);
    }

    #[test]
    #[should_panic(expected = "Game is ended!")]
    fn panic_cancel_ended() {
        let (player_0, _, mut game) = setup();

        game.cancel(&player_0);
        game.cancel(&player_0);
    }

    #[test]
    #[should_panic(expected = "Player not found in this game!")]
    fn panic_cancel_player_not_exists() {
        let (_, _, mut game) = setup();
        let player_2 = ActorId::new([2u8; 32]);

        game.cancel(&player_2);
    }
}
