#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidArgument,
    InvalidState,
    Io,
    Format,
}
pub type Result<T> = std::result::Result<T, Error>;
