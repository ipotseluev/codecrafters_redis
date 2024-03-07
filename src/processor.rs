//! Handles client requests.

use crate::{
    error::RedisError,
    protocol::{Request, Response},
    storage,
};

/// Executes handler for client's request. Generates response.
pub(crate) async fn process_request(request: Request) -> Result<Response, RedisError> {
    match request {
        Request::Ping => Ok(Response::Ping),
        Request::Echo(arg) => Ok(Response::Echo(arg)),
        Request::Set(key, value) => process_request_set(key, value).await,
        Request::Get(key) => process_request_get(key).await,
    }
}

/// Get value from the storage by a key (returns Nil response if key is absent).
async fn process_request_get(key: String) -> Result<Response, RedisError> {
    match storage::get(&key).await {
        Some(value) => Ok(Response::Get(Some(value))),
        None => Ok(Response::Get(None)),
    }
}

/// Store key-value pair in the storage.
async fn process_request_set(key: String, value: String) -> Result<Response, RedisError> {
    storage::set(key, value).await;
    Ok(Response::Ok)
}
