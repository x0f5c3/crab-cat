use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug,  Error)]
pub enum Error {
    #[error("BLE error: {0}")]
    BTLE(#[from] btleplug::Error),
    #[error("Printer not found due to {0}")]
    PrinterNotFound(String),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("{0} not a command")]
    NOTACOMMAND(u8),
    #[cfg(feature = "cli")]
    #[error("Generic error: {0}")]
    Eyre(#[from] color_eyre::Report),
}