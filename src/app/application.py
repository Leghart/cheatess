import threading as th
import time
import tkinter as tk

import numpy as np
import pyautogui
from stockfish import Stockfish

from src.cnn.model import init_model, predict_fen_from_image
from src.utils.board import InvalidMove, fen_to_list, get_diff_move

stockfish = Stockfish("/home/leghart/projects/chessify_utils/stockfish_15.1_linux_x64/stockfish-ubuntu-20.04-x86-64")


board_cords = []


def take_bounded_screenshot(x1, y1, x2, y2):
    image = pyautogui.screenshot(region=(x1, y1, x2, y2))
    image_resized = image.resize((400, 400))
    return np.array(image_resized)


def scan_screen() -> None:
    model = init_model()
    previous_fen = None
    current_fen = None

    while True:
        image = take_bounded_screenshot(*board_cords)
        current_fen = predict_fen_from_image(image, model).strip(" ")
        if previous_fen == current_fen:
            time.sleep(0.02)
            continue

        if previous_fen is None:
            previous_fen = current_fen
            continue

        tmp_img = image = take_bounded_screenshot(*board_cords)
        if current_fen != predict_fen_from_image(tmp_img, model).strip(" "):
            continue

        try:
            move = get_diff_move(fen_to_list(previous_fen), fen_to_list(current_fen))
        except InvalidMove:
            try:
                image = take_bounded_screenshot(*board_cords)
                current_fen = predict_fen_from_image(image, model).strip(" ")
                move = get_diff_move(fen_to_list(previous_fen), fen_to_list(current_fen))
            except InvalidMove:
                print("INVALID MOVE")
                continue

        stockfish.make_moves_from_current_position([move])
        print("BEST MOVE: ", stockfish.get_best_move())
        previous_fen = current_fen


def start_scaning_thread():
    t1 = th.Thread(target=scan_screen)
    t1.start()


class Application:
    def __init__(self, master):
        self.snip_surface = None
        self.master = master
        self.start_x = None
        self.start_y = None
        self.current_x = None
        self.current_y = None

        self.master.geometry("400x50+200+200")
        self.master.title("Chessify")

        self.menu_frame = tk.Frame(master)
        self.menu_frame.pack(fill=tk.BOTH, expand=tk.YES, padx=1, pady=1)

        self.buttonBar = tk.Frame(self.menu_frame, bg="")
        self.buttonBar.pack()

        self.snipButton = tk.Button(
            self.buttonBar, width=5, height=5, command=self.create_screen_canvas, background="green"
        )
        self.snipButton.pack()

        self.master_screen = tk.Toplevel(self.master)
        self.master_screen.withdraw()
        self.picture_frame = tk.Frame(self.master_screen)
        self.picture_frame.pack(fill=tk.BOTH, expand=tk.YES)

    def create_screen_canvas(self):
        self.master_screen.deiconify()
        self.master.withdraw()

        self.snip_surface = tk.Canvas(self.picture_frame, cursor="cross", bg="grey11")
        self.snip_surface.pack(fill=tk.BOTH, expand=tk.YES)

        self.snip_surface.bind("<ButtonPress-1>", self.on_button_press)
        self.snip_surface.bind("<B1-Motion>", self.on_snip_drag)
        self.snip_surface.bind("<ButtonRelease-1>", self.on_button_release)

        self.master_screen.attributes("-fullscreen", True)
        self.master_screen.attributes("-alpha", 0.3)
        self.master_screen.lift()
        self.master_screen.attributes("-topmost", True)

    def on_button_release(self, event):
        global board_cords
        x1 = x2 = y1 = y2 = None
        if self.start_x <= self.current_x and self.start_y <= self.current_y:
            x1 = self.start_x
            y1 = self.start_y
            x2 = self.current_x - self.start_x
            y2 = self.current_y - self.start_y

        elif self.start_x >= self.current_x and self.start_y <= self.current_y:
            x1 = self.current_x
            y1 = self.start_y
            x2 = self.start_x - self.current_x
            y2 = self.current_y - self.start_y

        elif self.start_x <= self.current_x and self.start_y >= self.current_y:
            x1 = self.start_x
            y1 = self.current_y
            x2 = self.current_x - self.start_x
            y2 = self.start_y - self.current_y

        elif self.start_x >= self.current_x and self.start_y >= self.current_y:
            x1 = self.current_x
            y1 = self.current_y
            x2 = self.start_x - self.current_x
            y2 = self.start_y - self.current_y

        board_cords = (x1, y1, x2, y2)
        start_scaning_thread()
        self.exit_screenshot_mode()
        return event

    def exit_screenshot_mode(self):
        self.snip_surface.destroy()
        self.master_screen.withdraw()
        self.master.deiconify()

    def on_button_press(self, event):
        self.start_x = self.snip_surface.canvasx(event.x)
        self.start_y = self.snip_surface.canvasy(event.y)
        self.snip_surface.create_rectangle(0, 0, 1, 1, outline="red", width=3, fill="maroon3")

    def on_snip_drag(self, event):
        self.current_x, self.current_y = (event.x, event.y)
        self.snip_surface.coords(1, self.start_x, self.start_y, self.current_x, self.current_y)
