from __future__ import annotations

import time
from enum import StrEnum
from typing import Optional

import cv2
import numpy as np
import pyautogui
from PIL.Image import Image
from stockfish import Stockfish

import src.utils.types as t
from src.cnn.model import init_model, predict_fen_from_image
from src.log import EvaluationQueue, ImageArrayQueue, LogLevel, LogQueue, Message, MovesQueue
from src.utils.board import fen_to_list, get_diff_move
from src.utils.cache_loader import Cache
from src.utils.image_modifier import ImageModifier
from src.utils.path_manager import PathManager
from src.utils.thread import Thread

PIECES = {
    Stockfish.Piece.BLACK_PAWN: "\u265F",
    Stockfish.Piece.BLACK_ROOK: "\u265C",
    Stockfish.Piece.BLACK_KNIGHT: "\u265E",
    Stockfish.Piece.BLACK_BISHOP: "\u265D",
    Stockfish.Piece.BLACK_KING: "\u265A",
    Stockfish.Piece.BLACK_QUEEN: "\u265B",
    Stockfish.Piece.WHITE_PAWN: "\u2659",
    Stockfish.Piece.WHITE_ROOK: "\u2656",
    Stockfish.Piece.WHITE_KNIGHT: "\u2658",
    Stockfish.Piece.WHITE_BISHOP: "\u2657",
    Stockfish.Piece.WHITE_KING: "\u2654",
    Stockfish.Piece.WHITE_QUEEN: "\u2655",
}


class PlayColor(StrEnum):
    BLACK = "black"
    WHITE = "white"


def print_board(fen: str) -> None:
    for row in fen.split("/"):
        for p in row:
            if p.isdigit():
                print("." * int(p), end="")
            else:
                print(p, end="")
        print(end="\n")
    print("\n")


def mse(img1: np.ndarray, img2: np.ndarray) -> float:
    diff = cv2.subtract(img1, img2)
    err = np.sum(diff**2)
    mse = err / (400 * 400)
    return mse


class Engine:
    def __init__(self):
        self._image_modifier = ImageModifier()
        self._board_coords = None
        self.scanning_thread: Optional[Thread] = None
        self.calculating_thread: Optional[Thread] = None
        self.stop_thread: bool = False
        self.play_color: PlayColor = PlayColor.WHITE

        self._prev_img: Optional[Image] = None
        self._current_board_img: Optional[Image] = None
        self._correct_moves: list[str] = []

        self.previous_fen: str = ""
        self.current_fen: str = ""
        self.first_move: bool = True

        # self._load_stockfish()

    def take_screenshot(self):
        x1, y1, x2, y2 = self.board_coords
        image = pyautogui.screenshot(region=(x1, y1, x2, y2))
        self._current_board_img = image.resize((400, 400))

    def save_board_image(self):
        self._current_board_img.save(PathManager.current_board_image)

    def calculate(self) -> None:
        image: Optional[np.ndarray]
        if self.first_move:
            image = np.array(self._current_board_img)
        else:
            image = ImageArrayQueue.recv()
            if image is None:
                time.sleep(0.5)
                return

        self.current_fen = predict_fen_from_image(image, self.model).strip(" ")

        if self.first_move:
            self.play_color = self.__detect_play_color()
            self.moves_counter = 0 if self.play_color == PlayColor.WHITE else -1
            self.white_on_move = self.play_color == PlayColor.WHITE

        self.first_move = False

        if not self.previous_fen:
            self.previous_fen = self.current_fen
            return

        try:
            move = get_diff_move(fen_to_list(self.previous_fen), fen_to_list(self.current_fen), self.white_on_move)
            self.save_board_image()
        # except IndexError:
        #     # TODO? add retry?
        #     print("======== RETRY ======")
        #     self._load_stockfish()
        #     self.stockfish.make_moves_from_current_position(self._correct_moves)

        #     self.take_screenshot()
        #     self.current_fen = predict_fen_from_image(np.array(self._current_board_img), self.model).strip(" ")
        #     move = get_diff_move(fen_to_list(self.previous_fen), fen_to_list(self.current_fen), self.white_on_move)
        #     self.save_board_image()

        except Exception as exc:
            if msg := str(exc):
                LogQueue.send(Message(f"Invalid move: {msg}", LogLevel.ERROR))
            return

        self.moves_counter += 1

        try:
            self.stockfish.make_moves_from_current_position([move])
            best_move = self.stockfish.get_best_move()
            evaluation = self.stockfish.get_evaluation()
            EvaluationQueue.send(evaluation)

            if self.moves_counter % 2 == 0:
                msg_dict: t.StatisticsDict = {
                    "wdl_stats": self.stockfish.get_wdl_stats(),
                    "top_moves": self.__translate_top_moves(self.stockfish.get_top_moves(3)),
                    "best_move": self.__get_piece_from_position(best_move),
                }
                MovesQueue.send(msg_dict)
                self._current_board_img = self._image_modifier.draw(
                    best_move[:2], best_move[2:], white_on_move=self.white_on_move
                )
                self.save_board_image()

        except Exception as err:
            LogQueue.send(Message(str(err), LogLevel.ERROR))

        self._correct_moves.append(move)
        self.previous_fen = self.current_fen
        self.first_move = False

    def scan(self) -> None:
        self.take_screenshot()
        if self._prev_img is None:
            self._prev_img = self._current_board_img

        prev, curr = np.array(self._prev_img), np.array(self._current_board_img)
        if (mse(prev, curr)) != 0.0:
            ImageArrayQueue.send(curr)

        self._prev_img = self._current_board_img
        time.sleep(0.2)

    def start_scaning(self):
        if not self.board_coords:
            LogQueue.send(Message("Get board coordinates first", LogLevel.ERROR))
            return

        LogQueue.send(Message("Started scanning board", LogLevel.SUCCESS))

        self.take_screenshot()  # initial screenshot
        self._load_stockfish()
        self.model = init_model()

        self.previous_fen: str = ""
        self.current_fen: str = ""
        self.first_move: bool = True

        self.scanning_thread = Thread(name="ScanThread", target=self.scan).start()
        self.calculating_thread = Thread(name="CalcThread", target=self.calculate).start()

    def stop_scaning_thread(self):
        if not self.scanning_thread and not self.calculating_thread:
            LogQueue.send(Message("You have to start scanning first", LogLevel.ERROR))
            return

        LogQueue.send(Message("Stopped scanning board", LogLevel.ERROR))

        self.scanning_thread.stop()
        self.calculating_thread.stop()

    def update_parameters(self, level: int, time_thinking: int, depth: int, memory: int, threads: int):
        config = {"UCI_Elo": level, "Minimum Thinking Time": time_thinking, "Hash": memory, "Threads": threads}
        self.stockfish.update_engine_parameters(config)
        self.stockfish.set_depth(depth)

    @property
    def is_loaded_coords(self):
        return self.board_coords is not None

    @property
    def board_coords(self):
        return self._board_coords

    @board_coords.setter
    def board_coords(self, coords: tuple[int]) -> None:
        self._board_coords = coords

    def _load_stockfish(self):
        path_to_engine = Cache()["stockfish_engine_path"]
        self.stockfish = Stockfish(path_to_engine)

    def __get_piece_from_position(self, position: str) -> str:
        piece_to_move = self.stockfish.get_what_is_on_square(position[:2])
        return f"{PIECES[piece_to_move]}{position[2:]}"

    def __translate_top_moves(self, top_moves: list[t.TopMoves]) -> list[t.FinalTopMoves]:
        result = []

        for top_move in top_moves:
            batch: t.FinalTopMoves = {"move": "", "evaluation": ""}
            batch["move"] = self.__get_piece_from_position(top_move["Move"])

            if mate := top_move["Mate"]:
                if mate > 0:
                    batch["evaluation"] = f"M{mate}"
                else:
                    batch["evaluation"] = f"-M{abs(mate)}"
            else:
                batch["evaluation"] = str(top_move["Centipawn"])

            result.append(batch)

        return result

    def __detect_play_color(self) -> PlayColor:
        """Detect player pieces color.

        If first row starts with 'R' it means that opponent has white
        pieces (you play as black).
        """
        if self.current_fen.startswith("R"):
            return PlayColor.BLACK

        return PlayColor.WHITE
