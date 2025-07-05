use anyhow::Result;
use lib::Request;
use std::collections::BTreeMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    server().await
}

#[tracing::instrument]
async fn server() -> Result<()> {
    let token = CancellationToken::new();
    let listener = TcpListener::bind("0.0.0.0:1337").await?;
    info!("Listening on 0.0.0.0:1337");
    loop {
        debug!("Waiting for connection");
        select! {
            accept_result = listener.accept() => {
                let (socket, addr) = accept_result?;
                info!("{addr} connected");
                let cloned = token.clone();
                tokio::spawn(async move { Client::start(socket, cloned).await });
            }
            _ = signal::ctrl_c() => {
                break;
            }
        }
    }
    info!("Shutting down...");
    token.cancel();
    Ok(())
}

struct Client {
    socket: TcpStream,
    data: BTreeMap<i32, i32>,
}

impl Client {
    async fn start(socket: TcpStream, token: CancellationToken) {
        let mut me = Self {
            socket,
            data: BTreeMap::new(),
        };
        match me.run(token).await {
            Ok(()) => info!("Client exited"),
            Err(e) => error!("Client errored: {e}"),
        };
    }

    #[tracing::instrument(skip(self))]
    async fn run(&mut self, token: CancellationToken) -> Result<()> {
        loop {
            debug!("Reading from client scoket");
            select! {
                read_result = self.read_nine_bytes() => {
                    match read_result? {
                        Some(buf) => self.process_request(buf).await?,
                        None => {
                            info!("Client hung  up");
                            break;
                        }
                    }
                }
                _ = token.cancelled() => {
                    info!("Cancellation token expired");
                    break
                }
            };
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn process_request(&mut self, buf: [u8; 9]) -> Result<()> {
        debug!("Read buffer: {:?}", buf);
        let r = Request::deserialize(&buf)?;
        match r {
            Request::Query { mintime, maxtime } => self.execute_query(mintime, maxtime).await?,
            Request::Insert { timestamp, price } => self.execute_insert(timestamp, price),
        };
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn execute_insert(&mut self, timestamp: i32, price: i32) {
        self.data.insert(timestamp, price);
    }

    #[tracing::instrument(skip(self))]
    async fn execute_query(&mut self, mintime: i32, maxtime: i32) -> Result<()> {
        let mut count = 0;
        let mut sum = 0;
        for (_, price) in self.data.range(mintime..maxtime) {
            count += 1;
            sum += price;
        }
        let avg = sum / count;
        let buf = avg.to_be_bytes();
        debug!("About to write {:?} to client", buf);
        self.socket.write_all(&buf).await?;
        debug!("write complete");
        Ok(())
    }

    async fn read_nine_bytes(&mut self) -> Result<Option<[u8; 9]>> {
        let mut buf = [0; 9];
        match self.socket.read_exact(&mut buf).await {
            Ok(_) => Ok(Some(buf)),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    Ok(None)
                } else {
                    Err(e)?
                }
            }
        }
    }
}
