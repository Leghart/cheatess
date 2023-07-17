from __future__ import annotations

import tkinter as tk
from typing import TYPE_CHECKING

import customtkinter as ctk

from src.components.logbox import LogBox
from src.utils.engine import Engine, PlayColor
from src.utils.snippet_machine import SnippetMachine

if TYPE_CHECKING:
    from src.app import App


class SideBar(ctk.CTkFrame):
    engine_handler = Engine()
    snippet = SnippetMachine()

    def __init__(self, master: App):
        super().__init__(master, corner_radius=0)

        self.master_screen = tk.Toplevel(self)
        self.master_screen.withdraw()

        self.picture_frame = tk.Frame(self.master_screen)
        self.picture_frame.pack(fill=tk.BOTH, expand=tk.YES)

        self.grid(row=0, column=0, rowspan=30, sticky="nsew")

        self.undo_move_button = ctk.CTkButton(self, text="Undo move")
        self.undo_move_button.grid(row=1, column=0, padx=20, pady=10)

        self._color: PlayColor = tk.StringVar(value=PlayColor.WHITE.value)
        self.current_color_switch = ctk.CTkSwitch(
            self,
            text="White/Black",
            variable=self._color,
            command=self.engine_handler.toggle_color,
        )
        self.current_color_switch.grid(row=1, column=1, padx=20, pady=10)

        self.scan_screen_button = ctk.CTkButton(self, text="Get board coordinates", command=self._create_screen_canvas)
        self.scan_screen_button.grid(row=3, column=0, padx=20, pady=10)

        self.start_scanning_button = ctk.CTkButton(
            self, text="Start scanning", command=self.engine_handler.start_scaning_thread, fg_color="green"
        )
        self.start_scanning_button.grid(row=4, column=0, padx=20, pady=10)
        self.stop_scanning_button = ctk.CTkButton(
            self, text="Stop scanning", command=self.__stop_scanning, fg_color="red"
        )
        self.stop_scanning_button.grid(row=4, column=1, padx=20, pady=10)

        self.logbox = LogBox(self)
        self.clear_logs_button = ctk.CTkButton(
            self, text="Clear logs", command=self.logbox.clear_logs, text_color="black", fg_color="white"
        )
        self.clear_logs_button.grid(row=6, column=0, columnspan=2, sticky="nsew", padx=20, pady=10)

    def _create_screen_canvas(self):
        self.master_screen.deiconify()
        self.master.withdraw()

        self.snip_surface = ctk.CTkCanvas(self.picture_frame, cursor="cross", bg="grey11")
        self.snip_surface.pack(fill=tk.BOTH, expand=tk.YES)

        self.snip_surface.bind("<ButtonPress-1>", self.__on_button_press)
        self.snip_surface.bind("<B1-Motion>", self.__on_snip_drag)
        self.snip_surface.bind("<ButtonRelease-1>", self.__on_button_release)

        self.master_screen.attributes("-fullscreen", True)
        self.master_screen.attributes("-alpha", 0.3)
        self.master_screen.lift()
        self.master_screen.attributes("-topmost", True)

    def __on_button_press(self, event):
        # Reset threads
        self.engine_handler.thread = None
        self.engine_handler.board_coords = None

        self.snippet.start_x = self.snip_surface.canvasx(event.x)
        self.snippet.start_y = self.snip_surface.canvasy(event.y)
        self.snip_surface.create_rectangle(0, 0, 1, 1, outline="SkyBlue1", width=3, fill="grey60")

    def __on_snip_drag(self, event):
        self.snippet.current_x, self.snippet.current_y = (event.x, event.y)
        self.snip_surface.coords(
            1, self.snippet.start_x, self.snippet.start_y, self.snippet.current_x, self.snippet.current_y
        )

    def __on_button_release(self, event):
        if not self.snippet.is_frame_set():
            return

        self.engine_handler.board_coords = self.snippet.get_frame()
        self.engine_handler.take_screenshot()
        self.engine_handler.save_screenshot()
        self.master.tabview.scanning_view.update_board_with_image("current_board.png")

        self.snip_surface.destroy()
        self.master_screen.withdraw()
        self.master.deiconify()

        return event

    def __stop_scanning(self):
        self.engine_handler.stop_scaning_thread()
        self.master.tabview.scanning_view.thread_update_board.stop()
        self.master.tabview.scanning_view.update_board_with_image()
