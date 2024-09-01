use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::Duration;

pub struct EconRaw {
    socket: TcpStream,
    buffer: Vec<u8>,
    lines: Vec<String>,
    unfinished_line: String,
    authed: bool,
}

impl EconRaw {
    pub fn connect(
        address: impl Into<SocketAddr>,
        buffer_size: usize,
        timeout_secs: u64,
    ) -> std::io::Result<Self> {
        let buffer = vec![0u8; buffer_size];
        let address = address.into();

        let connection = TcpStream::connect_timeout(&address, Duration::from_secs(timeout_secs))?;

        Ok(Self {
            socket: connection,
            buffer,
            lines: Vec::new(),
            unfinished_line: String::new(),
            authed: false,
        })
    }

    pub fn disconnect(&mut self) -> std::io::Result<()> {
        self.socket.shutdown(Shutdown::Both)
    }

    pub fn auth(&mut self, password: &str) -> std::io::Result<bool> {
        self.read()?;
        self.lines.clear();

        self.send(password)?;

        self.read()?;

        while let Some(line) = self.pop_line() {
            if line.starts_with("Authentication successful") {
                self.authed = true;
            }
        }

        Ok(self.authed)
    }

    pub fn read(&mut self) -> std::io::Result<usize> {
        let mut lines_amount = 0;
        let written = self.socket.read(&mut self.buffer)?;

        if written != 0 {
            let mut lines: Vec<String> = String::from_utf8_lossy(&self.buffer[..written])
                .replace('\0', "")
                .split("\n")
                .map(String::from)
                .collect();

            if lines.last().unwrap() == "" {
                let _ = lines.pop();

                if !self.unfinished_line.is_empty() {
                    let take = self.unfinished_line.to_owned();
                    lines[0] = take + &lines[0];

                    self.unfinished_line.clear();
                }
            } else {
                self.unfinished_line = lines.pop().unwrap();
            }

            lines_amount = lines.len();

            self.lines.extend(lines);
        }

        Ok(lines_amount)
    }

    pub fn send(&mut self, line: &str) -> std::io::Result<()> {
        self.socket.write_all(line.as_bytes())?;
        self.socket.write_all("\n".as_bytes())?;

        self.socket.flush()?;

        Ok(())
    }

    pub fn pop_line(&mut self) -> Option<String> {
        self.lines.pop()
    }

    pub fn is_authed(&self) -> bool {
        self.authed
    }
}
