from dataclasses import dataclass
from pathlib import Path


@dataclass
class PathManager:
    root: Path = Path().parent.parent
    images: Path = root / "images"
    current_board_image: Path = images / "current_board.png"
    cache: Path = root / "src" / ".cache.json"
