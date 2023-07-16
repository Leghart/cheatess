import threading
import time
from typing import Any, Callable, Self

from src.log import BaseQueue


class Thread:
    def __init__(self, target: Callable[..., None], **kwargs: Any):
        self.__thread = threading.Thread(target=self.__runner, kwargs=kwargs)
        self.__target = target
        self._stop_thread = False

    def start(self) -> Self:
        self.__thread.start()
        return self

    def stop(self):
        self._stop_thread = True

    def __runner(self, **kwargs: Any):
        while True:
            if self._stop_thread:
                break
            self.__target(**kwargs)


class QueueThread(Thread):
    def __init__(
        self, queue: BaseQueue, redirect_data: Callable[..., Any], custom_fetch_queue: Callable[..., None] = None
    ):
        super().__init__(target=custom_fetch_queue or self.__fetch_queue)
        self.__queue = queue
        self.__redirect_data = redirect_data

    def __fetch_queue(self) -> None:
        while not self._stop_thread:
            try:
                if result := self.__queue.recv():
                    self.__redirect_data(result)
            except IndexError:
                pass
            time.sleep(0.1)
