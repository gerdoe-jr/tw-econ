use std::net::SocketAddr;
use std::sync::{Mutex, Arc};
use tokio::net::TcpStream;
use chrono::{NaiveTime, Utc};
use std::thread::JoinHandle;


#[derive(thiserror::Error, Debug, Clone)]
pub enum EconError {
    #[error("Wrong password.")]
    WrongPassword,
    #[error("No response.")]
    NoResponse
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EconMessage {
    timestamp: NaiveTime,
    category: String,
    content: String,
}

impl EconMessage {
    pub fn new<T: Into<NaiveTime>, S: Into<String>>(timestamp: T, category: S, content: S) -> Self {
        Self {
            timestamp: timestamp.into(),
            category: category.into(),
            content: content.into(),
        }
    }

    pub fn from_string<S: Into<String>>(msg: S) -> Option<Self> {
        match sscanf::scanf!(msg.into(), "[{}][{}]: {}", String, String, String) {
            Some((utc, ctg, cnt)) => {
                Some(Self::new(
                    NaiveTime::parse_from_str(&utc, "%H:%M:%S").unwrap(),
                    ctg,
                    cnt,
                ))
            },
            _ => None
        }
    }

    pub fn set_timestamp<T: Into<NaiveTime>>(&mut self, value: T) -> &Self {
        self.timestamp = value.into();

        self
    }

    pub fn set_category<S: Into<String>>(&mut self, value: S) -> &Self {
        self.category = value.into();

        self
    }

    pub fn set_content<S: Into<String>>(&mut self, value: S) -> &Self {
        self.content = value.into();

        self
    }

    pub fn get_timestamp(&self) -> NaiveTime {
        self.timestamp
    }

    pub fn get_category(&self) -> String {
        self.category.clone()
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }
}

pub struct EconConnection {
    stream: Arc<Mutex<TcpStream>>,

    vec_in: Arc<Mutex<Vec<String>>>,
    vec_out: Arc<Mutex<Vec<EconMessage>>>,

    connected: Arc<Mutex<bool>>
}

impl EconConnection {
    pub async fn new<N: Into<SocketAddr>, S: Into<String>>(address: N, password: S) -> Result<Self, EconError>
    {
        let vec_in = Vec::new();
        let mut vec_out = Vec::new();

        let address = address.into();
        let password = password.into() + "\n";

        let stream = match TcpStream::connect(address).await {
            Ok(s) => {

                let mut buffer: [u8; 1024] = [0; 1024];

                let size = s.try_read(&mut buffer).unwrap();
                if !std::str::from_utf8(&buffer[..size]).unwrap().contains("Enter") {
                    return Err(EconError::NoResponse);
                }

                s.try_write(password.as_bytes()).expect(&format!("Can't send password for address: {:}", address));

                let size = s.try_read(&mut buffer).unwrap();
                if std::str::from_utf8(&buffer[..size]).unwrap().contains("Wrong") {
                    return Err(EconError::WrongPassword);
                }

                let msg = EconMessage::new(
                    Utc::now().time(),
                    "tw-econ",
                    &format!("Connected to '{}'", address)
                );

                vec_out.push(msg);

                s
            },
            _ => {
                return Err(EconError::NoResponse);
            }
        };

        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
            vec_in: Arc::new(Mutex::new(vec_in)),
            vec_out: Arc::new(Mutex::new(vec_out)),
            connected: Arc::new(Mutex::new(true)),
        })
    }

    pub async fn connect(&self) -> tokio::task::JoinHandle<()> {
        let stream = self.stream.as_ref();
        let vec_in = self.vec_in.as_ref();
        let vec_out = self.vec_out.as_ref();
        let connected = self.connected.as_ref();

        tokio::spawn(async move {
            while *connected.lock().unwrap() {
                let mut buffer: [u8; 4096] = [0; 4096];
                unsafe {
                    match stream.lock().unwrap().try_read(&mut buffer) {
                        Ok(size) => match std::str::from_utf8_unchecked(&buffer[..size]) {
                            words => {
                                for word in words.to_string().split('\n') {
                                    let msg = match EconMessage::from_string(word) {
                                        Some(m) => {
                                            Some(m)
                                        },
                                        _ => {
                                            if !word.is_empty() {
                                                Some(EconMessage::new(Utc::now().time(), "tw-econ", &word))
                                            }
                                            else {
                                                None
                                            }
                                        }
                                    };

                                    if let Some(m) = msg {
                                        vec_out.lock().unwrap().push(m);
                                    }
                                }
                            }
                        },
                        Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {},
                        _ => {}
                    };
                }

                if let Some(received) = vec_in.lock().unwrap().pop() {
                    let mut received = received.into_bytes().into_boxed_slice();

                    stream.lock().unwrap().try_write(&mut received).expect("Can't read streambuffer");
                }
            }
        })
    }

    pub async fn disconnect(&mut self) {
        *self.connected.as_ref().lock().unwrap() = true;
    }

    pub async fn send_message<S: Into<String>>(&self, message: S) {
        self.vec_in.as_ref().lock().unwrap().push(message.into());
    }

    pub async fn recv_message(&self) -> Option<EconMessage> {
        self.vec_out.as_ref().lock().unwrap().pop()
    }
}