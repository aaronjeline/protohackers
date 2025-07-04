use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::ops::ControlFlow;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::error;

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

pub async fn process_requests(buf: &[u8], stream: &mut TcpStream) -> Result<ControlFlow<()>> {
    let source = String::from_utf8_lossy(buf);
    let rs = process_requests_(source.lines());
    let buf: Vec<u8> = rs
        .responses
        .into_iter()
        .flat_map(|response| response.serialize())
        .collect();
    stream.write_all(&buf).await?;
    if !rs.ok {
        stream.write_all(b"malformed request").await?;
        Ok(ControlFlow::Break(()))
    } else {
        Ok(ControlFlow::Continue(()))
    }
}

#[derive(Debug)]
struct Responses {
    responses: Vec<Response>,
    ok: bool,
}

fn process_requests_<'a>(lines: impl Iterator<Item = &'a str>) -> Responses {
    let mut responses = vec![];
    for line in lines {
        match process_request(line) {
            Ok(response) => responses.push(response),
            Err(e) => {
                error!("Error process request: {e}");
                return Responses {
                    responses,
                    ok: false,
                };
            }
        }
    }
    Responses {
        responses,
        ok: true,
    }
}

#[tracing::instrument(skip(buf))]
fn process_request(buf: &str) -> anyhow::Result<Response> {
    let request: Request = serde_json::from_str(buf)?;
    request.process()
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

fn is_prime_opt(x: i64) -> bool {
    if x <= 1 {
        false
    } else {
        let bound = x.isqrt() + 1;
        for i in 2..bound {
            if x % i == 0 {
                return false;
            }
        }
        true
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

    pub fn serialize(self) -> impl Iterator<Item = u8> {
        let s = serde_json::to_string(&self).unwrap();
        format!("{s}\n").bytes().collect::<Vec<u8>>().into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    use proptest::prelude::*;
    fn request() -> impl Strategy<Value = super::Request> {
        (-1000i64..=10000i64).prop_map(|number| super::Request {
            method: "isPrime".to_string(),
            number,
        })
    }

    fn response() -> impl Strategy<Value = super::Response> {
        proptest::bool::ANY.prop_map(|prime| super::Response {
            method: "isPrime",
            prime,
        })
    }

    fn requests() -> impl Strategy<Value = Vec<super::Request>> {
        proptest::collection::vec(request(), 0..40)
    }

    #[derive(Debug)]
    struct Malformed {
        before_malformed: usize,
        buf: String,
    }

    fn malformed() -> impl Strategy<Value = Malformed> {
        use std::iter::once;
        let strats = (requests(), gibberish(), requests());
        strats.prop_map(|(before, gib, after)| Malformed {
            before_malformed: before.len(),
            buf: before
                .iter()
                .map(|r| serde_json::to_string(r).unwrap())
                .chain(once(gib))
                .chain(after.iter().map(|r| serde_json::to_string(r).unwrap()))
                .collect::<Vec<_>>()
                .join("\n"),
        })
    }

    fn gibberish() -> impl Strategy<Value = String> {
        proptest::string::string_regex("a-zA-Z").unwrap()
    }

    fn is_prime(x: i64) -> bool {
        if x <= 1 {
            false
        } else {
            for i in 2..x {
                if x % i == 0 {
                    return false;
                }
            }
            true
        }
    }

    proptest! {

        #[test]
        fn is_prime_equivalence(x in -10000_i64..10000_i64) {
            prop_assert_eq!(is_prime(x), super::is_prime_opt(x));
        }


        #[test]
        fn reesponses_end_in_newlines(x in response()) {
            let response_buf = x.serialize();
            let response_str = String::from_utf8(response_buf.collect()).unwrap();
            prop_assert!(response_str.ends_with('\n'));
        }

        #[test]
        fn every_request_processed(requests in requests()) {
            let s = requests.iter().map(|r| serde_json::to_string(r).unwrap()).collect::<Vec<_>>().join("\n");
            let result = process_requests_(s.lines());
            prop_assert_eq!(requests.len(), result.responses.len());
            prop_assert!(result.ok);
        }

        #[test]
        fn up_to_malformed(m in malformed()) {
            let result = process_requests_(m.buf.lines());
            prop_assert_eq!(result.responses.len(), m.before_malformed);
            prop_assert!(!result.ok);
        }



    }
}
