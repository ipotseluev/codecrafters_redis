use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    println!("handling connection");
    let buf = &mut String::new();
    {
        let mut buf_reader = BufReader::new(&stream).take(1000);
        while let Ok(req_size) = buf_reader.read_line(buf) {
            println!("Got {} bytes: '{}'", req_size, buf.trim());
            if buf.trim() == "ping" {
                println!("Got ping");
                break;
            }
            buf.clear();
        }
    }
    let pong = b"+PONG\r\n";
    println!("Answering with: pong");
    stream.write_all(pong)?;

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
