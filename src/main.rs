mod econ_connection;

use structopt::StructOpt;
use std::io::{self, Read};
use std::sync::mpsc::{self, Receiver};


#[derive(Debug, StructOpt)]
struct Arguments {
    #[structopt(long, default_value = "127.0.0.1:8303")]
    address: String,
    #[structopt(long)]
    password: String
}

fn main() {
    let args = Arguments::from_args();
    let stdin = spawn_stdin_channel();
    let mut conn = econ_connection::EconConnection::connect(args.address.parse::<std::net::SocketAddr>().unwrap(), args.password);
    
    loop {
        conn.update();

        if let Some(msg) = conn.get_messages().pop() {
            println!("{} : {} ::: {}", msg.get_timestamp(), msg.get_category(), msg.get_content());
        }

        if let Ok(received) = stdin.try_recv() {
            conn.send(received);
        }
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();

    std::thread::spawn(move || loop {
        let mut buffer = String::new();

        io::stdin().read_line(&mut buffer).unwrap();

        tx.send(buffer).unwrap();
    });

    rx
}