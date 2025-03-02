from __future__ import annotations

import json
from typing import Any, Generic, Type, TypeVar

from src.utils.path_manager import PathManager

T = TypeVar("T")


class Singleton(type, Generic[T]):
    """Singleton design pattern.

    Always uses the same instance of object.
    """

    _instances: dict[Singleton[T], T] = {}

    def __call__(cls, *args: Any, **kwargs: Any) -> T:
        if cls not in cls._instances:
            cls._instances[cls] = super(Singleton, cls).__call__(*args, **kwargs)
        return cls._instances[cls]


class Cache(metaclass=Singleton):
    """Class which allows to save and load data from .cache.json file.

    You can have access to key from json by using nested get items e.g.

    >> cache = Cache()
    >> cache["stockfish"]["level"] = 1900
    """

    def __init__(self) -> None:
        f = open(PathManager.cache)
        self.__cache = json.load(f)
        f.close()

    def __str__(self):
        return str(self.__cache)

    def __getitem__(self, key):
        try:
            return self.__cache[key]
        except KeyError:
            return None

    def __setitem__(self, key, value):
        self.__cache[key] = value
        self.__save()

    def __save(self):
        with open(PathManager.cache, "w") as f:
            data = json.dumps(self.__cache)
            f.write(data)
