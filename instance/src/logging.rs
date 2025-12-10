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
use chrono::{DateTime, Utc};
use tokio_postgres::NoTls;

struct LogRow {
    pub timestamp: DateTime<Utc>,
    pub worker: String,
    pub level: String,
    pub context: String,
    pub message: String,
}

pub struct PostgresLogger {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    buffer: Vec<LogRow>,
    buffer_size: usize,
}
impl From<&Pool<PostgresConnectionManager<NoTls>>> for PostgresLogger {
    fn from(value: &Pool<PostgresConnectionManager<NoTls>>) -> Self {
        Self {
            pool: value.clone(),
            buffer: Vec::with_capacity(1024),
            buffer_size: 1024,
        }
    }
}

impl PostgresLogger {
    fn should_flush(&self) -> bool {
        self.buffer.len() >= self.buffer_size
    }

    async fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let mut sql = String::new();
        sql.push_str("INSERT INTO logs (timestamp, worker, level, context, message) VALUES ");

        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            Vec::with_capacity(self.buffer.len() * 5);

        for (i, row) in self.buffer.iter().enumerate() {
            if i > 0 {
                sql.push(',');
            }

            let base = i * 5;
            sql.push_str(&format!(
                "(${}::TIMESTAMPTZ, ${}::TEXT, ${}::TEXT, ${}::TEXT, ${}::TEXT)",
                base + 1,
                base + 2,
                base + 3,
                base + 4,
                base + 5
            ));

            params.push(&row.timestamp);
            params.push(&row.worker);
            params.push(&row.level);
            params.push(&row.context);
            params.push(&row.message);
        }

        let conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::error!("Failed to get connection for Postgres logger: {}", err);
                return;
            }
        };

        if let Err(err) = conn.query(&sql, &params).await {
            tracing::warn!(err = %err, "failed to flush batched logs");
        }

        self.buffer.clear();
    }
}

#[async_trait::async_trait]
impl LoggerProvider for PostgresLogger {
    async fn log(&mut self, worker_id: &str, level: Level, context: String, message: String) {
        let level = match level {
            Level::Info => "INFO",
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Critical => "CRITICAL",
        };

        let row = LogRow {
            timestamp: Utc::now(),
            worker: worker_id.to_string(),
            level: level.to_string(),
            context,
            message,
        };

        self.buffer.push(row);

        if self.should_flush() {
            self.flush().await;
        }
    }
}
