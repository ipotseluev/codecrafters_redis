use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

const PONG: &[u8; 7] = b"+PONG\r\n";

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let reader = BufReader::new(stream.try_clone()?).take(10000);
    for line in reader.lines() {
        let line = line?;
        println!("Got message: {}", line);
        if line.trim() == "ping" {
            stream.write_all(PONG)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_connection(stream)?;
            }
            Err(e) => {
                println!("error: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}
