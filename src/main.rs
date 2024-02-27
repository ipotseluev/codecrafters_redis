use std::{error::Error, net::SocketAddr};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    signal,
};

const PONG: &[u8; 7] = b"+PONG\r\n";
const LISTEN_ADDR: &str = "127.0.0.1:6379";

async fn handle_connection(
    mut connection: TcpStream,
    sender: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    println!("Accepted connection from {}", sender);
    let mut buf = String::new();
    let mut buf_reader = BufReader::new(&mut connection);
    loop {
        let n = match buf_reader.read_line(&mut buf).await {
            Ok(0) => {
                println!("Reached EOF on the connection {}\n", sender);
                return Ok(());
            }
            Err(e) => {
                println!("Error reading from connection {}: {}\n", sender, e);
                return Err(Box::new(e));
            }
            Ok(n) => n,
        };

        println!("got {} bytes: '{}'", n, buf.trim());
        if buf.trim() == "ping" {
            buf_reader.write_all(PONG).await?;
        }
        buf.clear();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    println!("Starting accepting connections on {}", LISTEN_ADDR);
    loop {
        tokio::select! {
            result = listener.accept() => {
                // let a = result;
                let (connection, addr) = result?;
                tokio::spawn(async move {
                    let _ = handle_connection(connection, addr).await;
                });
            }
            _ = signal::ctrl_c() => {
                println!("Got keyboard signal. Shutting down the server...");
                break;
            }
        }
    }
    Ok(())
}
