import threading
import time
from typing import Any, Callable, Self

from src.log import BaseQueue


class Thread:
    def __init__(self, name: str, target: Callable[..., None], **kwargs: Any):
        self.__thread = threading.Thread(name=name, target=self.__runner, kwargs=kwargs)
        self.__target = target
        self.__stop_event = threading.Event()

    def start(self) -> Self:
        self.__thread.start()
        return self

    def stop(self):
        self.__stop_event.set()

    def is_stopped(self) -> bool:
        return self.__stop_event.is_set()

    def status(self) -> bool:
        return self.__thread.is_alive()

    def __runner(self, **kwargs: Any):
        while True:
            if self.is_stopped():
                break
            self.__target(**kwargs)


class QueueThread(Thread):
    def __init__(
        self,
        name: str,
        queue: BaseQueue,
        redirect_data: Callable[..., Any],
        custom_fetch_queue: Callable[..., None] = None,
    ):
        super().__init__(name=name, target=custom_fetch_queue or self.__fetch_queue)
        self.__queue = queue
        self.__redirect_data = redirect_data

    def __fetch_queue(self) -> None:
        while not self.is_stopped():
            try:
                if result := self.__queue.recv():
                    self.__redirect_data(result)
            except Exception:
                pass
            time.sleep(0.1)
