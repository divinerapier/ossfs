#[derive(Debug)]
pub enum Error {
    Fuse(libc::c_int),
    Backend(String),
    IO(std::io::Error),
    Nix(nix::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Fuse(code) => write!(f, "[fuse] {}", code),
            Error::Backend(message) => write!(f, "[backend] {}", message),
            Error::IO(io_error) => io_error.fmt(f),
            Error::Nix(e) => e.fmt(f),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}

impl<T> From<rusoto_core::RusotoError<T>> for Error
where
    T: 'static + std::fmt::Display + std::error::Error,
{
    fn from(e: rusoto_core::RusotoError<T>) -> Error {
        Error::Backend(format!("{}", e))
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Backend(format!("hyper error: {:?}", e))
    }
}
