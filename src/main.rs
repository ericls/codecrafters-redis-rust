mod resp;
use resp::RESPType;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread, time::{SystemTime, UNIX_EPOCH},
};

macro_rules! extract {
    ($enum:path, $expr:expr) => {{
        if let $enum(item) = $expr {
            item
        } else {
            panic!()
        }
    }};
}

fn now() -> u128 {
    let n = SystemTime::now();
    return n.duration_since(UNIX_EPOCH).expect("what").as_millis();
}

// const SCARY_GLOBAL_HASHMAP: HashMap<String, String> = HashMap::new();

fn handle_redis_connection(mut stream: TcpStream, store: &Arc<Mutex<HashMap<String, (String, u128, usize)>>>) {
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
                            } else if command.to_lowercase() == "echo" {
                                if let RESPType::BulkString(arg0) = args[1] {
                                    stream.write(&RESPType::BulkString(arg0).pack()).unwrap();
                                };
                            } else if command.to_lowercase() == "set" {
                                let key = extract!(RESPType::BulkString, args[1]);
                                let value = extract!(RESPType::BulkString, args[2]);
                                let mut hash_map = store.lock().unwrap();
                                let mut px: usize = 0;
                                if args.len() == 5 {
                                    let px_word = extract!(RESPType::BulkString, args[3]);
                                    assert_eq!(px_word.to_lowercase(), "px");
                                    let px_value = extract!(RESPType::BulkString, args[4]);
                                    px = px_value.parse().unwrap();
                                }
                                hash_map.insert(key.into(), (value.into(), now(), px));
                                stream.write(&RESPType::SimpleString("OK").pack()).unwrap();
                            } else if command.to_lowercase() == "get" {
                                let key = extract!(RESPType::BulkString, args[1]);
                                let hash_map = store.lock().unwrap();
                                let value = hash_map.get(key);
                                match value {
                                    Some(v) => {
                                        let (str_v, added, px) = v;
                                        if px == &0 || now() < (added + &u128::try_from(px.to_owned()).unwrap()) {
                                            stream.write(&RESPType::BulkString(&str_v).pack()).unwrap();
                                        }
                                        stream.write(b"$-1\r\n").unwrap();
                                    }
                                    _ => {
                                        stream.write(b"$-1\r\n").unwrap();
                                    }
                                }
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
    let store: Arc<Mutex<HashMap<String, (String, u128, usize)>>> = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let thread_store = Arc::clone(&store);
                thread::spawn(move || {
                    handle_redis_connection(stream, &thread_store);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
