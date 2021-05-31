use std::net::TcpStream;
use std::io::prelude::*;
use std::thread;

use chrono::{NaiveTime, Timelike, Utc};


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EconMessage {
    timestamp: NaiveTime,
    category: String,
    content: String,
}

impl EconMessage {
    pub fn from_string(msg: &String) -> Option<Self> {
        match sscanf::scanf!(msg.clone(), "[{}][{}]: {}", String, String, String) {
            Some((utc, ctg, cnt)) => {
                Some(EconMessage {
                    timestamp: NaiveTime::parse_from_str(&utc, "%H:%M:%S").unwrap(),
                    category: ctg,
                    content: cnt,
                })
            },
            _ => None
        }
    }

    pub fn from_string_with_current_time(msg: &String) -> Self {
        let now = Utc::now();
        
        match sscanf::scanf!(msg.clone(), "[{}]: {}", String, String) {
            Some((ctg, cnt)) => {
                EconMessage {
                    timestamp: NaiveTime::from_hms(now.hour(), now.minute(), now.second()),
                    category: ctg,
                    content: cnt,
                }
            },
            _ => {
                EconMessage {
                    timestamp: NaiveTime::from_hms(now.hour(), now.minute(), now.second()),
                    category: String::from("tw-econ"),
                    content: msg.to_string(),
                }
            }
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("[{}][{}]: {}", self.timestamp, self.category, self.content)
    }

    // requires a good stable runtime-formatter
    // wish i will find it
    // or do it myself...
    // pub fn to_string_fmt(&self, format: String) -> String {
    //     unimplemented!();
    // }

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

pub struct EconConnection;

impl EconConnection {
    pub fn connect(address: std::net::SocketAddr, 
        password: String, 
        disconnect_msg: String) 
        -> 
        (std::sync::mpsc::Sender<String>, 
        std::sync::mpsc::Receiver<EconMessage>) 
    {
        use std::sync::mpsc::{self};
        let (tx, rx) = mpsc::channel::<EconMessage>();
        let sup_tx = tx.clone();
        let (stx, srx) = mpsc::channel::<String>();
        
        thread::spawn(move || {
            let addr = address.clone();
            let mut password = password; password.push('\n');
            let mut stream = match TcpStream::connect(addr) {
                Ok(mut s) => {
                    let mut found = false;
                    while !found {
                        let mut buffer: [u8; 1024] = [0; 1024];
                        s.read(&mut buffer).expect(&format!("Can't read streambuffer for address: {:}", addr));
                        found = std::str::from_utf8(&buffer).unwrap().contains("Enter password:");
                    }
                    
                    s.write(password.as_bytes()).expect(&format!("Can't send password for address: {:}", addr));

                    let msg = EconMessage::from_string_with_current_time(&format!("[tw-econ]: Connected to '{}'", addr));

                    tx.send(msg).expect(&format!("Can't write streambuffer for address: {:}", addr));

                    s.set_nonblocking(true).expect(&format!("Can't set non-blocking mode for address: {:}", addr));

                    s
                },
                _ => {
                    let msg = EconMessage::from_string_with_current_time(&format!("[tw-econ]: Can't connect to '{}'", addr));

                    tx.send(msg).expect(&format!("Can't write streambuffer for address: {:}", addr));

                    return false;
                }
            };

            loop {
                let mut buffer: [u8; 1024] = [0; 1024];
                unsafe {
                    match stream.read(&mut buffer) {
                        Ok(_) => match std::str::from_utf8_unchecked(&buffer) {
                            words => {
                                for word in words.to_string().split('\n') {
                                    let word = word.replace("\u{0}", "");
                                    let msg = match EconMessage::from_string(&word) {
                                        Some(m) => {
                                            Some(m)
                                        },
                                        _ => {
                                            if !word.is_empty() {
                                                Some(EconMessage::from_string_with_current_time(&word))
                                            }
                                            else {
                                                None
                                            }
                                        }
                                    };

                                    if let Some(m) = msg {
                                        tx.send(m).expect(&format!("Can't write streambuffer for address: {:}", addr));
                                    }
                                }
                            }
                        },
                        Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {},
                        Err(_) => {}
                    };
                }

                if let Ok(received) = srx.try_recv() {
                    if received == disconnect_msg {
                        let _ = sup_tx.send(EconMessage::from_string_with_current_time(&format!("[tw-econ]: Disconnected from: {:}", addr)));
                        return true;
                    }

                    let mut received = received.into_bytes().into_boxed_slice();
                    stream.write(&mut received).expect(&format!("Can't read streambuffer for address: {:}", addr));
                }
            }
        });

        (stx, rx)
    }

    pub async fn connect_async(address: std::net::SocketAddr, 
        password: String, 
        disconnect_msg: String) 
        -> 
        (tokio::sync::mpsc::Sender<String>, 
        tokio::sync::mpsc::Receiver<EconMessage>) 
    {
        use tokio::net::TcpStream;
        use tokio::io::AsyncWriteExt;
        use tokio::sync::mpsc::{self};

        let (tx, rx) = mpsc::channel::<EconMessage>(100);
        let (stx, mut srx) = mpsc::channel::<String>(100);
        
        tokio::spawn(async move {
            let addr = address.clone();
            let mut password = password; password.push('\n');
            let mut stream = match TcpStream::connect(addr).await {
                Ok(mut s) => {
                    let mut found = false;
                    while !found {
                        let mut buffer: [u8; 1024] = [0; 1024];
                        s.try_read(&mut buffer).expect(&format!("Can't read streambuffer for address: {:}", addr));
                        found = std::str::from_utf8(&buffer).unwrap().contains("Enter password:");
                    }
                    
                    s.try_write(password.as_bytes()).expect(&format!("Can't send password for address: {:}", addr));

                    let msg = EconMessage::from_string_with_current_time(&format!("[tw-econ]: Connected to '{}'", addr));

                    tx.send(msg).await.expect(&format!("Can't write streambuffer for address: {:}", addr));

                    s
                },
                _ => {
                    let msg = EconMessage::from_string_with_current_time(&format!("[tw-econ]: Can't connect to '{}'", addr));
                    tx.send(msg).await.expect(&format!("Can't write streambuffer for address: {:}", addr));
                    return false;
                }
            };

            loop {
                let mut buffer: [u8; 1024] = [0; 1024];
                unsafe {
                    match stream.try_read(&mut buffer) {
                        Ok(_) => match std::str::from_utf8_unchecked(&buffer) {
                            words => {
                                for word in words.to_string().split('\n') {
                                    let word = word.replace("\u{0}", "");
                                    let msg = match EconMessage::from_string(&word) {
                                        Some(m) => {
                                            Some(m)
                                        },
                                        _ => {
                                            if !word.is_empty() {
                                                Some(EconMessage::from_string_with_current_time(&word))
                                            }
                                            else {
                                                None
                                            }
                                        }
                                    };

                                    if let Some(m) = msg {
                                        tx.send(m).await.expect(&format!("Can't write streambuffer for address: {:}", addr));
                                    }
                                }
                            }
                        },
                        Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {},
                        Err(_) => {}
                    };
                }

                if let Some(received) = srx.recv().await {
                    if received == disconnect_msg {
                        let _ = tx.send(EconMessage::from_string_with_current_time(&format!("[tw-econ]: Disconnected from: {:}", addr)));
                    }

                    let mut received = received.into_bytes().into_boxed_slice();
                    stream.try_write(&mut received).expect(&format!("Can't read streambuffer for address: {:}", addr));
                }
            }
        });

        (stx, rx)
    }
}