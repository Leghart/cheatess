import json
from pathlib import Path


class Singleton(type):
    _instances = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super(Singleton, cls).__call__(*args, **kwargs)
        return cls._instances[cls]


class CacheLoader(metaclass=Singleton):
    def __init__(self) -> None:
        f = open(Path(__file__).parent.parent.resolve() / ".cache.json")
        self.cache = json.load(f)
        f.close()

    def __str__(self):
        return str(self.cache)

    def __getitem__(self, key):
        try:
            return self.cache[key]
        except KeyError:
            return None
