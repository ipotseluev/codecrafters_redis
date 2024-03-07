//! This module contains the protocol implementation for the Redis protocol.
mod request;
mod response;
pub(crate) use request::Request;
pub(crate) use response::Response;

