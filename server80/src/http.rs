use std::fmt::Display;

#[derive(Debug, Clone)]
pub(crate) enum Method {
    Get(Box<str>),
}

#[derive(Clone, Copy)]
pub(crate) enum Code {
    Unknown = 0,
    Ok = 200,
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    NotImplemented = 501,
    InternalServerError = 500,
    ServiceUnavailable = 503,
}

impl From<i32> for Code {
    fn from(value: i32) -> Self {
        match value {
            200 => Self::Ok,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            500 => Self::InternalServerError,
            501 => Self::NotImplemented,
            503 => Self::ServiceUnavailable,
            _ => Self::Unknown,
        }
    }
}

impl From<std::io::Error> for Code {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::Forbidden,
            _ => Self::InternalServerError,
        }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            *self as i32,
            match self {
                Code::Unknown => "Unknown",
                Code::Ok => "Ok",
                Code::BadRequest => "Bad Request",
                Code::Forbidden => "Forbidden",
                Code::NotFound => "Not Found",
                Code::NotImplemented => "Not Implemented",
                Code::InternalServerError => "Internal Server Error",
                Code::ServiceUnavailable => "Service Unavailable",
            }
        )
    }
}
