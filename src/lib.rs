use std::sync::mpsc::{self, Sender, Receiver};
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
            _ => { None }
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

    // requires good stable runtime-formatter
    // wish i will find it
    // or do it myself...
    pub fn to_string_fmt(&self, format: String) -> String {
        unimplemented!();
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
    thread: thread::JoinHandle<bool>,
    address: std::net::SocketAddr,
    tx: Sender<String>,
    rx: Receiver<EconMessage>,
    messages: Vec<EconMessage>
}

impl EconConnection {
    pub fn connect(address: std::net::SocketAddr, password: String) -> Self {
        let (tx, rx): (Sender<EconMessage>, Receiver<EconMessage>) = mpsc::channel();
        let (stx, srx): (Sender<String>, Receiver<String>) = mpsc::channel();
        
        let t = thread::spawn(move || {
            let tx = tx.clone();
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

                    let now = Utc::now();
                    let msg = EconMessage {
                        timestamp: NaiveTime::from_hms(now.hour(), now.minute(), now.second()),
                        category: String::from("tw-econ"),
                        content: format!("Connected to '{}'", addr),
                    };

                    tx.send(msg).expect(&format!("Can't write streambuffer for address: {:}", addr));

                    s.set_nonblocking(true).expect(&format!("Can't set non-block read for address: {:}", addr));

                    s
                },
                _ => { panic!("pizdec!"); }
            };

            loop {
                let mut buffer: [u8; 1024] = [0; 1024];

                match stream.read(&mut buffer) {
                    Ok(_) => {},
                    Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {},
                    Err(_) => {}
                };

                match std::str::from_utf8(&buffer).unwrap() {
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
                }

                if let Ok(received) = srx.try_recv() {
                    let mut received = received.into_bytes().into_boxed_slice();
                    stream.write(&mut received).expect(&format!("Can't read streambuffer for address: {:}", addr));
                }
            }
        });

        EconConnection {
            thread: t,
            address: address,
            tx: stx,
            rx,
            messages: Vec::new()
        }
    }

    pub fn disconnect(&mut self) {
        // drop connection somehow
        unimplemented!();
        // self.messages.push(EconMessage::from_string_with_current_time(format!("[tw-econ]: Disconnected from '{}'", self.address)).unwrap());
    }

    pub fn update(&mut self) {
        if let Ok(received) = self.rx.try_recv() {
            self.messages.push(received);
        }
    }

    pub fn send(&self, command: String) {
        self.tx.send(command).expect(&format!("Can't send streambuffer for address: {:}", self.address));
    }

    pub fn get_messages(&mut self) -> &mut Vec<EconMessage> {
        &mut self.messages
    }
}