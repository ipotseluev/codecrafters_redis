//! Handles client requests.
use crate::{
    error::RedisError,
    protocol::{self, Request, Response},
    storage,
};

/// Processes client requests.
pub(crate) struct RequestProcessor {
    storage: storage::Storage,
}

impl RequestProcessor {
    pub(crate) fn new() -> Self {
        Self {
            storage: storage::Storage::instance(),
        }
    }

    /// Executes handler for client's request. Generates response.
    pub(crate) async fn process_request(&self, request: Request) -> Result<Response, RedisError> {
        match request {
            Request::Ping => Ok(Response::Ping),
            Request::Echo(arg) => Ok(Response::Echo(arg)),
            Request::Set(request) => self.process_request_set(request).await,
            Request::Get(key) => self.process_request_get(key).await,
        }
    }

    /// Get value from the storage by a key (returns Nil response if key is absent).
    async fn process_request_get(&self, key: String) -> Result<Response, RedisError> {
        match self.storage.get(&key).await {
            Some(value) => Ok(Response::Get(Some(value))),
            None => Ok(Response::Get(None)),
        }
    }

    /// Store key-value pair in the storage.
    async fn process_request_set(&self, request: protocol::Set) -> Result<Response, RedisError> {
        self.storage.set(request).await;
        Ok(Response::Ok)
    }
}
