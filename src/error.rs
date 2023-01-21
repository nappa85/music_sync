use std::io;

#[derive(Debug)]
pub enum Error {
    ChannelClosed,
    Io(io::Error),
    Musicbrainz(musicbrainz_rs::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<musicbrainz_rs::Error> for Error {
    fn from(value: musicbrainz_rs::Error) -> Self {
        Error::Musicbrainz(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ChannelClosed => write!(f, "channel closed"),
            Error::Io(e) => e.fmt(f),
            Error::Musicbrainz(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
