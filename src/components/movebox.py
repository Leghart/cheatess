from __future__ import annotations

import time
from threading import Thread
from typing import TYPE_CHECKING

import customtkinter as ctk

import src.utils.types as t
from src.log import LogLevel, MovesQueue

if TYPE_CHECKING:
    from src.components.views import ScanningView
    from src.utils.engine import Engine


class MoveBox(ctk.CTkTextbox):
    def __init__(self, master: ctk.CTk, view: ScanningView, engine: Engine):
        super().__init__(master, height=200, width=600, corner_radius=0, font=("Helvetica", 22))
        self.parent = view
        self.engine_handler = engine

        self.grid(row=3, column=0, padx=20)
        self.configure(state="disabled")

        thread = Thread(target=self.fetch_queue)
        thread.start()

    def clear_logs(self) -> None:
        self.configure(state="normal")
        self.delete(1.0, ctk.END)
        self.configure(state="disabled")

    def _set_layout(self, msg_dict: t.StatisticsDict) -> str:
        top_moves_msg = ""
        for batch in msg_dict["top_moves"]:
            top_moves_msg += f"\n{round(batch['evaluation']/100,1)}: {batch['move']}"

        msg = f"""
Best move: {msg_dict['best_move']}
Top moves: {top_moves_msg} 
Win-Draw-Loss stats (%): {'-'.join([str(int(x/10)) for x in msg_dict['wdl_stats']])}
        """
        return msg

    def add_log(self, msg_dict: t.StatisticsDict) -> None:
        self.configure(state="normal")
        self.delete(0.0, ctk.END)
        msg = self._set_layout(msg_dict)
        self.insert("0.0", msg, LogLevel.INFO)
        self.yview(ctk.END)
        self.configure(state="disabled")

    def fetch_queue(self):
        while True:
            try:
                if msg_dict := MovesQueue.recv():
                    self.add_log(msg_dict)
            except IndexError:
                pass
            time.sleep(0.1)
