use std::io::{self, BufRead};
use std::marker::PhantomData;

use serde::de::DeserializeOwned;

pub mod locate_manifest;
pub mod read_manifest;

pub struct MessageIter<R, M> {
    _m: PhantomData<M>,
    input: R,
}

impl<R: BufRead, M: DeserializeOwned> Iterator for MessageIter<R, M> {
    type Item = io::Result<M>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        self.input
            .read_line(&mut line)
            .map(|n| {
                if n == 0 {
                    None
                } else {
                    if line.ends_with('\n') {
                        line.truncate(line.len() - 1);
                    }
                    let mut deserializer = serde_json::Deserializer::from_str(&line);
                    M::deserialize(&mut deserializer).ok()
                }
            })
            .transpose()
    }
}
