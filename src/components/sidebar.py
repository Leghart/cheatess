from __future__ import annotations

import tkinter as tk
from typing import TYPE_CHECKING

import customtkinter as ctk

from src.components import buttons
from src.components.logbox import LogBox
from src.utils.snippet_machine import SnippetMachine

if TYPE_CHECKING:
    from src.app import App


class Engine:
    def start_scaning(self): ...
    def is_loaded_coords(self):
        return True


class SideBar(ctk.CTkFrame):
    engine_handler = Engine()
    snippet = SnippetMachine()

    def __init__(self, master: App):
        super().__init__(master, corner_radius=0)
        self.master: App = master
        self.logbox = LogBox(self)
        self._snip_surface: ctk.CTkCanvas

        self.master_screen = tk.Toplevel(self)
        self.master_screen.withdraw()

        self.picture_frame = tk.Frame(self.master_screen)
        self.picture_frame.pack(fill=tk.BOTH, expand=tk.YES)

        self.grid(row=0, column=0, rowspan=30, sticky="nsew")

        self.ping = buttons.PingButton(self)
        self.scan = buttons.SelectRegionButton(self)
        self.start = buttons.StartScanningButton(self)
        self.stop = buttons.StopScanningButton(self)
        self.clear_logs = buttons.ClearLogsButton(self)
