import os

from src.app import App
from src.utils.path_manager import PathManager


def cleanup():
    if os.path.exists(PathManager.current_board_image):
        os.remove(PathManager.current_board_image)

    for file in os.listdir(PathManager.root):
        if file.startswith(".screenshot"):
            os.remove(os.path.join(PathManager.root, file))


if __name__ == "__main__":
    cleanup()
    app = App()
    app.mainloop()
    os._exit(0)
