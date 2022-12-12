mod resp;
use resp::RESPType;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn handle_redis_connection(mut stream: TcpStream) {
    let remote_addr = stream.peer_addr().unwrap();
    println!("accepted new connection from {}", remote_addr);
    let mut buf: [u8; 1024] = [0; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(_size) => {
                let (command_buf, _com_size) = RESPType::unpack(&buf);
                match command_buf {
                    RESPType::Array(args) => {
                        if let RESPType::BulkString(command) = args[0] {
                            println!("got command: {}", command);
                            if command.to_lowercase() == "ping" {
                                stream
                                    .write(&RESPType::SimpleString("PONG").pack())
                                    .unwrap();
                            }
                            if command.to_lowercase() == "echo" {
                                if let RESPType::BulkString(arg0) = args[1] {
                                    stream.write(&RESPType::BulkString(arg0).pack()).unwrap();
                                };
                            }
                        };
                    }
                    _ => {
                        println!("Command format not right");
                        break;
                    }
                }
            }
            Err(_) => {
                break;
            }
        }
    }
    println!("{} connection closed", remote_addr);
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

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
