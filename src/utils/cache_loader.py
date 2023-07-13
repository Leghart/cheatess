import json
from pathlib import Path


class Singleton(type):
    _instances = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super(Singleton, cls).__call__(*args, **kwargs)
        return cls._instances[cls]


class Cache(metaclass=Singleton):
    def __init__(self) -> None:
        self.__path = Path(__file__).parent.parent.resolve() / ".cache.json"
        f = open(self.__path)
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
        with open(self.__path, "w") as f:
            data = json.dumps(self.__cache)
            f.write(data)