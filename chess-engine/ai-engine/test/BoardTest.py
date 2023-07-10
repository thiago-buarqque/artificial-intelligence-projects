import unittest

from src.Board import Board
from src.Piece import (PieceColor, PieceType)


class BoardTest(unittest.TestCase):
    def test_piece_placement(self):
        board = Board()

        board.place_piece(0, PieceColor.Black | PieceType.Bishop)
        board.place_piece(63, PieceColor.White | PieceType.Bishop)

        self.assertEqual(
            board.get_piece(0),
            PieceColor.Black | PieceType.Bishop
        )

        self.assertEqual(
            board.get_piece(63),
            PieceColor.White | PieceType.Bishop
        )

        board.move_piece(0, 63)

        self.assertEqual(
            board.get_piece(0),
            PieceType.Empty
        )

        self.assertEqual(
            board.get_piece(63),
            PieceColor.Black | PieceType.Bishop
        )

        with self.assertRaises(IndexError):
            board.move_piece(0, 60)

    def test_load_fen(self):
        board = Board()

        # Assert initial FEN

        # Mudar para outra FEN nao inicial
        board.load_position(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")


if __name__ == '__main__':
    unittest.main()