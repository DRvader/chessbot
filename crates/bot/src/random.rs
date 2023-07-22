use chess::MoveGen;
use rand::Rng;

use crate::{Bot, BotCreator};

pub struct Random {}

impl Bot for Random {
    fn get_next_move(
        &mut self,
        _my_remaining_time: std::time::Duration,
        _opponent_remaining_time: std::time::Duration,
        state: chess::Board,
    ) -> chess::ChessMove {
        let mut movegen = MoveGen::new_legal(&state);

        let mut rng = rand::thread_rng();

        let move_index = rng.gen_range(0..movegen.len());

        movegen.nth(move_index).unwrap()
    }
}

impl BotCreator for Random {
    fn create() -> Self {
        Random {}
    }
}
