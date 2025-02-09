use async_trait::async_trait;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io::Error as IoError;

use crate::http::errors::{ParseError, StatusError};
use crate::{Depot, Request, Response, Writer};

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// Errors that can happen inside salvo.
#[derive(Debug)]
pub enum Error {
    /// A error happened in hyper.
    Hyper(hyper::Error),
    /// A error happened in http parse.
    HttpParse(ParseError),
    /// A error from http response error status.
    HttpStatus(StatusError),
    /// Std io error.
    Io(IoError),
    /// Std io error.
    SerdeJson(serde_json::Error),
    /// A anyhow error.
    #[cfg(feature = "anyhow")]
    #[cfg_attr(docsrs, doc(cfg(feature = "anyhow")))]
    Anyhow(anyhow::Error),
    /// A custom error that does not fall under any other error kind.
    Other(BoxedError),
}

impl Error {
    /// Create a custom error.
    pub fn other(error: impl Into<BoxedError>) -> Self {
        Self::Other(error.into())
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Hyper(e) => Display::fmt(e, f),
            Self::HttpParse(e) => Display::fmt(e, f),
            Self::HttpStatus(e) => Display::fmt(e, f),
            Self::Io(e) => Display::fmt(e, f),
            Self::SerdeJson(e) => Display::fmt(e, f),
            #[cfg(feature = "anyhow")]
            Self::Anyhow(e) => Display::fmt(e, f),
            Self::Other(e) => Display::fmt(e, f),
        }
    }
}

impl StdError for Error {}

impl From<Infallible> for Error {
    fn from(infallible: Infallible) -> Error {
        match infallible {}
    }
}
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::Hyper(err)
    }
}
impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::HttpParse(err)
    }
}
impl From<StatusError> for Error {
    fn from(err: StatusError) -> Error {
        Error::HttpStatus(err)
    }
}
impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::SerdeJson(err)
    }
}
#[cfg(feature = "anyhow")]
#[cfg_attr(docsrs, doc(cfg(feature = "anyhow")))]
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Error {
        Error::Anyhow(err)
    }
}

impl From<BoxedError> for Error {
    fn from(err: BoxedError) -> Error {
        Error::Other(err)
    }
}

#[async_trait]
impl Writer for Error {
    #[inline]
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let status_error = match self {
            Error::HttpStatus(e) => e,
            _ => StatusError::internal_server_error(),
        };
        res.set_status_error(status_error);
    }
}
#[cfg(feature = "anyhow")]
#[cfg_attr(docsrs, doc(cfg(feature = "anyhow")))]
#[async_trait]
impl Writer for anyhow::Error {
    #[inline]
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.set_status_error(StatusError::internal_server_error());
    }
}

#[cfg(test)]
mod tests {
    use crate::http::*;

    use super::*;

    #[tokio::test]
    #[cfg(feature = "anyhow")]
    async fn test_anyhow() {
        let mut req = Request::default();
        let mut res = Response::default();
        let mut depot = Depot::new();
        let e: anyhow::Error = anyhow::anyhow!("detail message");
        e.write(&mut req, &mut depot, &mut res).await;
        assert_eq!(res.status_code(), Some(StatusCode::INTERNAL_SERVER_ERROR));
    }

    #[tokio::test]
    async fn test_error() {
        let mut req = Request::default();
        let mut res = Response::default();
        let mut depot = Depot::new();

        let e = Error::Other("detail message".into());
        e.write(&mut req, &mut depot, &mut res).await;
        assert_eq!(res.status_code(), Some(StatusCode::INTERNAL_SERVER_ERROR));
    }
}
