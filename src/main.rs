use tokio::sync::mpsc;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::env;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, BufReader};

// THE STATE
// we need a phone book to keep track of everyone we are connected to
// TX = transmitter, it's the sending half of a channel

type Tx = mpsc::UnboundedSender<String>;

// PeerMap = shared memory
// Arc = allow multiple threads to own it
// Mutex = allow only one thread to edit it at a time
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // SETUP ARGS
    // we read the port from the command line eg (cargo run -- 8080)
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <port> [peer_addr]");
        return Ok(());
    }

    let my_port = &args[1];
    let my_addr = format!("127.0.0.1:{}", my_port);

    // initialize the empty phonebook
    let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    // THE SERVER()
    // create a clone for the phonebook for the server thread
    let peers_server = peers.clone();
    let addr = my_addr.clone();

    // spawn a background task to listen for connection
    tokio::spawn(async move {
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
        println!("Listening on {}", addr);
        // loop forever, accepting new friends
        while let Ok((stream, addr)) = listener.accept().await {
            println!("Incoming connection from {}", addr);
            // handle this specific friend in their new task
            tokio::spawn(handle_connection(peers_server.clone(), stream, addr));
        }
    });

    // THE CLIENT CONNECT
    // if the user provided a 2nd argument, connect to that address
    if args.len() > 2 {
        let peer_addr = &args[2];
        if let Ok(stream) = TcpStream::connect(peer_addr).await {
            println!("connected to peer {} ", peer_addr);
            let peer_socket_addr = stream.peer_addr().unwrap();
            tokio::spawn(handle_connection(peers.clone(), stream, peer_socket_addr));
        }
    }

    // THE INPUT LOOP (STDIN)
    // Listen to the keyboard
    let mut stdin = BufReader::new(tokio::io::stdin()).lines();
    println!("start typing to chat");

    while let Ok(Some(line)) = stdin.next_line().await {
        let peers = peers.lock().unwrap();
        // broadcast the typed line to everyone in the phonebook]
        for (addr, tx) in peers.iter() {
            let _ = tx.send(format!("From {}: {}", my_port, line));
        }
    }


    Ok(())
}

// THE CONNECTOR LOGIC
// this function handles every connection, incoming or outgoing
async fn handle_connection(peers: PeerMap, stream: TcpStream, addr: SocketAddr) {
    
}