from __future__ import annotations

import tkinter as tk
from abc import abstractmethod
from typing import TYPE_CHECKING

import customtkinter as ctk

from src.utils.context import MsgKey, ProtocolInterface

if TYPE_CHECKING:
    from .sidebar import SideBar


class CustomButton(ctk.CTkButton):
    @abstractmethod
    def action(self): ...


class SelectRegionButton(CustomButton):
    def __init__(self, parent: SideBar):
        self.parent = parent
        super().__init__(parent, text="Mark region", command=self.action)
        self.grid(row=1, column=0, padx=20, pady=10)
        self.configure(state="")

    def action(self):
        self.parent.master_screen.deiconify()
        self.parent.master.withdraw()

        self._snip_surface = ctk.CTkCanvas(
            self.parent.picture_frame, cursor="cross", bg="grey11"
        )
        self._snip_surface.pack(fill=tk.BOTH, expand=tk.YES)

        self._snip_surface.bind("<ButtonPress-1>", self._on_button_press)
        self._snip_surface.bind("<B1-Motion>", self._on_snip_drag)
        self._snip_surface.bind("<ButtonRelease-1>", self._on_button_release)

        self.parent.master_screen.attributes("-fullscreen", True)
        self.parent.master_screen.attributes("-alpha", 0.3)
        self.parent.master_screen.lift()
        self.parent.master_screen.attributes("-topmost", True)

    def _on_button_press(self, event: tk.Event):
        # Reset threads
        # self.engine_handler.scanning_thread = None
        # self.engine_handler.calculating_thread = None
        # self.engine_handler.board_coords = None
        self.parent.snippet.start_x = self._snip_surface.canvasx(event.x)
        self.parent.snippet.start_y = self._snip_surface.canvasy(event.y)
        self._snip_surface.create_rectangle(
            0, 0, 1, 1, outline="SkyBlue1", width=3, fill="grey60"
        )

    def _on_snip_drag(self, event: tk.Event):
        self.parent.snippet.current_x, self.parent.snippet.current_y = (
            event.x,
            event.y,
        )
        self._snip_surface.coords(
            1,
            self.parent.snippet.start_x,
            self.parent.snippet.start_y,
            self.parent.snippet.current_x,
            self.parent.snippet.current_y,
        )

    def _on_button_release(self, event: tk.Event):
        if not self.parent.snippet.is_frame_set():
            return
        # self.parent.engine_handler.board_coords = self.parent.snippet.get_frame()
        self.parent.stop.configure(
            state=(
                "normal" if self.parent.engine_handler.is_loaded_coords else "disabled"
            )
        )
        self.parent.start.configure(
            state=(
                "normal" if self.parent.engine_handler.is_loaded_coords else "disabled"
            )
        )

        self._snip_surface.destroy()
        self.parent.master_screen.withdraw()
        self.parent.master.deiconify()

        self.parent.master.ctx.send(
            ProtocolInterface(
                key=MsgKey.Region, message=str(self.parent.snippet.get_frame())
            ).serialize()
        )
        assert self.parent.master.ctx.recv().key is MsgKey.Ok
        return event
        self.engine_handler.take_screenshot()
        self.engine_handler.save_board_image()
        self.master.tabview.scanning_view.create_updating_image_thread()

        self._snip_surface.destroy()
        self.master_screen.withdraw()
        self.master.deiconify()

        return event


class StartScanningButton(CustomButton):
    def __init__(self, parent: SideBar):
        super().__init__(parent, text="Start", command=self.action, fg_color="green")

        self.grid(row=2, column=0, padx=20, pady=10)
        self.configure(state="disabled")

    def action(self):
        # self.engine_handler.start_scaning
        # TODO
        ...


class StopScanningButton(CustomButton):
    def __init__(self, parent: SideBar):
        super().__init__(parent, text="Stop", command=self.action, fg_color="red")
        self.grid(row=2, column=1, padx=20, pady=10)
        self.configure(state="normal")

    def action(self):
        # self.engine_handler.stop_scanning
        # TODO
        ...


class ClearLogsButton(CustomButton):
    def __init__(self, parent: SideBar):
        self.parent = parent
        super().__init__(
            parent,
            text="Clear",
            command=self.action,
            fg_color="white",
            bg_color="black",
        )
        self.grid(row=6, column=0, columnspan=2, sticky="nsew", padx=20, pady=10)

    def action(self):
        self.parent.logbox.clear_logs


class PingButton(CustomButton):
    def __init__(self, parent: SideBar):
        super().__init__(parent, text="ping", command=self.action, fg_color="blue")
        self.grid(row=1, column=1, padx=20, pady=10)
        self.configure(state="")

    def action(self):
        print("todo")
