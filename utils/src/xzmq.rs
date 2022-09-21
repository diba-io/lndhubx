pub use zmq::{Context as ZmqContext, Socket as ZmqSocket, SocketType};

#[derive(Clone)]
pub struct SocketContext {
    context: ZmqContext,
}

impl SocketContext {
    pub fn new() -> Self {
        Self {
            context: ZmqContext::new(),
        }
    }

    fn create_listening_socket(&self, address: &str, socket_type: SocketType) -> ZmqSocket {
        let socket = match self.context.socket(socket_type) {
            Ok(created) => created,
            Err(err) => {
                panic!(
                    "Failed to create a {:?} socket listening on: {}, reason: {:?}",
                    socket_type, address, err
                );
            }
        };
        match socket.bind(address) {
            Ok(()) => socket,
            Err(err) => {
                panic!("Failed to bind socket to {}, reason: {:?}", address, err);
            }
        }
    }

    fn create_connecting_socket(
        &self,
        address: &str,
        socket_type: SocketType,
        subscriber_topic: Option<&[u8]>,
    ) -> ZmqSocket {
        let socket = match self.context.socket(socket_type) {
            Ok(created) => created,
            Err(err) => {
                panic!(
                    "Failed to create a {:?} socket connecting to: {}, reason: {:?}",
                    socket_type, address, err
                );
            }
        };
        if socket_type == zmq::SUB {
            let topic = subscriber_topic.unwrap_or(&[]);
            if let Err(err) = socket.set_subscribe(topic) {
                panic!("Failed to set subscribe on the socket, reason {:?}", err);
            }
        }
        match socket.connect(address) {
            Ok(()) => socket,
            Err(err) => {
                panic!("Failed to connect socket to {}, reason: {:?}", address, err);
            }
        }
    }

    pub fn create_publisher(&self, address: &str) -> ZmqSocket {
        self.create_listening_socket(address, zmq::PUB)
    }

    pub fn create_subscriber(&self, address: &str) -> ZmqSocket {
        self.create_subscriber_with_topic(address, &[])
    }

    pub fn create_subscriber_with_topic(&self, address: &str, topic: &[u8]) -> ZmqSocket {
        self.create_connecting_socket(address, zmq::SUB, Some(topic))
    }

    pub fn create_push(&self, address: &str) -> ZmqSocket {
        self.create_connecting_socket(address, zmq::PUSH, None)
    }

    pub fn create_pull(&self, address: &str) -> ZmqSocket {
        self.create_listening_socket(address, zmq::PULL)
    }

    pub fn create_request(&self, address: &str) -> ZmqSocket {
        self.create_connecting_socket(address, zmq::REQ, None)
    }

    pub fn create_response(&self, address: &str) -> ZmqSocket {
        self.create_listening_socket(address, zmq::REP)
    }
}

impl Default for SocketContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn send_as_json<T: ?Sized>(socket: &ZmqSocket, message: &T)
where
    T: serde::Serialize + std::fmt::Debug,
{
    let payload = match serde_json::to_string(message) {
        Ok(serialized) => serialized,
        Err(err) => {
            panic!(
                "Failed to serialize a message: {:?} into a json payload, reason: {:?}",
                message, err
            );
        }
    };
    if let Err(err) = socket.send(payload.as_str(), 0x00) {
        panic!(
            "Failed to send a message: {:?} as a json payload, reason: {:?}",
            message, err
        );
    }
}

pub fn send_as_bincode<T: ?Sized>(socket: &ZmqSocket, message: &T)
where
    T: serde::Serialize + std::fmt::Debug,
{
    let payload = match bincode::serialize(message) {
        Ok(serialized) => serialized,
        Err(err) => {
            panic!(
                "Failed to serialize a message: {:?} into a bincode payload, reason: {:?}",
                message, err
            );
        }
    };
    if let Err(err) = socket.send(payload, 0x00) {
        panic!(
            "Failed to send a message: {:?} as a bincode payload, reason: {:?}",
            message, err
        );
    }
}

pub fn send_multipart_as_bincode<T: ?Sized>(socket: &ZmqSocket, message: &T)
where
    T: serde::Serialize + std::fmt::Debug,
{
    let payload = match bincode::serialize(message) {
        Ok(serialized) => vec![vec![], vec![], serialized],
        Err(err) => {
            panic!(
                "Failed to serialize a message: {:?} into a bincode payload, reason: {:?}",
                message, err
            );
        }
    };
    if let Err(err) = socket.send_multipart(payload, 0x00) {
        panic!(
            "Failed to send a message: {:?} as a multipart bincode payload, reason: {:?}",
            message, err
        );
    }
}
