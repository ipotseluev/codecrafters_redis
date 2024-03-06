use std::{error::Error, net::SocketAddr};
mod error;
mod protocol;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    signal,
};

const LISTEN_ADDR: &str = "127.0.0.1:6379";

fn handle_request(request: protocol::Request) -> Result<protocol::Response, Box<dyn Error>> {
    match request {
        protocol::Request::Ping => Ok(protocol::Response::Ping),
        protocol::Request::Echo(arg) => Ok(protocol::Response::Echo(arg)),
    }
}

async fn handle_connection(
    mut connection: TcpStream,
    sender: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    println!("Accepted connection from {}", sender);
    let mut bbuf = [0; 1024];
    let mut sbuf = String::new();
    loop {
        let n = connection.read(&mut bbuf).await?;
        if n == 0 {
            break;
        }
        println!("received {n} bytes");

        sbuf.push_str(String::from_utf8_lossy(&bbuf[..n]).as_ref());
        match protocol::Request::deserialize(&mut sbuf) {
            Ok(request) => {
                dbg!(&request);
                let response: protocol::Response = handle_request(request)?;
                connection.write_all(response.serialize().as_bytes()).await?;
            }
            Err(e) => {
                println!("Failed to deserialize request. Error: {:?}", e);
            }
        }
    }

    println!("Finished serving {}", sender);
    Ok(())
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
