#[derive(Debug, Clone, Copy)]
pub enum Error {
    Timeout,
    WrongPassword,
    DisconnectedByServer,
    NoResponse,
    NoConnection,
    UnableToRead,
    UnableToWrite,
    BufferExhausted
}
