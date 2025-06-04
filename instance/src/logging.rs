/// Postgres backend for Key Value interface.
///
///
/// This expects to be connected to a DB that has a table named `kv`, which should be created
/// using the following insert statement:
///
/// ```sql
///
/// CREATE TABLE logs (
///     id BIGSERIAL PRIMARY KEY,
///     timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
///     worker VARCHAR(100) NOT NULL,
///     level VARCHAR(50) NOT NULL, -- e.g., INFO, WARN, ERROR, DEBUG
///     message TEXT NOT NULL,
///     context TEXT NOT NULL
/// );
use balius_runtime::{logging::LoggerProvider, wit::balius::app::logging::Level};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub struct PostgresLogger {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}
impl From<&Pool<PostgresConnectionManager<NoTls>>> for PostgresLogger {
    fn from(value: &Pool<PostgresConnectionManager<NoTls>>) -> Self {
        Self {
            pool: value.clone(),
        }
    }
}

#[async_trait::async_trait]
impl LoggerProvider for PostgresLogger {
    async fn log(&mut self, worker_id: &str, level: Level, context: String, message: String) {
        let conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::error!("Failed to get connection for Postgres logger: {}", err);
                return;
            }
        };

        let level = match level {
            Level::Info => "INFO",
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Critical => "CRITICAL",
        };

        if let Err(err) = conn
            .query(
                "INSERT INTO logs (worker, level, context, message)
                 VALUES ($1::TEXT, $2::TEXT, $3::TEXT, $4::TEXT)",
                &[&worker_id, &level, &context, &message],
            )
            .await
        {
            tracing::warn!(
                worker_id = worker_id,
                err = err.to_string(),
                "failed to log into postgres"
            )
        }
    }
}
