use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug,  Error)]
pub enum Error {
    #[error("Bleasy error: {0}")]
    Bleasy(#[from] bleasy::Error),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error)
}