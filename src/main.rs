// Uncomment this block to pass the first stage
// use std::net::TcpListener;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn handle_redis_connection(mut stream: TcpStream) {
    let remote_addr = stream.peer_addr().unwrap();
    println!(
        "accepted new connection from {}",
        remote_addr
    );
    let mut buf: [u8; 1024] = [0; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(_size) => match stream.write(&"+PONG\r\n".as_bytes()) {
                Ok(_) => continue,
                Err(_) => break,
            },
            Err(_) => {
                break;
            }
        }
    }
    println!(
        "{} connection closed",
        remote_addr
    );
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6380").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_redis_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
