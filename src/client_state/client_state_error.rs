use crate::config::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ClientStateError {
    ReservedClientID,
    NonExistentClientID,
}

impl Display for ClientStateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(ClientStateError::{:?}", self)
    }
}

impl Into<Error> for ClientStateError {
    fn into(self) -> Error {
        Error::from(self.to_string())
    }
}
