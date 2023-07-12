import customtkinter as ctk

from src.components.views import GeneralSettingsView, ScanningView, StockfishView


class TabView(ctk.CTkTabview):
    def __init__(self, master):
        super().__init__(master, height=650)
        self.master = master
        self.grid(row=0, column=1, padx=(20, 0), pady=(20, 0), sticky="nsew")
        self.add("Scanning")
        self.add("Stockfish")
        self.add("General")

        self.tab("Scanning").grid_columnconfigure(0, weight=1)
        self.tab("Stockfish").grid_columnconfigure(0, weight=1)
        self.tab("General").grid_columnconfigure(0, weight=1)

        self.scanning_view = ScanningView(self, self.master.sidebar.engine_handler)
        self.stockfish_view = StockfishView(self)
        self.settings_view = GeneralSettingsView(self)
