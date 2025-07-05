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

struct Client<T> {
    socket: T,
    data: BTreeMap<i32, i32>,
}

impl Client<TcpStream> {
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

    #[tracing::instrument(
        skip(self, token), 
        fields(
            // `%` serializes the peer IP addr with `Display`
            peer_addr = %self.socket.peer_addr().unwrap()
        ))]
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

    #[tracing::instrument(
        skip(self), 
        fields(
            // `%` serializes the peer IP addr with `Display`
            peer_addr = %self.socket.peer_addr().unwrap()
        ))]
    async fn process_request(&mut self, buf: [u8; 9]) -> Result<()> {
        debug!("Read buffer: {:?}", buf);
        let r = Request::deserialize(&buf)?;
        match r {
            Request::Query { mintime, maxtime } => {
                let avg = self.execute_query(mintime, maxtime);
                self.write_int(avg).await?
            }
            Request::Insert { timestamp, price } => self.execute_insert(timestamp, price),
        };
        Ok(())
    }

    #[tracing::instrument(
        skip(self), 
        fields(
            // `%` serializes the peer IP addr with `Display`
            peer_addr = %self.socket.peer_addr().unwrap()
        ))]
    async fn write_int(&mut self, i: i32) -> Result<()> {
        let buf = i.to_be_bytes();
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

impl<T> Client<T> {
    #[tracing::instrument(skip(self))]
    fn execute_insert(&mut self, timestamp: i32, price: i32) {
        self.data.insert(timestamp, price);
    }

    #[tracing::instrument(skip(self))]
    fn execute_query(&self, mintime: i32, maxtime: i32) -> i32 {
        if mintime > maxtime {
            return 0;
        }
        let mut count = 0;
        let mut sum = 0;
        for (_, price) in self.data.range(mintime..=maxtime) {
            count += 1;
            sum += price;
        }
        if count == 0 {
            return 0;
        }
        sum / count
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn test() {
        let mut c = Client {
            socket: (),
            data: BTreeMap::new(),
        };
        assert_eq!(0, c.execute_query(12288, 16384));
        c.execute_insert(12345, 101);
        c.execute_insert(12346, 102);
        c.execute_insert(12347, 100);
        c.execute_insert(40960, 7);
        let r = c.execute_query(12288, 16384);
        assert_eq!(r, 101);
        let r = c.execute_query(12345, 12347);
        assert_eq!(r, 101);
        assert_eq!(0, c.execute_query(500, 2));
        assert_eq!(0, c.execute_query(2, 50));
        assert_eq!(100, c.execute_query(12347, 12347));
    }

    fn request() -> impl Strategy<Value = Request> {
        let strat = (proptest::bool::ANY, -1000..1000, -1000..1000);
        strat.prop_map(|(b,i1,i2)| 
                if b {
                    Request::Insert { timestamp : i1, price : i2 }
                }  else {
                    Request::Query { mintime : i1, maxtime : i2 }
                }
        )
    }

    fn requests() -> impl Strategy<Value = Vec<Request>> {
        proptest::collection::vec(request(), 0..500)
    }

    fn execute_simulation(reqs : Vec<Request>) -> Result<(), TestCaseError> {
        let mut seen = HashSet::new();
        let mut data = vec![];
        let mut client = Client { socket : (), data : BTreeMap::new() };
        for req in reqs {
            match req {
                Request::Insert { timestamp, price } => {
                    if seen.contains(&timestamp) {
                        return Ok(());
                    }
                    seen.insert(timestamp);
                    insert_into_sorted(&mut data, (timestamp, price))?;
                    client.execute_insert(timestamp, price);
                }
                Request::Query { mintime, maxtime } => {
                    let r = client.execute_query(mintime, maxtime);
                    if mintime > maxtime {
                        prop_assert_eq!(r, 0);
                    } else {
                        let mut count = 0;
                        let mut sum = 0;
                        for (time,price) in data.iter() {
                            if *time >= mintime && *time <= maxtime {
                                count += 1;
                                sum += price;
                            }
                            if count == 0 {
                                prop_assert_eq!(r, 0);
                            } else {
                                prop_assert_eq!(r, sum / count);
                            }
                        }

                    }
                }
            }
        }
        Ok(())
    }

    fn insert_into_sorted(lst : &mut Vec<(i32, i32)>, x : (i32, i32)) -> Result<(), TestCaseError> {
        prop_assert!(lst.is_sorted_by(|(t1,_), (t2,_)| t1 < t1));
        for i in 0..lst.len() {
            if lst[i].0 > x.0 {
                lst.insert(i, x);
                break;
            }
        }
        prop_assert!(lst.is_sorted_by(|(t1,_), (t2,_)| t1 < t1));
        Ok(())
    }

    proptest! {
        #[test]
        fn simulate(reqs in requests()) {
            execute_simulation(reqs)?;
        }
    }


}
