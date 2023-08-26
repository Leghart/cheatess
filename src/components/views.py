from __future__ import annotations

import os
import time
import tkinter as tk
from typing import TYPE_CHECKING, Literal, TypedDict

import customtkinter as ctk
from PIL import Image, ImageTk

from src.components.modals import HelpModal
from src.components.movebox import MoveBox
from src.log import EvaluationQueue, LogLevel, LogQueue, Message
from src.utils.cache_loader import Cache
from src.utils.engine import Engine
from src.utils.path_manager import PathManager
from src.utils.thread import QueueThread, Thread

if TYPE_CHECKING:
    from src.components.tabview import TabView


class _TEval(TypedDict):
    value: int
    type: Literal["cp", "mate"]


class ScanningView(ctk.CTkFrame):
    def __init__(self, master: TabView, engine_handler: Engine):
        super().__init__(master)
        self.master = master

        self.tab = self.master.tab("Scanning")

        self.board_visual = ctk.CTkLabel(self.tab, text="")
        self.board_visual.grid(row=0, column=0)
        self.update_board_with_image("default.png")

        self.slider_progressbar_frame = ctk.CTkFrame(self.tab, fg_color="transparent")
        self.slider_progressbar_frame.grid(row=1, column=0)

        self.thread_update_evaluation = QueueThread(
            name="EvaluationThread", queue=EvaluationQueue, redirect_data=self._update_evalbar
        ).start()

        self.evalbar_label = ctk.CTkLabel(self.tab, text="0.0")
        self.evalbar_label.grid(row=1, column=0)
        self.evalbar = ctk.CTkProgressBar(self.slider_progressbar_frame, orientation="horizontal")
        self.evalbar.grid(row=2, columnspan=2, padx=20, pady=20)
        self.evalbar.set(0.5)

        self.movebox = MoveBox(self.tab, view=self, engine=engine_handler)

    @property
    def current_thread_board_image(self):
        return self.__thread_update_board

    def create_updating_image_thread(self) -> None:
        """Starts a thread which is updating board image."""
        self.__thread_update_board = Thread(
            name="UpdateBoardThread",
            target=self.update_board_with_image,
            path_to_img="current_board.png",
        ).start()

    def update_board_with_image(self, path_to_img: str = "default.png") -> None:
        """Updates board visualization by image stored in images/{path_to_img}."""
        try:
            image = Image.open(os.path.join(PathManager.images, path_to_img))
            image_tk = ImageTk.PhotoImage(image)
            time.sleep(0.1)
            self.board_visual.configure(image=image_tk)
        except (OSError, SyntaxError):
            pass

    def _update_evalbar(self, eval_: _TEval) -> None:
        """Changes stockfish's evaluation to bar representation.

        Set an instance attribute with translated value from stockfish.
        Stockfish's value is stored as integer [-n,n] in case of 'type' is
        a centy-pawns and as integer in case check-mate's comming.
        """
        if eval_["type"] == "cp":
            scaled_val = eval_["value"] / 100
            f = lambda x: 0.05 * x + 0.5
            self.evalbar.set(f(scaled_val))
            self.evalbar_label.configure(text=scaled_val)
        else:
            if (val := eval_["value"]) > 0:
                self.evalbar.set(1)
                self.evalbar_label.configure(text=f"M{val}")
            else:
                self.evalbar.set(0)
                self.evalbar_label.configure(text=f"-M{abs(val)}")


class StockfishView(ctk.CTkFrame):
    """View of stockfish settings.

    Allows to change stockifsh engine power (level), time thinking etc.
    """

    def __init__(self, master: TabView, engine_handler: Engine):
        super().__init__(master, bg_color="red")
        self.master = master

        self.tab = self.master.tab("Stockfish")
        self.engine_handler = engine_handler
        self.cache = Cache()

        self._set_depth()
        self._set_hash()
        self._set_time_thinking()
        self._set_threads()
        self._set_level()
        self._set_path_to_stockfish_engine()
        self._set_buttons()

    def _set_depth(self):
        self._depth = tk.IntVar(value=self.cache["depth"] or 15)
        label = ctk.CTkLabel(self.tab, text="Stockfish depth (1-20):", anchor="w")
        label.grid(row=1, column=0, padx=20, pady=(10, 0))
        self.depth_spinbox = tk.Spinbox(
            self.tab,
            from_=1,
            to=20,
            width=4,
            textvariable=self._depth,
        )
        self.depth_spinbox.grid(row=2, column=0, padx=20, pady=(10, 10))

    def _set_hash(self):
        self._hash = tk.IntVar(value=self.cache["hash"] or 16)
        label = ctk.CTkLabel(self.tab, text="Amount of memory to use [MB]:", anchor="w")
        label.grid(row=1, column=1, padx=20, pady=(10, 0))

        self.hash_spinbox = tk.Spinbox(
            self.tab,
            from_=2,
            to=16384,
            width=5,
            textvariable=self._hash,
        )
        self.hash_spinbox.grid(row=2, column=1, padx=20, pady=(10, 10))

    def _set_time_thinking(self):
        self._time_thinking = tk.IntVar(value=self.cache["time_thinking"] or 1)
        label = ctk.CTkLabel(self.tab, text="Stockfish time thinking [s]:", anchor="w")
        label.grid(row=3, column=0, padx=20, pady=(10, 0))

        self.thinking_spinbox = tk.Spinbox(
            self.tab,
            from_=1,
            to=5,
            width=4,
            textvariable=self._time_thinking,
        )
        self.thinking_spinbox.grid(row=4, column=0, padx=20, pady=(10, 10))

    def _set_threads(self):
        self._threads = tk.IntVar(value=self.cache["threads"] or 1)
        label = ctk.CTkLabel(self.tab, text="Threads:", anchor="w")
        label.grid(row=3, column=1, padx=20, pady=(10, 0))

        self.threads_spinbox = tk.Spinbox(
            self.tab,
            from_=1,
            to=9,
            width=4,
            textvariable=self._threads,
        )
        self.threads_spinbox.grid(row=4, column=1, padx=20, pady=(10, 10))

    def _set_level(self):
        self._level = tk.IntVar(value=self.cache["level"] or 1500)

        self.elo_label = ctk.CTkLabel(self.tab, text=f"ELO level: {self._level.get()}", anchor="w")
        self.elo_label.grid(row=5, columnspan=2, padx=20, pady=(10, 0))

        self.level_slider = ctk.CTkSlider(
            self.tab,
            from_=1350,
            to=2850,
            number_of_steps=30,
            variable=self._level,
            command=lambda _: self.elo_label.configure(text=f"ELO level: {self._level.get()}"),
        )
        self.level_slider.grid(row=6, column=0, columnspan=2, padx=(20, 10), pady=(10, 10), sticky="ew")

    def _set_path_to_stockfish_engine(self):
        label = ctk.CTkLabel(self.tab, text="Path to downloaded stockfish engine:", anchor="w")
        label.grid(row=7, columnspan=2, padx=20, pady=(20, 0))

        self._engine_path = tk.StringVar(value=self.cache["stockfish_engine_path"])
        self.engine_path_entry = ctk.CTkEntry(self.tab,width=500, textvariable=self._engine_path)
        self.engine_path_entry.grid(row=8, columnspan=2, padx=20, pady=20)


    def _set_buttons(self):
        def set_default_values():
            self._level = tk.IntVar(value=1500)
            self.elo_label.configure(text=f"ELO level: {self._level.get()}")
            self._time_thinking = tk.IntVar(value=1)
            self._depth = tk.IntVar(value=15)
            self._hash = tk.IntVar(value=16)
            self._threads = tk.IntVar(value=1)
            self._engine_path = tk.StringVar(value="")

            self.level_slider.configure(variable=self._level)
            self.thinking_spinbox.configure(textvariable=self._time_thinking)
            self.depth_spinbox.configure(textvariable=self._depth)
            self.hash_spinbox.configure(textvariable=self._hash)
            self.threads_spinbox.configure(textvariable=self._threads)
            self.engine_path_entry.configure(textvariable=self._engine_path)

            LogQueue.send(Message("Default settings have been restored", LogLevel.WARNING))

        def save():
            level = self._level.get()
            time_thinking = self._time_thinking.get()
            depth = self._depth.get()
            _hash = self._hash.get()
            threads = self._threads.get()
            engine_path = self._engine_path.get()

            if not engine_path:
                LogQueue.send(Message("Path to engine is empty!", LogLevel.ERROR))
                return
        
            elif not os.path.isfile(engine_path):
                LogQueue.send(Message("Typed path to engine doesn't exist", LogLevel.ERROR))
                return

            self.cache["stockfish_engine_path"] = engine_path
            self.engine_handler._load_stockfish()

            self.engine_handler.update_parameters(
                level=level,
                time_thinking=time_thinking,
                depth=depth,
                memory=_hash,
                threads=threads,
            )

            self.cache["level"] = level
            self.cache["time_thinking"] = time_thinking
            self.cache["depth"] = depth
            self.cache["hash"] = _hash
            self.cache["threads"] = threads   


            LogQueue.send(Message("Updated settings", LogLevel.SUCCESS))

        to_default_button = ctk.CTkButton(self.tab, text="Reset settings", command=set_default_values)
        to_default_button.grid(row=9, column=0, padx=(20, 10), pady=(10, 10), sticky="ew")

        save_button = ctk.CTkButton(self.tab, text="Save", command=save)
        save_button.grid(row=9, column=1, padx=(20, 10), pady=(10, 10), sticky="ew")


class GeneralSettingsView(ctk.CTkFrame):
    """View of general settings, information.

    Allows to change appearance, show help modal.
    """

    def __init__(self, master: ctk.CTkTabview):
        super().__init__(master)
        self.master = master

        appearance_mode_label = ctk.CTkLabel(self.master.tab("General"), text="Appearance Mode:", anchor="w")
        appearance_mode_label.grid(row=1, column=0, padx=20, pady=(10, 0))
        appearance_mode_optionemenu = ctk.CTkOptionMenu(
            self.master.tab("General"),
            values=["Dark", "Light"],
            command=lambda nmode: ctk.set_appearance_mode(nmode),
        )
        appearance_mode_optionemenu.grid(row=2, column=0, padx=20, pady=(10, 10))

        self.hide_help_button = ctk.CTkButton(self.master.tab("General"), text="Show help", command=self.__hide_help)
        self.hide_help_button.grid(row=3, column=0)

    def __hide_help(self):
        HelpModal(self.master.tab("General"))
