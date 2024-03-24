

use crate::config::Config;
pub use crate::matcher;
pub use crate::http;

type HandlerFn = fn(r: http::HttpRequest, config: &Config) -> http::HttpResponse;
type Route = (matcher::Path, HandlerFn);

#[derive(Clone)]
pub struct Router {
    handlers: Vec<Route>
}

impl Router {
    pub fn new() -> Self {
        Self {handlers: vec![]}
    }

    pub fn add(mut self, path: &str, handler: HandlerFn) -> Self {
        let path = matcher::Path::new(path.to_string());
        self.handlers.push((path, handler));
        self
    }

    pub fn match_url(self, path: &str) -> Option<(matcher::Match, HandlerFn)> {
        for route in self.handlers.iter() {
            if let Some(matches) = route.0.match_path(path.to_string()) {
                return Some((matches, route.1));
            }
        };

        None
    }
}

