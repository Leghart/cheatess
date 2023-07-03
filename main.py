import tkinter as tk

from src.app.application import Application

if __name__ == "__main__":
    root = tk.Tk()
    app = Application(root)
    root.mainloop()
