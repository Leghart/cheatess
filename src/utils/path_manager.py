from dataclasses import dataclass
from pathlib import Path


@dataclass
class PathManager:
    root: Path = Path().parent.parent
    images: Path = root / "images"
    current_board_image: Path = images / "current_board.png"
    models: Path = root / "models"
    default_models: Path = models / "default"
    custom_models: Path = models / "custom"
    cache: Path = root / "src" / ".cache.json"
