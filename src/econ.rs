use std::{io::ErrorKind, net::SocketAddr};

use crate::raw::EconRaw;

#[derive(Default)]
pub struct Econ {
    raw: Option<EconRaw>,
    is_alive: bool,
}

impl Econ {
    pub fn new() -> Self {
        Self::default()
    }

    /// Connects to given address
    pub fn connect(&mut self, address: impl Into<SocketAddr>) -> std::io::Result<()> {
        self.raw = Some(EconRaw::connect(address, 2048, 5)?);

        self.is_alive = true;

        Ok(())
    }

    pub fn reconnect(&mut self) -> std::io::Result<()> {
        assert!(
            self.raw.is_some() && !self.is_alive,
            "can't reconnect without being disconnected"
        );

        let raw = unsafe { self.raw.as_mut().unwrap_unchecked() };

        raw.reconnect()
    }

    pub fn disconnect(&mut self) -> std::io::Result<()> {
        let raw = self.get_raw_mut();

        raw.disconnect()
    }

    /// Tries to authenticate, returns `false` if password is incorrect`
    pub fn try_auth(&mut self, password: impl Into<String>) -> std::io::Result<bool> {
        let raw = self.get_raw_mut();

        raw.auth(password.into().as_str())
    }

    /// Change auth message
    pub fn set_auth_message<T: ToString>(&mut self, auth_message: T) {
        let raw = self.get_raw_mut();

        raw.set_auth_message(auth_message.to_string());
    }

    /// Blocking *write* operation, sends line to socket
    pub fn send_line(&mut self, line: impl Into<String>) -> std::io::Result<()> {
        let raw = self.get_raw_mut();

        assert!(
            raw.is_authed(),
            "can't send commands without being authed"
        );

        raw.send(line.into().as_str())
    }

    /// Blocking *read* operation, reads to buffer and appends to inner line buffer
    pub fn fetch(&mut self) -> std::io::Result<()> {
        let raw = self.get_raw_mut();

        assert!(
            raw.is_authed(),
            "can't fetch lines without being authed"
        );

        match raw.read() {
            Err(error) => {
                if error.kind() == ErrorKind::ConnectionAborted {
                    self.is_alive = false;
                    return Err(error);
                }

                Err(error)
            }
            _ => Ok(()),
        }
    }

    pub fn pop_line(&mut self) -> std::io::Result<Option<String>> {
        let raw = self.get_raw_mut();

        assert!(
            raw.is_authed(),
            "can't fetch lines without being authed"
        );

        Ok(raw.pop_line())
    }

    fn get_raw_mut(&mut self) -> &mut EconRaw {
        assert!(
            self.raw.is_some() && self.is_alive,
            "can't do anything without being connected"
        );

        unsafe { self.raw.as_mut().unwrap_unchecked() }
    }
}
