/* 
    Move this section into separate crate
    tw-econ is library, not the toolbox
 */

use std::io;
use std::sync::{
    mpsc::{self, Receiver, TryRecvError},
    Arc, Mutex
};
use std::net::SocketAddr;
use std::thread;

use tw_econ::connection::*;

fn main() {
    let stdin_channel = spawn_stdin_channel();

    let connection: Arc<Mutex<Connection<1024, 16>>> = Arc::new(Mutex::new(Connection::new()));

    println!("Enter server address (don't forget about ec_port): ");
    let address = stdin_channel.recv().unwrap();

    connection.lock().unwrap().launch(address.parse::<SocketAddr>().unwrap()).unwrap();

    let connection_channel = spawn_connection_channel(connection.clone());


    loop {
        match stdin_channel.try_recv() {
            Ok(key) => connection.lock().unwrap().send(key).unwrap(),
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("stdin receiver hang up"),
        }

        match connection_channel.try_recv() {
            Ok(key) => println!("{}", key),
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("conn receiver hang up"),
        }
    }

}

fn spawn_connection_channel<const T: usize, const TT: usize>(connection: Arc<Mutex<Connection<T, TT>>>) -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let connection = connection.clone();
        loop {
            if let Ok(received) = connection.lock().unwrap().recv() {
                tx.send(received).unwrap();
            }
        }
    });
    rx
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        let buffer = buffer.trim_end().to_string();
        tx.send(buffer).unwrap();
    });
    rx
}
