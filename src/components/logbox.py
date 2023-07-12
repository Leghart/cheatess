import time
from threading import Thread

import customtkinter as ctk

from src.log import LogLevel, LogQueue, Message


class LogBox(ctk.CTkTextbox):
    def __init__(self, master: ctk.CTk):
        super().__init__(master, height=400, corner_radius=0, font=("Helvetica", 16))
        self.grid(row=5, column=0, sticky="nsew", columnspan=2, padx=20, pady=10)
        self.tag_config(LogLevel.SUCCESS, foreground="green")
        self.tag_config(LogLevel.ERROR, foreground="red")
        self.tag_config(LogLevel.WARNING, foreground="yellow")

        self.configure(state="disabled")
        self.start_fetch_queue()

    def clear_logs(self) -> None:
        self.configure(state="normal")
        self.delete(1.0, ctk.END)
        self.configure(state="disabled")

    def add_log(self, message: Message) -> None:
        self.configure(state="normal")
        self.insert(ctk.END, message.body + "\n", message.level)
        self.yview(ctk.END)
        self.configure(state="disabled")

    def fetch_queue(self):
        while True:
            try:
                if message := LogQueue.recv():
                    self.add_log(message)
            except IndexError:
                pass
            time.sleep(0.1)

    def start_fetch_queue(self):
        t = Thread(target=self.fetch_queue)
        t.start()
