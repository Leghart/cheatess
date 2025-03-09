use serde::{Deserialize, Serialize};
use zmq::Socket;

#[derive(Serialize, Deserialize, Debug)]
pub enum MsgKey {
    Configurate,
    Ping,
    Game,
    Region,
    Ok,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtocolInterface {
    pub key: MsgKey,
    pub message: String,
}

pub struct Context {
    hanlder: zmq::Context,
    socket: Option<Socket>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            hanlder: zmq::Context::new(),
            socket: None,
        }
    }

    pub fn connect(&mut self) {
        let sock = self.hanlder.socket(zmq::REP).expect("Socket error");
        sock.bind("tcp://127.0.0.1:5555").expect("Fatal error");
        self.socket = Some(sock);
    }

    pub fn recv(&self) -> ProtocolInterface {
        let msg = self
            .socket
            .as_ref()
            .expect("Socket has not been created")
            .recv_string(0)
            .expect("none")
            .unwrap();
        serde_json::from_str(&msg).unwrap()
    }

    pub fn send(&self, msg: ProtocolInterface) {
        let response_str = serde_json::to_string(&msg).unwrap();
        self.socket
            .as_ref()
            .expect("Socket has not been created")
            .send(&response_str, 0)
            .expect("sendind error");
    }
}
