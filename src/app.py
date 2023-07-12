import customtkinter as ctk

from .components.sidebar import SideBar
from .components.tabview import TabView

# https://github.com/TomSchimansky/CustomTkinter/blob/master/examples/complex_example.py

ctk.set_appearance_mode("System")
ctk.set_default_color_theme("blue")


class App(ctk.CTk):
    def __init__(self):
        super().__init__()

        self.title("Cheatess")
        self.geometry(f"{900}x{680}")

        self.grid_columnconfigure(1, weight=1)

        self.sidebar = SideBar(self)
        self.tabview = TabView(self)
