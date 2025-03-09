from __future__ import annotations

import queue
from enum import StrEnum
from typing import Any, Type, TypeVar

T = TypeVar("T")


class LogLevel(StrEnum):
    SUCCESS = "success"
    ERROR = "error"
    WARNING = "warning"
    INFO = "info"


class Message:
    """Class to store message data with color information."""

    def __init__(self, body: str, level: LogLevel = LogLevel.INFO):
        self.body = body
        self.level = level


class QMeta(type):
    """Metaclass to store every queue in class attribute.

    Allows to get data from queue by class, not instance.
    """

    queues: dict[str, queue.Queue] = {}

    def __new__(cls, *args: Any, **kwargs: Any) -> Type[T]:
        new = super().__new__(cls, *args, **kwargs)

        if new not in cls.queues:
            cls.queues[new.__name__] = queue.Queue()

        return new


class BaseQueue(metaclass=QMeta):
    """Abstract base queue.

    This class should be always overriden to get a specific, named queue.
    """

    @classmethod
    def recv(cls) -> Any:
        try:
            return cls.queues[cls.__name__].get(False)
        except queue.Empty:
            return None

    @classmethod
    def send(cls, msg: T) -> None:
        cls.queues[cls.__name__].put(msg)


class LogQueue(BaseQueue):
    """Queue of logs, errors from app."""


class MovesQueue(BaseQueue):
    """Queue of best moves from stockifish analysis."""


class EvaluationQueue(BaseQueue):
    """Queue of evaluation bar values stockifish."""


class ImageArrayQueue(BaseQueue):
    """Queue of board images (as arrays)."""
