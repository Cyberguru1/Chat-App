use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;
use std::thread;


const LOCAL: &str = "127.0.0.1:5000";
const MSG_SIZE: usize = 64;

fn main() {
    println!("Server is initializing...");
    println!("Server started...");
    println!("Server serving at {LOCAL}...\n");
    let server = TcpListener::bind(LOCAL).expect("listener failed to bind");
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("\nClient {} connected\n", addr);
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone Client"));

            thread::spawn(move || loop {
                let mut buffer: Vec<u8> = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buffer) {
                    Ok(_) => {
                        let msg = buffer.into_iter().take_while( |&x | x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("invalid utf8 message ");
                        let addr = format!("{addr}");
                        let parts: Vec<&str> = addr.split(":").collect();
                        let port = parts[1];

                        println!("<ID_{}>: {:?}", port, msg);

                        tx.send(format!("<ID_{}>: {}", port, msg)).expect("Failed to send message to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(| mut client | {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok()
            })
              .collect::<Vec<_>>();
        }

        sleep();
    }

}

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}