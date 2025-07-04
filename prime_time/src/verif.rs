use serde::{Deserialize, Serialize};
use std::time;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    method: String,
    number: i64,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid method: {0}")]
    InvalidMethod(String),
}

#[tracing::instrument(skip(buf))]
pub fn process_request(buf: &[u8]) -> anyhow::Result<Vec<u8>> {
    let time = time::Instant::now();
    let parsed: Request = serde_json::from_slice(buf)?;
    let resp = parsed.process()?;
    let buf = serde_json::to_vec(&resp)?;
    info!("Processed request in {} ms", time.elapsed().as_millis());
    Ok(buf)
}

impl Request {
    pub fn process(self) -> anyhow::Result<Response> {
        if self.method == "isPrime" {
            Ok(Response::new(is_prime_opt(self.number)))
        } else {
            Err(Error::InvalidMethod(self.method))?
        }
    }
}

fn is_prime(x: i64) -> bool {
    if x <= 1 {
        return false;
    } else {
        for i in 2..x {
            if x % i == 0 {
                return false;
            }
        }
        return true;
    }
}

fn is_prime_opt(x: i64) -> bool {
    if x <= 1 {
        return false;
    } else {
        let bound = x.isqrt() + 1;
        for i in 2..bound {
            if x % i == 0 {
                return false;
            }
        }
        return true;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    method: &'static str,
    prime: bool,
}

impl Response {
    fn new(prime: bool) -> Self {
        Self {
            method: "isPrime",
            prime,
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn is_prime_equivalence(x in -10000 as i64..10000 as i64) {
            prop_assert_eq!(super::is_prime(x), super::is_prime_opt(x));
        }
    }
}
