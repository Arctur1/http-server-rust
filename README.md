[![progress-banner](https://backend.codecrafters.io/progress/http-server/02ae0321-82f8-4d92-a1e5-baf97014f684)](https://app.codecrafters.io/courses/http-server/overview)

This is a solution to the
["Build Your Own HTTP server" Challenge](https://app.codecrafters.io/courses/http-server/overview).

## Files Description

### src/config.rs

Config with `directory` flag value for file handlers.


### src/http.rs

Structures for http request and response.

### src/main.rs

Tokio tcp listener, minimal http server implementation, handlers and their registration.

### src/matcher.rs

Path struct implementation which can match http path with parameters and trailing wildcard.


### src/router.rs

Router implementation which uses path from matcher.rs.
