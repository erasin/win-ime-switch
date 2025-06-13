#[derive(thiserror::Error, Debug)]
pub enum ImeError {
    #[error("Unsupported language format: {0}")]
    Unsupported(String),
}
