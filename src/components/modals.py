import tkinter as tk

import customtkinter as ctk

from src.utils.cache_loader import Cache

msg = """
Welcome to Cheatess.

How to use it?
At first get coordinates of board by click `Scan board`. Try to make a rectangle 
as accurately as you can (focus on corners - the more accurate the scan, the less chance of misreading).

Remember to set on which side you play - as white or black by click switch. You have to do it before next step.
White color is set by default.

After that click `Start scanning` - it runs a thread to collect images of board and runs stockfish.
Analysis starts after 2nd move on both sides. After match you can stop thead to reduce 
CPU usage by click `stop scanning`. 

In the "Stockfish" tab, you can configure the settings as you wish (it's cached, so
you can only do it the first time).

"""


class HelpModal(ctk.CTkToplevel):
    def __init__(self, master):
        super().__init__(master, height=400, width=500)
        super(tk.Toplevel, self).title("Help")
        self.master = master
        self.cache = Cache()

        self.info = ctk.CTkLabel(self, text=msg)
        self.info.grid(row=0, column=0, padx=10, pady=10)

        self.__choice = tk.BooleanVar(value=self.cache["hide_help"])
        self.hide_modal_checker = ctk.CTkSwitch(
            self,
            text="Dont show it again",
            variable=self.__choice,
            command=self.__save_in_cache,
        )
        self.hide_modal_checker.grid(row=1, column=0, pady=20, padx=10)

    def __save_in_cache(self) -> None:
        self.cache["hide_help"] = self.__choice.get()
