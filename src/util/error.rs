use std::io;

#[derive(Debug)]
pub enum JsonIOError {
    Json(json::Error),
    IO(io::Error),
}

impl From<json::Error> for JsonIOError {
    fn from(e: json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<io::Error> for JsonIOError {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}
