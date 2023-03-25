use std::net::{TcpStream, Shutdown, SocketAddr};
use std::io::{Read, Write};

use crate::error::Error;


pub struct Connection<const BUFFER_SIZE: usize, const READ_TIMEOUT: usize> {
    socket: Option<TcpStream>,
}

impl<const BUFFER_SIZE: usize, const READ_TIMEOUT: usize> Connection<BUFFER_SIZE, READ_TIMEOUT> {
    pub fn new() -> Self {
        Self {
            socket: None
        }
    }

    fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        let mut stream = self.socket.as_ref().unwrap();
        let size = if let Ok(size) = stream.read(buffer) {
            size
        } else {
            1025usize
        };

        match size {
            0 => return Err(Error::NoResponse),
            1025 => return Err(Error::UnableToRead),
            _ => return Ok(size)
        }
    }

    fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        let mut stream = self.socket.as_ref().unwrap();
        let size = if let Ok(size) = stream.write(buffer) {
            size
        } else {
            1025usize
        };

        match size {
            0 => return Err(Error::NoResponse),
            1025 => return Err(Error::UnableToWrite),
            _ => return Ok(size)
        }
    }

    pub fn launch<A: Into<SocketAddr>>(&mut self, address: A) -> Result<(), Error> {
        let address = address.into();

        self.socket = match TcpStream::connect(address) {
            Ok(stream) => Some(stream),
            Err(_) => return Err(Error::NoResponse)
        };

        self.socket.as_ref().unwrap().set_read_timeout(Some(std::time::Duration::new(0, READ_TIMEOUT as _))).unwrap();

        Ok(())
    }

    pub fn launch_with_password<A: Into<SocketAddr>, S: Into<String>>(&mut self, address: A, password: S) -> Result<(), Error> {
        let address = address.into();
        let password = password.into();

        self.socket = match TcpStream::connect(address) {
            Ok(stream) => Some(stream),
            Err(_) => return Err(Error::NoResponse)
        };

        let mut buffer = [0; BUFFER_SIZE];

        if let Err(error) = self.read(&mut buffer) {
            return Err(error);
        }

        if let Err(error) = self.send(password) {
            return Err(error)
        }

        match self.read(&mut buffer) {
            Ok(size) => {
                if std::str::from_utf8(&buffer[..size]).unwrap().contains("Wrong") {
                    return Err(Error::WrongPassword);
                }
            },
            Err(error) => return Err(error)
        }

        self.socket.as_ref().unwrap().set_read_timeout(Some(std::time::Duration::new(0, READ_TIMEOUT as _))).unwrap();

        Ok(())
    }

    pub fn send<S: Into<String>>(&self, string: S) -> Result<(), Error> {
        let string = string.into() + "\n";
        let _ = match self.write(string.as_bytes()) {
            Ok(size) => size,
            Err(error) => return Err(error)
        };

        Ok(())
    }

    pub fn recv(&self) -> Result<String, Error> {
        let mut buffer = [0u8; BUFFER_SIZE];
        let size = match self.read(&mut buffer) {
            Ok(size) => size,
            Err(error) => return Err(error)
        };

        let result = String::from_utf8(buffer[..size].to_vec()).unwrap();
        let result = result.replace("\0", "").trim_end().to_string();

        return Ok(result);
    }

    pub fn shutdown(&mut self) {
        self.socket.as_ref().unwrap().shutdown(Shutdown::Both).unwrap();

        self.socket = None;
    }

    pub fn socket_addr(&self) -> Option<SocketAddr> {
        if let Some(socket) = &self.socket {
            if let Ok(address) = socket.peer_addr() {
                return Some(address);
            }
        }

        None
    }


    pub fn alive(&self) -> bool {
        self.socket.is_some()
    }
}
