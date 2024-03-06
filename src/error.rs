#[derive(thiserror::Error, Debug)]
pub enum RedisError {
    #[error("Failed to deserialize string '{raw_redis_message}' into Redis message. {details}")]
    DeserializationError {
        raw_redis_message: String,
        details: String,
    },
}
