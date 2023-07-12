import os

from src.app import App


def cleanup():
    if os.path.exists("/home/leghart/projects/cheatess/current_board.png"):
        os.remove("/home/leghart/projects/cheatess/current_board.png")


if __name__ == "__main__":
    cleanup()
    app = App()
    app.mainloop()
    os._exit(0)
