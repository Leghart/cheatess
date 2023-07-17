from __future__ import annotations

import queue
from enum import StrEnum
from typing import Optional, Self, TypeVar

T = TypeVar("T")


class LogLevel(StrEnum):
    SUCCESS = "success"
    ERROR = "error"
    WARNING = "warning"
    INFO = "info"


class Message:
    def __init__(self, body: str, level: LogLevel = LogLevel.INFO):
        self.body = body
        self.level = level


class QMeta(type):
    queues: dict[str, queue.Queue] = {}

    def __new__(cls, *args, **kwargs) -> Self:
        new = super().__new__(cls, *args, **kwargs)

        if new not in cls.queues:
            cls.queues[new.__name__] = queue.Queue()

        return new


class BaseQueue(metaclass=QMeta):
    @classmethod
    def recv(cls) -> Optional[T]:
        try:
            return cls.queues[cls.__name__].get(False)
        except queue.Empty:
            return None

    @classmethod
    def send(cls, msg: T) -> None:
        cls.queues[cls.__name__].put(msg)


class LogQueue(BaseQueue):
    ...


class MovesQueue(BaseQueue):
    ...


class EvaluationQueue(BaseQueue):
    ...
