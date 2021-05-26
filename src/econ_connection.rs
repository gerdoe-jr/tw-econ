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
    raw: String
}

impl EconMessage {
    pub fn new() -> Self {
        EconMessage {
            timestamp: NaiveTime::from_hms(0, 0, 0),
            category: String::from("empty"),
            content: String::from("empty"),
            raw: String::from("empty")
        }
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

    pub fn get_raw(&self) -> String {
        self.raw.clone()
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
                    
                    thread::sleep(std::time::Duration::new(1, 0));
                    s.write(password.as_bytes()).expect(&format!("Can't send password for address: {:}", addr));

                    let now = Utc::now();
                    let msg = EconMessage {
                        timestamp: NaiveTime::from_hms(now.hour(), now.minute(), now.second()),
                        category: String::from("tw-econ"),
                        content: format!("Connected to '{}'", addr),
                        raw: format!("[tw-econ]: Connected to '{}'", addr)
                    };
                    tx.send(msg).expect(&format!("Can't write streambuffer for address: {:}", addr));

                    s
                },
                _ => { panic!("pizdec!"); }
            };

            loop {
                if let Ok(received) = srx.try_recv() {
                    let mut received = received.into_bytes().into_boxed_slice();
                    stream.write(&mut received).expect(&format!("Can't read streambuffer for address: {:}", addr));
                }

                let mut buffer: [u8; 1024] = [0; 1024];

                stream.read(&mut buffer).expect(&format!("Can't read streambuffer for address: {:}", addr));

                match std::str::from_utf8(&buffer).unwrap() {
                    words => {
                        for word in words.to_string().split('\n') {
                            let mut msg: Option<EconMessage> = None;

                            match sscanf::scanf!(word.clone(), "[{}][{}]: {}", String, String, String) {
                                Some((utc, ctg, cnt)) => {
                                    msg = Some(EconMessage {
                                        timestamp: NaiveTime::parse_from_str(&utc, "%H:%M:%S").unwrap(),
                                        category: ctg,
                                        content: cnt,
                                        raw: String::from(word)
                                    }); 
                                },
                                _ => {
                                    if !word.is_empty() {
                                        let now = Utc::now();
                                        let now = NaiveTime::from_hms(now.hour(), now.minute(), now.second());
                                        msg = Some(EconMessage {
                                            timestamp: now,
                                            category: String::from("tw-econ"),
                                            content: String::from(word),
                                            raw: format!("[{}][tw-econ]: {}", now, word)
                                        });
                                    }
                                }
                            }

                            if let Some(m) = msg {
                                tx.send(m).expect(&format!("Can't write streambuffer for address: {:}", addr));
                            }
                        }
                    }
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
        let now = chrono::Utc::now();
        let now = NaiveTime::from_hms(now.hour(), now.minute(), now.second());
        let disconnected = EconMessage {
            timestamp: now,
            category: String::from("tw-econ"),
            content: format!("Disconnected from '{}'", self.address),
            raw: format!("[{}][tw-econ]: Disconnected from '{}'", now, self.address)
        };
        self.messages.push(disconnected);
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
