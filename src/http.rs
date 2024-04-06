use std::collections::HashMap;

use bytes::Bytes;

pub enum HttpStatusCode {
    Success,
    Created,
    NotFound,
    ServerError,
    BadRequest,
}

pub struct HttpResponse {
    pub code: HttpStatusCode,
    pub body: Bytes,
    pub headers: Vec<Header>,
}

pub struct Header {
    pub name: Bytes,
    pub value: Bytes,
}

impl HttpStatusCode {
    pub fn header(self) -> Bytes {
        match self {
            Self::Success => "HTTP/1.1 200 OK".into(),
            Self::Created => "HTTP/1.1 201 OK".into(),
            Self::NotFound => "HTTP/1.1 404 Not Found".into(),
            Self::ServerError => "HTTP/1.1 500 Server Error".into(),
            Self::BadRequest => "HTTP/1.1 400 Bad Request".into(),
        }
    }
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub body: String,
}

pub enum HttpMethod {
    Unimplemented,
    Get,
    Post,
}
