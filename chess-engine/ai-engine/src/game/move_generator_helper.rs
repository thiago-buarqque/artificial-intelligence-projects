use crate::common::{piece_move::PieceMove, piece_utils::PieceType};

use super::board_state::BoardState;

pub fn is_pawn_first_move(white_piece: bool, piece_position: i8) -> bool {
    if white_piece && (48..=55).contains(&piece_position) {
        return true;
    }

    if !white_piece && (8..=15).contains(&piece_position) {
        return true;
    }

    false
}

pub fn position_is_not_attacked(n: i8, opponent_moves: &[PieceMove]) -> bool {
    !opponent_moves.iter().any(|_mut| _mut.to_position == n)
}

pub fn is_path_clear(board_state: &BoardState, start: i8, end: i8, step: i8) -> bool {
    let mut i = start;

    while i != end {
        if board_state.get_piece(i) != PieceType::Empty as i8 {
            return false;
        }
        i += step;
    }

    true
}

pub fn get_king_move(current_position: i8, new_position: i8) -> i8 {
    if !(0..=63).contains(&new_position) {
        return -1;
    }

    // Is on the left side of the board
    if current_position % 8 == 0
        && (new_position == current_position - 1 // left
            || new_position == current_position - 9 // top left
            || new_position == current_position + 7)
    // top bottom
    {
        return -1;
    }

    // Is on the right side of the board
    if (current_position + 1) % 8 == 0
        && (new_position == current_position + 1 // right
            || new_position == current_position - 7 // top right
            || new_position == current_position + 9)
    // bottom right
    {
        return -1;
    }

    new_position
}

pub fn get_knight_move(lines_apart: i8, new_position: i8, current_position: i8) -> i8 {
    if get_line_distance_between_positions(current_position, new_position) == lines_apart {
        return new_position;
    }

    -1
}

pub fn get_line_distance_between_positions(position1: i8, position2: i8) -> i8 {
    let line_start1 = position1 - (position1 % 8);
    let line_start2 = position2 - (position2 % 8);

    if line_start1 > line_start2 {
        return (line_start1 - line_start2) / 8;
    }

    (line_start2 - line_start1) / 8
}