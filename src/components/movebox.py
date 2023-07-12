import time
from threading import Thread

import customtkinter as ctk

from src.log import Message, MovesQueue


class MoveBox(ctk.CTkTextbox):
    def __init__(self, master: ctk.CTk, view, engine):
        super().__init__(master, height=40, width=75, corner_radius=0, font=("Helvetica", 24))
        self.parent = view
        self.engine_handler = engine
        self.current_color_button = ctk.CTkButton(
            self,
            text=f"Play as: {self.engine_handler.play_color}",
            command=self._change_color,
            fg_color=self.engine_handler.play_color,
            text_color="black",
        )
        self.current_color_button.grid(row=1, column=0, padx=20, pady=10)

        self.grid(row=2, column=0, padx=20, pady=50)
        self.configure(state="disabled")

        thread = Thread(target=self.fetch_queue)
        thread.start()

    def _change_color(self):
        new_color = self.engine_handler.toggle_color()
        self.current_color_button.configure(text=f"Play as: {new_color}")

        if new_color == "white":
            self.current_color_button.configure(text_color="black", fg_color=new_color)
        else:
            self.current_color_button.configure(text_color="white", fg_color=new_color)

        self.parent._set_image(new_color)
        # TODO change image

    def clear_logs(self) -> None:
        self.configure(state="normal")
        self.delete(1.0, ctk.END)
        self.configure(state="disabled")

    def add_log(self, message: Message) -> None:
        self.configure(state="normal")
        self.delete(0.0, ctk.END)
        self.insert("0.0", message.body, message.level)
        self.yview(ctk.END)
        self.configure(state="disabled")

    def fetch_queue(self):
        while True:
            try:
                if message := MovesQueue.recv():
                    self.add_log(message)
            except IndexError:
                pass
            time.sleep(0.1)
