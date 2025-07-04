use serde::{Deserialize, Serialize};
use thiserror::Error;

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

pub fn process_request(buf: &[u8]) -> anyhow::Result<Vec<u8>> {
    let parsed: Request = serde_json::from_slice(buf)?;
    let resp = parsed.process()?;
    let buf = serde_json::to_vec(&resp)?;
    Ok(buf)
}

impl Request {
    pub fn process(self) -> anyhow::Result<Response> {
        if self.method == "isPrime" {
            Ok(Response::new(is_prime(self.number)))
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
