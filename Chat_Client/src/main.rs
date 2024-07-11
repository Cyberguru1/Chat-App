use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;


const LOCAL: &str = "127.0.0.1:5000";
const MSG_SIZE: usize = 64;


fn main() {
    /// Client connection initializing...
    println!("Client initializing...\n");

    let mut client = TcpStream::connect(LOCAL).expect("Stream Failed to connect!");
    client.set_nonblocking(true).expect("Failed to initialze non-blocking!");

    let local_addr = client.local_addr().expect("Failed to extract local address");
    let port = local_addr.port();

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).unwrap();
                let mut parts: Vec<&str> = msg.split(":").collect();
                let msg = parts[1].trim();
                parts = parts[0].split("_").collect();
                parts = parts[1].split(">").collect();
                let rport = parts[0];

                if rport != format!("{port}") {
                    println!("<ID_{rport}>: {:?}", msg);
                }
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("<ID_{port}>: Connection with server was severed");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed");
                println!("<ID_{port}>: {:?}", msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Client Started\n\nClient id: <{port}>\n\nWrite a message:");
    
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() { break }
    }

    println!("Bye!!!");
}

