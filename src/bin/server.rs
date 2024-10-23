use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::{env, thread};

const MSG_SIZE: usize = 32;

fn sleep() {
   // thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let address = match decode_input() {
        Some(value) => value,
        None => return,
    };

    let server = start_server(address);

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        if let Ok((socket, addr)) = server.accept() {
            initiate_client(addr, &tx, &mut clients, socket);
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();

                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }

        sleep();
    }
}

fn initiate_client(addr: std::net::SocketAddr, tx: &mpsc::Sender<String>, clients: &mut Vec<std::net::TcpStream>, socket: std::net::TcpStream) {
    println!("Client {} connected", addr);

    let tx = tx.clone();
    clients.push(socket.try_clone().expect("Failed to clone client"));

    thread::spawn(move || read_client_message(socket, addr, tx));
}

fn read_client_message(
    mut socket: std::net::TcpStream,
    addr: std::net::SocketAddr,
    tx: mpsc::Sender<String>,
) {
    loop {
        let mut buff = vec![0; MSG_SIZE];

        match socket.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                println!("{}: {:?}", addr, msg);
                tx.send(msg).expect("failed to send msg to rx");
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("closing connection with: {}", addr);
                break;
            }
        }

        sleep();
    }
}

fn start_server(address: String) -> TcpListener {
    let server = TcpListener::bind(address).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");
    server
}

fn decode_input() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: client <Server Port>");
        return None;
    }
    let address = format!("127.0.0.1:{}", args[1]);
    Some(address)
}
