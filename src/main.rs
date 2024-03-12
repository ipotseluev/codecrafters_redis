//! Implementation of Redis server made for educational purposes.
//! [Link to course](https://app.codecrafters.io/courses/redis)
use std::{error::Error, net::SocketAddr};
mod error;
pub(crate) mod protocol;
mod request_processor;
mod storage;
use request_processor::RequestProcessor;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    signal,
};

const LISTEN_ADDR: &str = "127.0.0.1:6379";

async fn handle_connection(
    mut connection: TcpStream,
    sender: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    println!("Accepted connection from {}", sender);
    let mut byte_buf = [0; 1024];
    let mut str_buf = String::new();
    let processor = RequestProcessor::new();
    loop {
        let n = connection.read(&mut byte_buf).await?;
        if n == 0 {
            break;
        }
        println!("received {n} bytes");

        str_buf.push_str(String::from_utf8_lossy(&byte_buf[..n]).as_ref());
        match protocol::Request::deserialize(&mut str_buf) {
            Ok(request) => {
                dbg!(&request);
                let response = processor.process_request(request).await?;
                connection
                    .write_all(response.serialize().as_bytes())
                    .await?;
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
