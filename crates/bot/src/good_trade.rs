use chess::{MoveGen, Piece};

use crate::{Bot, BotCreator};

pub struct GoodTrade {}

pub fn get_piece_value(piece: Piece) -> f32 {
    match piece {
        Piece::Pawn => 1.0,
        Piece::Knight => 5.0,
        Piece::Bishop => 3.0,
        Piece::Rook => 3.0,
        Piece::Queen => 9.0,
        Piece::King => std::f32::INFINITY,
    }
}

impl Bot for GoodTrade {
    fn get_next_move(
        &mut self,
        _my_remaining_time: std::time::Duration,
        _opponent_remaining_time: std::time::Duration,
        state: chess::Board,
    ) -> chess::ChessMove {
        let mut best_move = chess::ChessMove::default();
        let mut best_score = std::f32::INFINITY;
        for mv in MoveGen::new_legal(&state) {
            // This our move, we want the checks, captures, attacks score to be maximized.
            let second_level = state.make_move_new(mv);

            let mut move_score = 0.0;
            for mv2 in MoveGen::new_legal(&second_level) {
                // This was our opponents move, we want the checks, captures, attacks score is minimized.
                let third_level = second_level.make_move_new(mv2);

                let current_color = third_level.side_to_move();

                // Look for checks (this is from the perspective of the enemy so higher value is worse for me).
                let mut checks = 0.0;

                // Look for attackers (this is from the perspective of the enemy so higher value is worse move for me).
                let mut attackers = 0.0;

                let my_board = third_level.color_combined(current_color);
                let enemy_board = third_level.color_combined(!current_color);
                for piece in chess::ALL_PIECES {
                    for sq in third_level.pieces(piece).into_iter() {
                        let moves = chess::get_bishop_moves(sq, *my_board);
                        let attacked = moves & enemy_board;
                        if attacked.popcnt() != 0 {
                            for sq in attacked {
                                let piece = third_level.piece_on(sq);
                                if let Some(piece) = piece {
                                    match piece {
                                        Piece::King => {
                                            checks += 1000.0;
                                        }
                                        piece => {
                                            attackers += get_piece_value(piece) - 3.0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let score = checks + attackers;
                if score < move_score {
                    move_score = score;
                }
            }

            if move_score < best_score {
                best_score = move_score;
                best_move = mv;
            }
        }

        best_move
    }
}

impl BotCreator for GoodTrade {
    fn create() -> Self {
        GoodTrade {}
    }
}
