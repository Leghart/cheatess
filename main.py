import os

from src.app import App


def cleanup():
    if os.path.exists("/home/leghart/projects/cheatess/images/current_board.png"):
        os.remove("/home/leghart/projects/cheatess/images/current_board.png")

    my_dir = "/home/leghart/projects/cheatess/"
    for file in os.listdir(my_dir):
        if file.startswith(".screenshot"):
            os.remove(os.path.join(my_dir, file))


if __name__ == "__main__":
    cleanup()
    app = App()
    app.mainloop()
    os._exit(0)
