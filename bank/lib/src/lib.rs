use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Request {
    Insert { timestamp: i32, price: i32 },
    Query { mintime: i32, maxtime: i32 },
}

impl Request {
    pub fn deserialize(buf: &[u8]) -> Result<Self> {
        if buf.len() == 9 {
            match buf[0] {
                b'I' => Ok(Self::Insert {
                    timestamp: i32::from_be_bytes(buf[1..5].try_into()?),
                    price: i32::from_be_bytes(buf[5..9].try_into()?),
                }),
                b'Q' => Ok(Self::Query {
                    mintime: i32::from_be_bytes(buf[1..5].try_into()?),
                    maxtime: i32::from_be_bytes(buf[5..9].try_into()?),
                }),
                other => Err(Error::InvalidTag(other as char))?,
            }
        } else {
            Err(Error::LengthError(buf.len()))?
        }
    }

    pub fn serialize(self, dest: &mut [u8]) -> Result<()> {
        if dest.len() == 9 {
            match self {
                Self::Insert { timestamp, price } => {
                    dest[0] = b'I';
                    dest[1..5].copy_from_slice(&timestamp.to_be_bytes());
                    dest[5..9].copy_from_slice(&price.to_be_bytes());
                    Ok(())
                }
                Self::Query { mintime, maxtime } => {
                    dest[0] = b'Q';
                    dest[1..5].copy_from_slice(&mintime.to_be_bytes());
                    dest[5..9].copy_from_slice(&maxtime.to_be_bytes());
                    Ok(())
                }
            }
        } else {
            Err(Error::LengthError(dest.len()))?
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Expected a buffer of exactly length 9, got: {0}")]
    LengthError(usize),
    #[error("A message tag must be either I or Q, got {0}")]
    InvalidTag(char),
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn request() -> impl Strategy<Value = Request> {
        prop_oneof![
            (any::<i32>(), any::<i32>())
                .prop_map(|(timestamp, price)| Request::Insert { timestamp, price }),
            (any::<i32>(), any::<i32>())
                .prop_map(|(mintime, maxtime)| Request::Query { mintime, maxtime }),
        ]
    }

    proptest! {
        #[test]
        fn request_roundtrip(req in request()) {
            let mut buf = [0;9];
            req.serialize(&mut buf).unwrap();
            let new_req = Request::deserialize(&buf).unwrap();
            prop_assert_eq!(req, new_req);
        }
    }
}
