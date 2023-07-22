mod random;

use std::time::Duration;

pub use chess::Action;
use chess::ChessMove;

pub trait Bot: Send + Sync {
    fn get_next_move(
        &mut self,
        my_remaining_time: Duration,
        opponent_remaining_time: Duration,
        state: chess::Board,
    ) -> ChessMove;
}

pub trait BotCreator: Bot
where
    Self: Sized,
{
    fn create() -> Self;
}

pub use random::Random;
