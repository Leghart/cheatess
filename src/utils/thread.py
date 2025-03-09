import threading
import time
from typing import Any, Callable, Optional, Self, Type

from src.log import BaseQueue


class Thread:
    """Class representing a Thread with additional functionalities.

    Allows to create a thread with a passed function that will be called
    until the stop event is dispatched.
    """

    def __init__(self, name: str, target: Callable[..., None], sleep: float = 0.1, **kwargs: Any):
        """Initializes an instance with passed arguments.

        name: name of the thread
        target: function which has to use/process retrived data
        sleep: time in seconds, how long the function should wait before
               calling it again
        kwargs: key-arguments which should be passed to `target` function
        """
        self.__thread = threading.Thread(name=name, target=self.__runner, kwargs=kwargs)
        self.__target = target
        self.__stop_event = threading.Event()
        self._sleep_time = sleep

    def start(self) -> Self:
        """Starts a thread and return itself (as handler)."""
        self.__thread.start()
        return self

    def stop(self) -> None:
        """Set a stop flag for the thread."""
        self.__stop_event.set()

    def is_stopped(self) -> bool:
        """Checks whether thread stop flag was set."""
        return self.__stop_event.is_set()

    def status(self) -> bool:
        """Checks whether thread is still alive."""
        return self.__thread.is_alive()

    def __runner(self, **kwargs: Any) -> None:
        """Runs a target function until stop flag wouldn't be set.

        After each calling, sleep a chosen time to not to overload the
        processor.
        """
        while True:
            if self.is_stopped():
                break
            self.__target(**kwargs)
            time.sleep(self._sleep_time)


class QueueThread(Thread):
    """Extends the Thread class to collect input from queue.

    There could be passed a `custom_fetch_queue` that will be the main
    runner of thread.
    """

    def __init__(
        self,
        name: str,
        queue: Type[BaseQueue],
        redirect_data: Callable[..., Any],
        custom_fetch_queue: Optional[Callable[..., None]] = None,
    ):
        """Initializes an instance with passed arguments.

        name: name of the thread
        queue: queue class from which data should be retrieved
        redirect_data: function which has to use/process retrived data
        custom_fetch_queue: function which could be used instead of
                            default '__fetch_queue'. This function should
                            work in a loop to provide continous collecting
                            data.
        """
        super().__init__(name=name, target=custom_fetch_queue or self.__fetch_queue)
        self.__queue = queue
        self.__redirect_data = redirect_data

    def __fetch_queue(self) -> None:
        """Gets data from queue and pass it to `target` function.

        In case of any exception, skips them (it's treated as incorrect
        or incomplete data from queue).
        """
        while not self.is_stopped():
            try:
                if result := self.__queue.recv():
                    self.__redirect_data(result)
            except Exception:
                pass
            time.sleep(self._sleep_time)
