import time
from threading import Thread
from typing import Literal, Optional

import numpy as np
import pyautogui
from stockfish import Stockfish

from src.cnn.model import init_model, predict_fen_from_image
from src.log import LogLevel, LogQueue, Message, MovesQueue
from src.utils.board import InvalidMove, fen_to_list, get_diff_move
from src.utils.cache_loader import CacheLoader

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


class Engine:
    def __init__(self):
        self._board_coords = None
        self._thread: Optional[Thread] = None
        self.stop_thread: bool = False
        self.play_color: Literal["white", "black"] = "white"
        self._load_stockfish()

    def take_screenshot(self):
        x1, y1, x2, y2 = self.board_coords
        image = pyautogui.screenshot(region=(x1, y1, x2, y2))
        self._current_board = image.resize((400, 400))

    def save_screenshot(self):
        self._current_board.save("/home/leghart/projects/cheatess/current_board.png")

    def image_to_array(self):
        self.take_screenshot()
        return np.array(self._current_board)

    def scan_screen(self) -> None:
        self.take_screenshot()
        model = init_model()
        previous_fen = None
        current_fen = None

        moves_counter = 0 if self.play_color == "white" else 1
        white_on_move = self.play_color == "white"
        while True:
            if self.stop_thread:
                return

            image = self.image_to_array()
            current_fen = predict_fen_from_image(image, model).strip(" ")
            if previous_fen == current_fen:
                time.sleep(0.02)
                continue

            if previous_fen is None:
                previous_fen = current_fen
                continue

            tmp_img = image = self.image_to_array()
            if current_fen != predict_fen_from_image(tmp_img, model).strip(" "):
                continue

            try:
                move = get_diff_move(fen_to_list(previous_fen), fen_to_list(current_fen), white_on_move)
                self.save_screenshot()
            except InvalidMove:
                try:
                    image = self.image_to_array()
                    current_fen = predict_fen_from_image(image, model).strip(" ")
                    move = get_diff_move(fen_to_list(previous_fen), fen_to_list(current_fen), white_on_move)
                except Exception:
                    raise
            except Exception as exc:
                msg = Message(f"Invalid move: {str(exc)}", LogLevel.ERROR)
                LogQueue.send(msg)
                continue

            moves_counter += 1

            try:
                self.stockfish.make_moves_from_current_position([move])
                best_move = self.stockfish.get_best_move()
                # TODO Show which exactly piece should move
                piece_to_move = self.stockfish.get_what_is_on_square(best_move[:2])

                # doesnt show opponent's best moves
                if moves_counter % 2 == 0:
                    msg = Message(f"{PIECES[piece_to_move]}{best_move[2:]}", LogLevel.INFO)
                    MovesQueue.send(msg)
                    # TODO add arrows to img

            except Exception as err:
                msg = Message(str(err), LogLevel.ERROR)
                LogQueue.send(msg)

            previous_fen = current_fen

    def start_scaning_thread(self):
        if not self.board_coords:
            msg = Message("Get board coordinates first", LogLevel.ERROR)
            LogQueue.send(msg)
            return

        msg = Message("Started scanning board", LogLevel.SUCCESS)
        LogQueue.send(msg)
        self._thread = Thread(target=self.scan_screen)
        self._thread.start()

    def stop_scaning_thread(self):
        if not self._thread:
            msg = Message("You have to start scanning first", LogLevel.ERROR)
            LogQueue.send(msg)
            return

        msg = Message("Stopped scanning board", LogLevel.ERROR)
        LogQueue.send(msg)
        self.stop_thread = True
        self._thread.join()

    def toggle_color(self) -> str:
        if self.play_color == "white":
            self.play_color = "black"
        else:
            self.play_color = "white"

        return self.play_color

    @property
    def board_coords(self):
        return self._board_coords

    @board_coords.setter
    def board_coords(self, coords: tuple[int]) -> None:
        self._board_coords = coords

    def _load_stockfish(self):
        path_to_engine = CacheLoader()["stockfish_engine_path"]
        self.stockfish = Stockfish(path_to_engine)
