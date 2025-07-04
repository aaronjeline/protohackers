use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{stdin, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::select;
use tokio::signal;
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let stdin = stdin();
    let mut reader = BufReader::new(stdin);

    let mut socket = TcpStream::connect("localhost:1337").await?;

    let exit = loop {
        let mut line = String::new();
        select! {
            _  = signal::ctrl_c() => break 1,
            res  = reader.read_line(&mut line) => {
                match res {
                    Ok(amnt) => {
                        if amnt == 0 {
                            break 0;
                        }
                        process_request(line.trim(), &mut socket).await?;
                    }
                    Err(err) => {
                        error!("Error reading std: {err}");
                        break 1;
                    }
                }
            }
        }
    };

    std::process::exit(exit);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Request {
    method: &'static str,
    number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {
    method: String,
    prime: bool,
}

async fn process_request(line: &str, socket: &mut TcpStream) -> Result<()> {
    let number: i64 = match line.parse() {
        Ok(number) => number,
        Err(_) => {
            error!("Couldn't parse number");
            return Ok(());
        }
    };
    let request = Request {
        method: "isPrime",
        number,
    };
    debug!("Sending {:?}", request);
    let bytes = serde_json::to_vec(&request)?;
    socket.write_all(&bytes).await?;
    debug!("Wrote request to socket");
    let mut response_bytes = [0; 1024];
    let got = socket.read(&mut response_bytes).await?;
    debug!(
        "Read {got} bytes from socket: {}",
        String::from_utf8_lossy(&response_bytes)
    );
    let response: Response = serde_json::from_slice(&response_bytes[..got])?;
    if response.method == "isPrime" {
        if response.prime {
            println!("{number} is prime");
        } else {
            println!("{number} is not prime");
        }
    } else {
        error!("Invalid response method! ({})", response.method);
    }
    Ok(())
}
