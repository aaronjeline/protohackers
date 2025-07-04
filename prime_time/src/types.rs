use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    method: String,
    number: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    method: String,
    prime: bool,
}
