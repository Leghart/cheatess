import time
from threading import Thread

import customtkinter as ctk
from PIL import Image, ImageTk

from src.components.movebox import MoveBox
from src.utils.engine import Engine


class ScanningView(ctk.CTkFrame):
    def __init__(self, master: ctk.CTkTabview, engine_handler: Engine):
        super().__init__(master)

        self.board_visual = ctk.CTkLabel(self.master.tab("Scanning"))
        self._set_image(engine_handler.play_color)
        self.board_visual.grid(row=0, column=0)

        self.movebox = MoveBox(self.master.tab("Scanning"), view=self, engine=engine_handler)

        self.start_thread_update_board()

    def _set_image(self, color: str):
        ...
        # if color == "white":
        #     img = "/home/leghart/projects/cheatess/white_start.png"
        # else:
        #     img = "/home/leghart/projects/cheatess/black_start.png"

        # image = Image.open(img)
        # image_tk = ImageTk.PhotoImage(image)
        # self.board_visual.configure(image=image_tk)

    def _update_board(self):
        while True:
            try:
                image = Image.open("/home/leghart/projects/cheatess/current_board.png")
                image_tk = ImageTk.PhotoImage(image)
                time.sleep(0.2)
                self.board_visual.configure(image=image_tk)
            except OSError:
                continue

    def start_thread_update_board(self):
        t = Thread(target=self._update_board)
        t.start()


class StockfishView(ctk.CTkFrame):
    def __init__(self, master: ctk.CTkTabview):
        super().__init__(master)
        self.master = master
        self.optionmenu_1 = ctk.CTkOptionMenu(
            self.master.tab("Stockfish"),
            dynamic_resizing=False,
            values=["Value 1", "Value 2", "Value Long Long Long"],
        )
        self.optionmenu_1.grid(row=0, column=0, padx=20, pady=(20, 10))
        self.combobox_1 = ctk.CTkComboBox(
            self.master.tab("Stockfish"), values=["Value 1", "Value 2", "Value Long....."]
        )
        self.combobox_1.grid(row=1, column=0, padx=20, pady=(10, 10))
        self.string_input_button = ctk.CTkButton(
            self.master.tab("Stockfish"), text="Open CTkInputDialog", command=self.open_input_dialog_event
        )
        self.string_input_button.grid(row=2, column=0, padx=20, pady=(10, 10))

    def open_input_dialog_event(self):
        dialog = ctk.CTkInputDialog(text="DUNNO")
        print("DuNNO dialog: ", dialog.get_input())


class GeneralSettingsView(ctk.CTkFrame):
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
        appearance_mode_optionemenu.grid(row=2, column=0, padx=20, pady=(10, 10))
