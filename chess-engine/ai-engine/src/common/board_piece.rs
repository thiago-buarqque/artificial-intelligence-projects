use super::piece_move::PieceMove;

#[derive(Debug, Clone)]
pub struct BoardPiece {
    fen: char,
    moves: Vec<PieceMove>,
    position: i8,
    value: i8,
    white: bool,
}

impl BoardPiece {
    pub fn new(fen: char, moves: Vec<PieceMove>, position: i8, value: i8, white: bool) -> Self {
        BoardPiece {
            fen,
            moves,
            position,
            value,
            white,
        }
    }

    pub fn get_fen(&self) -> char {
        self.fen
    }
    pub fn get_moves(&mut self) -> &mut Vec<PieceMove> {
        &mut self.moves
    }

    pub fn get_immutable_moves(&self) -> Vec<PieceMove> {
        self.moves.clone()
    }

    pub fn get_position(&self) -> i8 {
        self.position
    }
    pub fn get_value(&self) -> i8 {
        self.value
    }
    pub fn is_white(&self) -> bool {
        self.white
    }
}
