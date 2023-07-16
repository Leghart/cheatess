import time
from typing import Literal, Optional

import numpy as np
import pyautogui
from keras.models import Model as KerasModel
from stockfish import Stockfish

import src.utils.types as t
from src.cnn.model import init_model, predict_fen_from_image
from src.log import EvaluationQueue, LogLevel, LogQueue, Message, MovesQueue
from src.utils.board import InvalidMove, fen_to_list, get_diff_move
from src.utils.cache_loader import Cache

# from threading import Thread
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


class Engine:
    def __init__(self):
        self._board_coords = None
        self.thread: Optional[Thread] = None
        self.stop_thread: bool = False
        self.play_color: Literal["white", "black"] = "white"
        self._load_stockfish()

    def take_screenshot(self):
        x1, y1, x2, y2 = self.board_coords
        image = pyautogui.screenshot(region=(x1, y1, x2, y2))
        self._current_board = image.resize((400, 400))

    def save_screenshot(self):
        self._current_board.save("/home/leghart/projects/cheatess/images/current_board.png")

    def image_to_array(self):
        self.take_screenshot()
        return np.array(self._current_board)

    def scan_screen(self) -> None:
        image = self.image_to_array()
        self.current_fen = predict_fen_from_image(image, self.model).strip(" ")
        if self.previous_fen == self.current_fen:
            time.sleep(0.02)
            return

        if self.previous_fen is None:
            self.previous_fen = self.current_fen
            return

        tmp_img = image = self.image_to_array()
        if self.current_fen != predict_fen_from_image(tmp_img, self.model).strip(" "):
            return

        try:
            move = get_diff_move(fen_to_list(self.previous_fen), fen_to_list(self.current_fen), self.white_on_move)
            self.save_screenshot()
        except InvalidMove:
            try:
                image = self.image_to_array()
                self.current_fen = predict_fen_from_image(image, self.model).strip(" ")
                move = get_diff_move(fen_to_list(self.previous_fen), fen_to_list(self.current_fen), self.white_on_move)
            except Exception:
                raise
        except Exception as exc:
            LogQueue.send(Message(f"Invalid move: {str(exc)}", LogLevel.ERROR))
            return

        self.moves_counter += 1

        try:
            self.stockfish.make_moves_from_current_position([move])
            best_move = self.stockfish.get_best_move()
            # TODO Show which exactly piece should move
            evaluation = self.stockfish.get_evaluation()

            EvaluationQueue.send(evaluation)

            # doesnt show opponent's best moves
            if self.moves_counter % 2 == 0:
                msg_dict: t.StatisticsDict = {
                    "wdl_stats": self.stockfish.get_wdl_stats(),
                    "top_moves": self.__translate_top_moves(self.stockfish.get_top_moves(3)),
                    "best_move": self.__get_piece_from_position(best_move),
                }
                MovesQueue.send(msg_dict)
                # TODO add arrows to img

        except Exception as err:
            LogQueue.send(Message(str(err), LogLevel.ERROR))

        self.previous_fen = self.current_fen

    def start_scaning_thread(self):
        if not self.board_coords:
            LogQueue.send(Message("Get board coordinates first", LogLevel.ERROR))
            return

        LogQueue.send(Message("Started scanning board", LogLevel.SUCCESS))

        self.take_screenshot()  # initial screenshot
        self._load_stockfish()
        self.model = init_model()
        self.previous_fen = None
        self.current_fen = None
        self.moves_counter = 0 if self.play_color == "white" else 1
        self.white_on_move = self.play_color == "white"

        self.thread = Thread(self.scan_screen).start()

    def stop_scaning_thread(self):
        if not self.thread:
            LogQueue.send(Message("You have to start scanning first", LogLevel.ERROR))
            return

        LogQueue.send(Message("Stopped scanning board", LogLevel.ERROR))

        self.thread.stop()

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

    def update_parameters(self, level: int, time_thinking: int, depth: int, memory: int, threads: int):
        config = {"UCI_Elo": level, "Minimum Thinking Time": time_thinking, "Hash": memory, "Threads": threads}
        self.stockfish.update_engine_parameters(config)
        self.stockfish.set_depth(depth)

    def _load_stockfish(self):
        path_to_engine = Cache()["stockfish_engine_path"]
        self.stockfish = Stockfish(path_to_engine)

    def __get_piece_from_position(self, position: str) -> str:
        piece_to_move = self.stockfish.get_what_is_on_square(position[:2])
        return f"{PIECES[piece_to_move]}{position[2:]}"

    def __translate_top_moves(self, top_moves: list[t.TopMoves]) -> list[t.FinalTopMoves]:
        result = []

        for top_move in top_moves:
            batch: t.FinalTopMoves = {}
            batch["move"] = self.__get_piece_from_position(top_move["Move"])

            if top_move["Mate"]:
                if (cp := top_move["Centipawn"]) > 0:
                    batch["evaluation"] = f"M{cp}"
                else:
                    batch["evaluation"] = f"-M{abs(cp)}"
            else:
                batch["evaluation"] = top_move["Centipawn"]

            result.append(batch)

        return result
