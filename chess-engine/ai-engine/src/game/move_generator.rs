use std::sync::{Arc, Mutex};

use crate::common::{
    board_piece::BoardPiece,
    piece_move::PieceMove,
    piece_utils::{
        get_piece_type, is_same_color, is_white_piece, piece_fen_from_value, PieceColor, PieceType,
    },
};

use super::{board::Board, board_state::BoardState};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SquareOffset {
    LineBelow = 8,
    LineAbove = -8,
    TopRight = -7,
    TopLeft = -9,
    BottomRight = 9,
    BottomLeft = 7,
    Left = -1,
    Right = 1,
}

impl SquareOffset {
    fn value(&self) -> i8 {
        *self as i8
    }
}

#[derive(Debug, Clone)]
pub struct MoveGenerator {
    board_state: Arc<Mutex<BoardState>>,
}

impl MoveGenerator {
    pub fn new(board_state: Arc<Mutex<BoardState>>) -> Self {
        Self { board_state }
    }

    fn generate_moves(&self, piece_type: PieceType, position: i8) -> Vec<PieceMove> {
        // King moves are generated after generating the other pieces moves

        match piece_type {
            PieceType::Bishop => self.generate_bishop_moves(position),
            PieceType::Knight => self.generate_knight_moves(position),
            PieceType::Pawn => self.generate_pawn_moves(position),
            PieceType::Queen => self.generate_queen_moves(position),
            PieceType::Rook => self.generate_rook_moves(position),
            _ => vec![],
        }
    }

    // pub fn load_state(&mut self, state: Arc<Mutex<BoardState>>) {
    //     self.board_state = state;
    // }

    pub fn get_available_moves(&mut self, board: &mut Board) -> Vec<Option<BoardPiece>> {
        let mut black_moves: Vec<PieceMove> = Vec::new();
        let mut white_moves: Vec<PieceMove> = Vec::new();
        let mut pieces: Vec<Option<BoardPiece>> = Vec::new();

        let mut white_king_position: i8 = -1;
        let mut black_king_position: i8 = -1;

        let board_state_clone = board.get_state_clone();

        let squares = board_state_clone.squares();

        for (position, &piece_value) in squares.iter().enumerate() {
            if piece_value == PieceType::Empty as i8 {
                pieces.push(Some(BoardPiece::new(
                    '-',
                    Vec::new(),
                    position as i8,
                    piece_value,
                    false,
                )));
                continue;
            }

            let white_piece = is_white_piece(piece_value);
            let piece_type = get_piece_type(piece_value);

            if piece_type == PieceType::King {
                if white_piece {
                    white_king_position = position as i8;
                } else {
                    black_king_position = position as i8;
                }
                pieces.push(Some(BoardPiece::new(
                    piece_fen_from_value(piece_value),
                    Vec::new(),
                    position as i8,
                    piece_value,
                    white_piece,
                )));
            } else {
                let moves = self.generate_moves(piece_type, position as i8);

                let piece_fen = piece_fen_from_value(piece_value);
                let piece = BoardPiece::new(
                    piece_fen,
                    moves.clone(),
                    position as i8,
                    piece_value,
                    white_piece,
                );

                pieces.push(Some(piece));

                if white_piece {
                    white_moves.extend(moves);
                } else {
                    black_moves.extend(moves);
                }
            }
        }

        // Only the two kings on the board
        if black_moves.is_empty() && white_moves.is_empty() {
            let mut board_state = self.board_state.lock().unwrap();

            // Game ends in draw
            board_state.set_winner(PieceColor::Black as i8 | PieceColor::White as i8);
        } else {
            self.get_king_available_moves(
                black_king_position,
                &black_moves,
                &mut pieces,
                white_king_position,
                &white_moves,
            );

            self.remove_blocked_piece_moves(
                &mut pieces,
                black_king_position,
                white_king_position,
                board,
            );
        }

        pieces
    }

    fn get_king_available_moves(
        &self,
        black_king_position: i8,
        black_moves: &[PieceMove],
        board_pieces: &mut [Option<BoardPiece>],
        white_king_position: i8,
        white_moves: &[PieceMove],
    ) {
        let mut white_king_moves = self.generate_king_moves(black_moves, white_king_position);
        let mut black_king_moves = self.generate_king_moves(white_moves, black_king_position);

        let common_moves: Vec<PieceMove> = white_king_moves
            .iter()
            .cloned()
            .filter(|x| black_king_moves.contains(x))
            .collect();

        white_king_moves.retain(|x| !common_moves.contains(x));
        black_king_moves.retain(|x| !common_moves.contains(x));

        let locked_board_state = self.board_state.lock().unwrap();

        let white_king_piece = BoardPiece::new(
            piece_fen_from_value(PieceColor::White as i8 | PieceType::King as i8),
            white_king_moves,
            white_king_position,
            locked_board_state.get_piece(white_king_position),
            true,
        );

        let black_king_piece = BoardPiece::new(
            piece_fen_from_value(PieceColor::Black as i8 | PieceType::King as i8),
            black_king_moves,
            black_king_position,
            locked_board_state.get_piece(black_king_position),
            false,
        );

        if white_king_position != -1 {
            board_pieces[white_king_position as usize] = Some(white_king_piece);
        }

        if black_king_position != -1 {
            board_pieces[black_king_position as usize] = Some(black_king_piece);
        }
    }

    pub fn remove_blocked_piece_moves(
        &mut self,
        pieces: &mut [Option<BoardPiece>],
        black_king_position: i8,
        white_king_position: i8,
        board: &mut Board,
    ) {
        let mut locked_board_state = self.board_state.lock().unwrap();

        let king_position = if locked_board_state.is_white_move() {
            white_king_position
        } else {
            black_king_position
        };

        let is_white_move = locked_board_state.is_white_move();
        drop(locked_board_state);

        let mut player_moves: Vec<PieceMove> = Vec::new();

        let mut is_king_in_check = false;

        // Is it possible to brake the loop earlier if king is in check on the next move?
        for board_piece in pieces.iter_mut().flatten() {
            // println!("To piece: {}", board_piece.get_position());
            let piece_position = board_piece.get_position();

            locked_board_state = self.board_state.lock().unwrap();

            let piece_value = locked_board_state.get_piece(piece_position);
            let mut invalid_moves = Vec::new();

            drop(locked_board_state);

            if board_piece.is_white() != is_white_move {
                continue;
            }

            let piece_moves = board_piece.get_immutable_moves();

            if !is_king_in_check
                && piece_moves
                    .iter()
                    .any(|_move| _move.to_position == king_position)
            {
                is_king_in_check = true;
            }

            for piece_move in piece_moves {
                if get_piece_type(piece_value) == PieceType::King {
                    let _ = board.move_piece(piece_move.clone());
    
                    let state = board.get_state_clone();
                    let opponent_next_moves =
                        self.get_next_moves_attack_to_king(piece_move.to_position, state);
                        
                    board.undo_move();
                    
                    self.board_state = board.get_state_reference().clone();
                        
                    println!(
                        "{} Opponent attack for king: {:?}",
                        piece_move.from_position, opponent_next_moves
                    );
                    if opponent_next_moves
                        .iter()
                        .any(|_move| _move.to_position == piece_move.to_position)
                    {
                        println!("Invalidating {:?}", piece_move);
                        invalid_moves.push(piece_move);
                    }

                    continue;
                }
                let mut piece_move_clone = piece_move.clone();

                if piece_move_clone.is_promotion && piece_move.promotion_type == PieceType::Empty as i8{
                    piece_move_clone.promotion_type = piece_value
                }

                let _ = board.move_piece(piece_move_clone.clone());

                let state = board.get_state_clone();
                // TODO I should probably care if the pawn is being promoted?
                let opponent_next_moves = 
                    self.get_next_moves_attack_to_king(king_position, state);


                board.undo_move();

                self.board_state = board.get_state_reference().clone();

                if opponent_next_moves
                    .iter()
                    .any(|_move| _move.to_position == king_position)
                {
                    println!("Invalidating move: {:?}", piece_move.clone());
                    invalid_moves.push(piece_move);
                }
            }

            board_piece
                .get_moves()
                .retain(|x| !invalid_moves.contains(x));

            if is_white_piece(piece_value) == is_white_move {
                player_moves.extend(board_piece.get_immutable_moves());
            }
        }

        // board.load_state_and_clear_history(self.board_state.clone());

        // TODO Check this when returning pieces on Board??
        if player_moves.is_empty() {
            locked_board_state = self.board_state.lock().unwrap();

            let winner = if is_king_in_check {
                if is_white_move {
                    PieceColor::Black as i8
                } else {
                    PieceColor::White as i8
                }
            } else {
                PieceColor::Black as i8 | PieceColor::White as i8 // Draw
            };

            locked_board_state.set_winner(winner);
        }
    }

    fn get_next_moves_attack_to_king(
        &self,
        king_position: i8,
        board_state: BoardState,
    ) -> Vec<PieceMove> {
        let mut all_moves: Vec<PieceMove> = Vec::new();

        let is_white_move = board_state.is_white_move();

        let squares = board_state.squares();

        for (position, &piece) in squares.iter().enumerate() {
            if (piece == (PieceType::Empty as i8)) || is_white_piece(piece) != is_white_move {
                continue;
            }

            let piece_type = get_piece_type(piece);

            if piece_type != PieceType::King {
                let moves = self.generate_moves(piece_type, position as i8);

                // Note for future me:
                // I can't declare moves before the if and use after the else. It is
                // moved for its declaration and can't be borrowed after the else. WHY???
                //
                // If the king is being attacked, we're abre to break the loop, no need
                // to check all piece in the board
                if moves.iter().any(|_move| _move.to_position == king_position) {
                    println!("Attacking king from {} to {}", position, king_position);
                    return moves;
                }

                all_moves.extend(moves);
            } else {
                let opponent_moves: Vec<PieceMove> = Vec::new();
                // generates the current player king possible moves, even if they're invalid.
                // This is just to prevent kings to be aside with each other

                // This probably should know all opponents attack
                let moves = self.generate_king_moves(&opponent_moves, position as i8);

                if moves.iter().any(|_move| _move.to_position == king_position) {
                    return moves;
                }

                all_moves.extend(moves);
            }
        }

        all_moves
    }

    fn get_knight_move(&self, lines_apart: i8, new_position: i8, current_position: i8) -> i8 {
        if self.get_line_distance_between_positions(current_position, new_position) == lines_apart {
            return new_position;
        }

        -1
    }

    // Get positions line distance
    fn get_line_distance_between_positions(&self, position1: i8, position2: i8) -> i8 {
        let line_start1 = position1 - (position1 % 8);
        let line_start2 = position2 - (position2 % 8);

        if line_start1 > line_start2 {
            return (line_start1 - line_start2) / 8;
        }

        (line_start2 - line_start1) / 8
    }

    pub fn generate_knight_moves(&self, position: i8) -> Vec<PieceMove> {
        let board_state = self.board_state.lock().unwrap();

        let positions = [
            self.get_knight_move(2, position - 17, position),
            self.get_knight_move(2, position - 15, position),
            self.get_knight_move(1, position - 10, position),
            self.get_knight_move(1, position - 6, position),
            self.get_knight_move(1, position + 6, position),
            self.get_knight_move(1, position + 10, position),
            self.get_knight_move(2, position + 15, position),
            self.get_knight_move(2, position + 17, position),
        ];

        let mut moves: Vec<PieceMove> = Vec::new();
        let knight_piece = board_state.get_piece(position);

        for new_position in positions {
            if board_state.is_valid_position(new_position) {
                let current_piece = board_state.get_piece(new_position);

                if current_piece == PieceType::Empty as i8
                    || !is_same_color(knight_piece, current_piece)
                {
                    moves.push(PieceMove::new(position, new_position));
                }
            }
        }

        moves
    }

    fn get_king_move(&self, current_position: i8, new_position: i8) -> i8 {
        if !(0..=63).contains(&new_position) {
            return -1;
        }

        // Is on the left side of the board
        if current_position % 8 == 0
            && (new_position == current_position - 1
                || new_position == current_position - 9
                || new_position == current_position + 7)
        {
            return -1;
        }

        // Is on the right side of the board
        if (current_position + 1) % 8 == 0
            && (new_position == current_position + 1
                || new_position == current_position - 7
                || new_position == current_position + 9)
        {
            return -1;
        }

        new_position
    }

    pub fn generate_king_moves(
        &self,
        opponent_moves: &[PieceMove],
        king_position: i8,
    ) -> Vec<PieceMove> {
        let positions = [
            self.get_king_move(king_position, king_position - 1),
            self.get_king_move(king_position, king_position + 1),
            self.get_king_move(king_position, king_position - 9),
            self.get_king_move(king_position, king_position - 8),
            self.get_king_move(king_position, king_position - 7),
            self.get_king_move(king_position, king_position + 7),
            self.get_king_move(king_position, king_position + 8),
            self.get_king_move(king_position, king_position + 9),
        ];

        let mut moves: Vec<PieceMove> = Vec::new();

        let board_state = self.board_state.lock().unwrap();

        let king = board_state.get_piece(king_position);
        let pawn_offset = if is_white_piece(king_position) { -8 } else { 8 };

        for position in positions {
            if position < 0 {
                continue;
            }

            // Refactor, if it's not a pawn attacking it's wasting time
            if opponent_moves
                .iter()
                .any(|_move| _move.to_position == position)
            {
                // Is a pawn straight attacking the position?
                let mut possible_pawn = board_state.get_piece(position + pawn_offset);

                let mut piece_type = get_piece_type(possible_pawn);

                if piece_type == PieceType::Pawn {
                    moves.push(PieceMove::new(king_position, position));
                    continue;
                } else if piece_type != PieceType::Empty {
                    continue;
                }

                possible_pawn = board_state.get_piece(position + (pawn_offset * 2));

                piece_type = get_piece_type(possible_pawn);

                if piece_type == PieceType::Pawn {
                    moves.push(PieceMove::new(king_position, position));
                }
            } else if board_state.is_valid_position(position) {
                let piece = board_state.get_piece(position);

                if piece == PieceType::Empty as i8 || !is_same_color(king, piece) {
                    moves.push(PieceMove::new(king_position, position));
                }
            }
        }

        drop(board_state);

        if !opponent_moves
            .iter()
            .any(|_move| _move.to_position == king_position)
        {
            self.generate_castle_moves(king, &mut moves, opponent_moves, king_position);
        }

        moves
    }

    fn is_path_clear(&self, start: i8, end: i8, step: i8) -> bool {
        let board_state = self.board_state.lock().unwrap();

        let mut i = start;

        while i != end {
            if board_state.get_piece(i) != PieceType::Empty as i8 {
                return false;
            }
            i += step;
        }

        true
    }

    fn position_is_not_attacked(&self, n: i8, opponent_moves: &[PieceMove]) -> bool {
        !opponent_moves.iter().any(|_mut| _mut.to_position == n)
    }

    fn is_able_to_castle_queen_side(&self, white_king: bool) -> bool {
        let board_state = self.board_state.lock().unwrap();

        (white_king && board_state.white_able_to_queen_side_castle())
            || (!white_king && board_state.black_able_to_queen_side_castle())
    }

    fn is_able_to_castle_king_side(&self, white_king: bool) -> bool {
        let board_state = self.board_state.lock().unwrap();

        (white_king && board_state.white_able_to_king_side_castle())
            || (!white_king && board_state.black_able_to_king_side_castle())
    }

    fn generate_castle_moves(
        &self,
        king_piece: i8,
        moves: &mut Vec<PieceMove>,
        opponent_moves: &[PieceMove],
        king_position: i8,
    ) {
        let board_state = self.board_state.lock().unwrap();

        let is_white_king = is_white_piece(king_piece);

        if (is_white_king && !board_state.white_king_moved())
            || (!is_white_king && !board_state.black_king_moved())
        {
            drop(board_state);

            let (queen_side_rook_position, king_side_rook_position) =
                if is_white_king { (56, 63) } else { (0, 7) };

            let able_to_castle_queen_side = self.is_able_to_castle_queen_side(is_white_king);
            let able_to_castle_king_side = self.is_able_to_castle_king_side(is_white_king);

            if able_to_castle_queen_side
                && self.is_path_clear(king_position - 1, queen_side_rook_position, -1)
            {
                let new_position = king_position - 2;

                // The next two squares on the left are not attacked
                if self.position_is_not_attacked(new_position, opponent_moves)
                    && self.position_is_not_attacked(king_position - 1, opponent_moves)
                {
                    moves.push(PieceMove::new(king_position, new_position));
                }
            }

            if able_to_castle_king_side
                && self.is_path_clear(king_position + 1, king_side_rook_position, 1)
            {
                let new_position = king_position + 2;

                // The next two squares on the right are not attacked
                if self.position_is_not_attacked(new_position, opponent_moves)
                    && self.position_is_not_attacked(king_position + 1, opponent_moves)
                {
                    moves.push(PieceMove::new(king_position, new_position));
                }
            }
        }
    }

    pub fn generate_queen_moves(&self, position: i8) -> Vec<PieceMove> {
        let mut moves = vec![];

        moves.extend(self.generate_bishop_moves(position));
        moves.extend(self.generate_rook_moves(position));

        moves
    }

    pub fn generate_bishop_moves(&self, position: i8) -> Vec<PieceMove> {
        let piece = self.board_state.lock().unwrap().get_piece(position);
        let mut moves: Vec<PieceMove> = vec![];

        self.generate_sliding_moves(&mut moves, piece, position, SquareOffset::TopLeft);
        self.generate_sliding_moves(&mut moves, piece, position, SquareOffset::TopRight);
        self.generate_sliding_moves(&mut moves, piece, position, SquareOffset::BottomLeft);
        self.generate_sliding_moves(&mut moves, piece, position, SquareOffset::BottomRight);

        moves
    }

    pub fn generate_rook_moves(&self, position: i8) -> Vec<PieceMove> {
        let piece = self.board_state.lock().unwrap().get_piece(position);
        let mut moves = vec![];

        self.generate_sliding_moves(&mut moves, piece, position, SquareOffset::LineAbove);
        self.generate_sliding_moves(&mut moves, piece, position, SquareOffset::Left);
        self.generate_sliding_moves(&mut moves, piece, position, SquareOffset::Right);
        self.generate_sliding_moves(&mut moves, piece, position, SquareOffset::LineBelow);

        moves
    }

    fn generate_sliding_moves(
        &self,
        moves: &mut Vec<PieceMove>,
        piece: i8,
        position: i8,
        offset: SquareOffset,
    ) {
        let board_state = self.board_state.lock().unwrap();

        for i in 0..7 {
            // Is on the right side of the board
            if (offset == SquareOffset::BottomRight || offset == SquareOffset::TopRight)
                && (position + 1) % 8 == 0
            {
                break;
            }

            // Is on the left side of the board
            if (offset == SquareOffset::BottomLeft || offset == SquareOffset::TopLeft)
                && position % 8 == 0
            {
                break;
            }

            // Should go left/right but is on the edge of the board
            if (offset == SquareOffset::Left && position % 8 == 0)
                || (offset == SquareOffset::Right && (position + 1) % 8 == 0)
            {
                break;
            }

            let new_position = position + ((i + 1) as i8 * offset.value());

            if !board_state.is_valid_position(new_position) {
                break;
            }

            let current_piece = board_state.get_piece(new_position);

            if current_piece == PieceType::Empty as i8 {
                moves.push(PieceMove::new(position, new_position));
            } else if !is_same_color(piece, current_piece) {
                moves.push(PieceMove::new(position, new_position));
                break;
            } else {
                break;
            }

            if offset != SquareOffset::LineAbove && offset != SquareOffset::LineBelow {
                let righty_offset = offset == SquareOffset::Right
                    || offset == SquareOffset::TopRight
                    || offset == SquareOffset::BottomRight;

                // Arrived at the edge of the board
                if (new_position + if righty_offset { 1 } else { 0 }) % 8 == 0 {
                    break;
                }
            }
        }
        drop(board_state);
    }

    pub fn generate_pawn_moves(&self, position: i8) -> Vec<PieceMove> {
        let mut moves: Vec<PieceMove> = Vec::new();

        let board_state = self.board_state.lock().unwrap();

        let white_piece = is_white_piece(board_state.get_piece(position));

        let offset = if white_piece { -8 } else { 8 };

        let next_line_position = position + offset;

        // Actually the pawn should be already promoted
        if !board_state.is_valid_position(next_line_position) {
            return moves;
        }

        drop(board_state);

        self.generate_pawn_regular_moves(
            &mut moves,
            next_line_position,
            offset,
            position,
            white_piece,
        );

        self.generate_pawn_capturing_moves(&mut moves, next_line_position, position, white_piece);

        self.generate_en_passant_moves(&mut moves, offset, position, white_piece);

        moves
    }

    fn generate_pawn_regular_moves(
        &self,
        moves: &mut Vec<PieceMove>,
        next_line_position: i8,
        offset: i8,
        position: i8,
        white_piece: bool,
    ) {
        let board_state = self.board_state.lock().unwrap();

        let existing_piece = board_state.get_piece(next_line_position);

        if existing_piece == PieceType::Empty as i8 {
            moves.push(PieceMove::new(position, next_line_position));

            if self.is_pawn_first_move(white_piece, position) {
                let two_lines_position = position + (offset * 2);

                let existing_piece = board_state.get_piece(two_lines_position);

                if existing_piece == PieceType::Empty as i8 {
                    moves.push(PieceMove::new(position, two_lines_position));
                }
            } else if (0..=7).contains(&next_line_position)
                || (56..=63).contains(&next_line_position)
            {
                let last_index_pos = moves.len() - 1;

                moves[last_index_pos].set_is_promotion(true);
            }
        }
    }

    fn is_pawn_first_move(&self, white_piece: bool, piece_position: i8) -> bool {
        if white_piece && (48..=55).contains(&piece_position) {
            return true;
        }

        if !white_piece && (8..=15).contains(&piece_position) {
            return true;
        }

        false
    }

    fn generate_pawn_diagonal_captures(
        &self,
        moves: &mut Vec<PieceMove>,
        next_line_position: i8,
        position: i8,
        white_piece: bool,
        left_diagonal: bool,
    ) {
        if (left_diagonal && (position % 8 == 0)) || !left_diagonal && ((position + 1) % 8 == 0) {
            return;
        }

        let diagonal = next_line_position - if left_diagonal { 1 } else { -1 };

        let board_state = self.board_state.lock().unwrap();

        let existing_piece = board_state.get_piece(diagonal);

        if board_state.is_valid_position(diagonal)
            && existing_piece != PieceType::Empty as i8
            && is_white_piece(existing_piece) != white_piece
        {
            moves.push(PieceMove::new(position, diagonal));

            if (0..=7).contains(&next_line_position) || (56..=63).contains(&next_line_position) {
                let last_index_pos = moves.len() - 1;

                moves[last_index_pos].set_is_promotion(true);
            }
        }
    }

    // Recenty refactored
    fn generate_pawn_capturing_moves(
        &self,
        moves: &mut Vec<PieceMove>,
        next_line_position: i8,
        position: i8,
        white_piece: bool,
    ) {
        self.generate_pawn_diagonal_captures(
            moves,
            next_line_position,
            position,
            white_piece,
            true,
        );

        self.generate_pawn_diagonal_captures(
            moves,
            next_line_position,
            position,
            white_piece,
            false,
        );
    }

    fn generate_en_passant_moves(
        &self,
        moves: &mut Vec<PieceMove>,
        offset: i8,
        position: i8,
        white_piece: bool,
    ) {
        let board_state = self.board_state.lock().unwrap();

        let en_passant = if white_piece {
            board_state.black_en_passant()
        } else {
            board_state.white_en_passant()
        };

        drop(board_state);

        if en_passant == -1 {
            return;
        }

        let left_square = if position % 8 != 0 { position - 1 } else { -1 };

        let right_square = if (position + 1) % 8 != 0 {
            position + 1
        } else {
            -1
        };

        let en_passant_target = if white_piece {
            en_passant + 8
        } else {
            en_passant - 8
        };

        if left_square == en_passant_target {
            let mut _move = PieceMove::new(position, left_square + offset);

            _move.is_en_passant = true;

            moves.push(_move);
        } else if right_square == en_passant_target {
            let mut _move = PieceMove::new(position, right_square + offset);

            _move.is_en_passant = true;

            moves.push(_move);
        }
    }
}
