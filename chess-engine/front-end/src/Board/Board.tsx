import React, { MouseEventHandler, useEffect, useRef, useState } from "react";

import BoardPiece from "./BoardPiece";
import { TBoard, TBoardPiece } from "./types";

//@ts-ignore
import captureAudio from "../assets/sound/capture.mp3";
//@ts-ignore
import moveAudio from "../assets/sound/move-self.mp3";

import http from "../http-common";

import "./board.scss";

const LINES = [0, 1, 2, 3, 4, 5, 6, 7];
const COLUMNS: { [key: number]: string } = {
  0: "A",
  1: "B",
  2: "C",
  3: "D",
  4: "E",
  5: "F",
  6: "G",
  7: "H",
};

const EMPTY_PIECE: TBoardPiece = {
  moves: [],
  position: -1,
  type: null,
};

const get_empty_piece = (position: number) => {
  const piece: TBoardPiece = JSON.parse(JSON.stringify(EMPTY_PIECE));
  piece.position = position;

  return piece;
};

const playMoveAudio = (capture: boolean) => {
  let audio;

  if (capture) {
    audio = new Audio(captureAudio);
  } else {
    audio = new Audio(moveAudio);
  }
  audio.play();
};

const isNotAnAvailableMove = (availableMoves: number[], position: number) =>
  !availableMoves.find((move) => move === position);

const Board = () => {
  const [selectedPiece, setSelectedPiece] = useState<TBoardPiece | null>(null);
  const [board, setBoard] = useState<TBoard>({
    blackCaptures: [],
    whiteCaptures: [],
    pieces: [],
    winner: null
  });

  const onPieceSelect = (piece: TBoardPiece) => {
    console.log("Piece click: ", piece.type, piece.position);
    console.log(piece.moves);
    if (selectedPiece === piece) {
      setSelectedPiece(null);
    } else {
      if (selectedPiece) {
        togglePieceAvailableMoves(selectedPiece);
      }

      setSelectedPiece(piece);
    }
    togglePieceAvailableMoves(piece);
  };

  const togglePieceAvailableMoves = (piece: TBoardPiece) => {
    piece.moves.forEach((move) => {
      const className = board.pieces[move].type ? "capture-receptor" : "empty-receptor";

      const cell = document.querySelector(`.cell[data-pos='${move}']`) as HTMLDivElement;

      // cell.onclick = () => onCellClick(cell, move.row, move.column);
      cell.classList.toggle(className);

      const cellPiece = document.querySelector(
        `.cell[data-pos='${move}'] button.piece-button`
      ) as HTMLDivElement;

      cellPiece?.classList.toggle("disabled");
    });
  };

  const onMovePiece = (cell: HTMLDivElement, cellPosition: number) => {
    if (selectedPiece) {
      const { position, moves } = selectedPiece;

      if (isNotAnAvailableMove(moves, cellPosition)) {
        return;
      }

      const copy_board: TBoard = JSON.parse(JSON.stringify(board));

      let capture = false;
      if (copy_board.pieces[cellPosition].type !== null) {
        capture = true;
      }

      copy_board.pieces[position] = get_empty_piece(position);

      selectedPiece.position = cellPosition;

      copy_board.pieces[cellPosition] = selectedPiece;

      setSelectedPiece(null);
      setBoard(copy_board);

      const cellPiece = document.querySelector(
        `.cell[data-pos='${position}'] button.piece-button.disabled`
      ) as HTMLDivElement;

      cellPiece?.classList.remove("disabled");

      playMoveAudio(capture);

      // console.log(`Capture`, capture);
      // if (capture) {
      //   console.log(cell);
      //   cell.classList.remove("capture-receptor");
      // }

      togglePieceAvailableMoves(selectedPiece);
      // add loading before sendin request
      movePiece(position, cellPosition);
      // currentTarget.onclick = null;
      // send request to server and update the state with the result
    }
  };

  const movePiece = (from: number, to: number) => {
    http
      .post<TBoard>("/board/move/piece", {
        from,
        to,
      })
      .then((response) => response.data)
      .then((data) => {
        setBoard(data)
      });
  };

  useEffect(() => {
    http
      .get<TBoard>("/board")
      .then((response) => response.data)
      .then((data) => setBoard(data));
  }, []);

  useEffect(() => {
    if(board.winner) {
      console.log(`${board.winner} wins!`)
    }
  }, [board])

  return (
    <div id="board">
      {LINES.map((i) => (
        <div key={i} className="row">
          {LINES.map((j) => (
            <div
              key={j}
              className="cell"
              data-pos={i * 8 + j}
              onClick={(e) => onMovePiece(e.currentTarget, i * 8 + j)}
            >
              {j === 0 && (
                <span className={`row-index ${(i + 1) % 2 !== 0 ? "white" : ""}`}>{i + 1}</span>
              )}
              {i === 7 && (
                <span className={`column-index ${(j + 1) % 2 === 0 ? "white" : ""}`}>
                  {COLUMNS[j]}
                </span>
              )}
              {board.pieces[i * 8 + j] && board.pieces[i * 8 + j].type !== null ? (
                <BoardPiece boardPiece={board.pieces[i * 8 + j]} onClick={onPieceSelect} />
              ) : (
                <div className="move-dot"></div>
              )}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
};

export default Board;
