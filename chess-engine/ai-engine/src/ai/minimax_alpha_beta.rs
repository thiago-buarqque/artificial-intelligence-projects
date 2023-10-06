use crate::{
    common::{
        piece_move::PieceMove,
        piece_utils::{
            get_piece_worth, get_promotion_char_options, is_white_piece, piece_fen_from_value,
            piece_value_from_fen, PieceType,
        },
    },
    game::board::Board,
};

pub struct MiniMaxAlphaBeta {
    pub states_checked: usize,
}

impl MiniMaxAlphaBeta {
    pub fn new() -> Self {
        MiniMaxAlphaBeta { states_checked: 0 }
    }

    pub fn make_move(&mut self, board: &mut Board, depth: u8) -> (i32, PieceMove) {
        // self.states_checked = 0;

        let result = self.minimax(board, i32::MIN, i32::MAX, depth, true);

        println!("Evaluated {} states", result.2);

        (result.0, result.1)
    }

    fn minimax(
        &mut self,
        board: &mut Board,
        alpha: i32,
        beta: i32,
        depth: u8,
        max: bool,
    ) -> (i32, PieceMove, i64) {
        let mut alpha = alpha;
        let mut beta = beta;

        if depth == 0 || board.is_game_finished() {
            return (self.get_board_value(board), PieceMove::new(-1, -1), 1);
        }

        let mut moves_count = 0;
        let mut value = if max { i32::MIN } else { i32::MAX };
        let mut best_move: PieceMove = PieceMove::new(-1, -1);

        let pieces = board.get_pieces().clone();

        // get_available_moves should only return the pieces, not empties
        for piece in pieces.iter().flatten() {
            if (piece.get_value() == PieceType::Empty as i8)
                || (piece.is_white()) != board.is_white_move()
            {
                continue;
            }

            'move_loop: for piece_move in piece.get_immutable_moves().iter() {
                let mut promotion_char_options =
                    vec![piece_fen_from_value(piece_move.promotion_type)];

                if piece_move.is_promotion {
                    promotion_char_options = get_promotion_char_options(piece.is_white());
                }

                let mut piece_move = piece_move.clone();

                for promotion_option in promotion_char_options {
                    piece_move.promotion_type = piece_value_from_fen(&promotion_option);

                    board.move_piece(piece_move.clone());

                    if max {
                        let current_move_value = self.minimax(board, alpha, beta, depth - 1, false);

                        if current_move_value.0 > value {
                            value = current_move_value.0;
                            best_move = piece_move.clone();
                        }

                        moves_count += current_move_value.2;

                        if value >= beta {
                            board.undo_move();

                            break 'move_loop;
                        }

                        alpha = if value > alpha { value } else { alpha };
                    } else {
                        let current_move_value = self.minimax(board, alpha, beta, depth - 1, true);

                        if current_move_value.0 < value {
                            value = current_move_value.0;
                            best_move = piece_move.clone();
                        }

                        moves_count += current_move_value.2;

                        if value <= alpha {
                            board.undo_move();

                            break 'move_loop;
                        }

                        beta = if value < beta { value } else { beta };
                    }

                    board.undo_move();
                }
            }
        }

        (value, best_move, moves_count)
    }

    fn get_board_value(&self, board: &mut Board) -> i32 {
        // For this first algorithm version the
        // sum of the pieces will give the state value
        let mut board_value: i32 = 0;
        for piece in board.get_squares().iter() {
            if *piece == PieceType::Empty as i8 {
                continue;
            }

            let piece_worth: i32 = get_piece_worth(*piece);

            if is_white_piece(*piece) {
                board_value += piece_worth;
            } else {
                board_value -= piece_worth;
            }
        }

        board_value
    }
}
