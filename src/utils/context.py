import dataclasses
import enum
import json
from typing import Self

import zmq


class MsgKey(enum.StrEnum):
    Configurate = "Configurate"
    Game = "Game"
    Region = "Region"
    Ok = "Ok"
    Ping = "Ping"


@dataclasses.dataclass
class ProtocolInterface:
    key: MsgKey
    message: str

    def serialize(self) -> dict[str, str]:
        return dataclasses.asdict(self)

    @classmethod
    def deserialize(cls, data: dict[str, str]) -> Self:
        return cls(key=data["key"], message=data["message"])


class Context:
    def __init__(self, addr: str):
        self._context = zmq.Context()
        self._socket = self._context.socket(zmq.REQ)
        self._socket.connect(f"tcp://{addr}")

    def send(self, data: ProtocolInterface):
        self._socket.send_string(json.dumps(data))

    def recv(self) -> ProtocolInterface:
        """Receive data from `core`. Block main thread until get response."""
        response = self._socket.recv_string()
        data = json.loads(response)

        return ProtocolInterface(key=MsgKey(data["key"]), message=data["message"])
