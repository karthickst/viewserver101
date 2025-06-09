use std::thread;
use std::time::Duration;

use tungstenite::{connect, Message};

fn main() {
    let mut handles = vec![];

    for _ in 0..100_000 {
    let (mut stream, _) = connect("ws://127.0.0.1:8080").unwrap();

    stream.send(Message::Text("{\"type\":\"create_view\",\"name\":\"my_view\",\"filter\":\"my_filter\"}".to_string())).unwrap();

    handles.push(thread::spawn(move || {
        loop {
            match stream.read_message() {
                Ok(message) => {
                    println!("Received message: {:?}", message);
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    break;
                }
            }
        }
    }));
}

for handle in handles {
    handle.join().unwrap();
}
}
