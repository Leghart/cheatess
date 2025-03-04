import customtkinter as ctk

from .components.modals import HelpModal
from .components.sidebar import SideBar
from .components.tabview import TabView
from .utils.cache_loader import Cache
from .utils.context import Context

# https://github.com/TomSchimansky/CustomTkinter/blob/master/examples/complex_example.py

ctk.set_appearance_mode("System")
ctk.set_default_color_theme("blue")


class App(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.cache = Cache()
        self.ctx = Context("127.0.0.1:5555")

        self.title("Cheatess")
        self.geometry(f"{900}x{730}")

        self.grid_columnconfigure(1, weight=1)

        self.sidebar = SideBar(self)
        self.tabview = TabView(self)

        if not self.cache["hide_help"]:
            HelpModal(self)
