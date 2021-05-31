mod lib;

use structopt::StructOpt;
use std::sync::mpsc::{self, Receiver};


#[derive(Debug, StructOpt)]
struct Arguments {
    #[structopt(long, default_value = "127.0.0.1:8303")]
    address: String,
    #[structopt(long)]
    password: String,
    #[structopt(long)]
    standard: bool
}

fn main() {
    let args = Arguments::from_args();
    let stdin = spawn_stdin_channel();
    let (cmd, msg) = lib::EconConnection::connect(args.address.parse::<std::net::SocketAddr>().unwrap(), args.password, String::from(":q!"));
    
    loop {
        if let Ok(msg) = msg.try_recv() {
            if args.standard {
                println!("{}", msg.to_string());
            }
            else {
                println!("{} : {} ::: {}", msg.get_timestamp(), msg.get_category(), msg.get_content());
            }
        }

        if let Ok(received) = stdin.try_recv() {
            let _ = cmd.send(received);
        }
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();

    std::thread::spawn(move || loop {
        let mut buffer = String::new();

        std::io::stdin().read_line(&mut buffer).unwrap();

        tx.send(buffer).unwrap();
    });

    rx
}