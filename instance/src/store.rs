/// Postgres implementation for Store interface.
///
///
/// This expects to be connected to a DB that has a table to tables, `cursors` and `wal` that
/// should be created with the following CREATE statements.
///
/// ```sql
/// CREATE TABLE cursors (
///     worker VARCHAR(100) PRIMARY KEY,
///     logseq BIGINT NOT NULL
/// );
///
/// CREATE TABLE wal (
///     logseq BIGSERIAL PRIMARY KEY,
///     logentry BYTEA NOT NULL
/// );
/// ```
use balius_runtime::{
    store::{AtomicUpdate, LogEntry, LogSeq, StoreTrait},
    AtomicUpdateTrait, Block, ChainPoint, Error,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use prost::Message;
use std::{collections::BTreeSet, sync::Arc};
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

pub struct PostgresStore {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    shard: String,
}

impl PostgresStore {
    pub fn new(pool: &Pool<PostgresConnectionManager<NoTls>>, shard: &str) -> Self {
        Self {
            pool: pool.clone(),
            shard: shard.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl StoreTrait for PostgresStore {
    async fn find_chain_point(&self, seq: LogSeq) -> Result<Option<ChainPoint>, Error> {
        let conn =
            self.pool.get().await.map_err(|err| {
                Error::Store(format!("failed to get connection for store: {}", err))
            })?;
        match conn
            .query_opt(
                "SELECT logentry FROM wal WHERE logseq = $1::BIGINT AND shard = $2::TEXT",
                &[&(seq as i64), &self.shard],
            )
            .await
            .map_err(|err| Error::Store(format!("Failed to query store: {}", err)))?
        {
            Some(row) => {
                let bytes: Vec<u8> = row.get(0);
                let entry: LogEntry = prost::Message::decode(bytes.as_slice())
                    .map_err(|err| Error::Store(format!("Failed to decode logentry: {}", err)))?;
                let block = Block::from_bytes(&entry.next_block);

                Ok(Some(block.chain_point()))
            }
            None => Ok(None),
        }
    }

    async fn write_ahead(
        &mut self,
        undo_blocks: &[Block],
        next_block: &Block,
    ) -> Result<LogSeq, Error> {
        let entry = LogEntry {
            next_block: next_block.to_bytes(),
            undo_blocks: undo_blocks.iter().map(|x| x.to_bytes()).collect(),
        };
        let conn =
            self.pool.get().await.map_err(|err| {
                Error::Store(format!("failed to get connection for store: {}", err))
            })?;
        match conn
            .query_opt(
                "INSERT INTO wal (logentry, shard)
                 VALUES ($1::BYTEA, $2::TEXT)
                 RETURNING logseq;",
                &[&entry.encode_to_vec(), &self.shard],
            )
            .await
            .map_err(|err| Error::Store(format!("Failed to query store: {}", err)))?
        {
            Some(row) => {
                let seq: i64 = row.get(0);
                Ok(seq as u64)
            }
            None => Err(Error::Store("failed to get logseq".to_string())),
        }
    }

    async fn get_worker_cursor(&self, id: &str) -> Result<Option<LogSeq>, Error> {
        let conn =
            self.pool.get().await.map_err(|err| {
                Error::Store(format!("failed to get connection for store: {}", err))
            })?;
        match conn
            .query_opt(
                "SELECT logseq FROM cursors WHERE worker = $1::TEXT AND shard = $2::TEXT",
                &[&id, &self.shard],
            )
            .await
            .map_err(|err| Error::Store(format!("Failed to query store: {}", err)))?
        {
            Some(row) => {
                let seq: i64 = row.get(0);
                Ok(Some(seq as u64))
            }
            None => Ok(None),
        }
    }

    async fn start_atomic_update(&self, log_seq: LogSeq) -> Result<AtomicUpdate, Error> {
        Ok(AtomicUpdate::Custom(Arc::new(Mutex::new(
            PostgresAtomicUpdate::new(&self.pool, log_seq, &self.shard),
        ))))
    }
}

pub struct PostgresAtomicUpdate {
    cache: BTreeSet<String>,
    pool: Pool<PostgresConnectionManager<NoTls>>,
    log_seq: LogSeq,
    shard: String,
}
impl PostgresAtomicUpdate {
    pub fn new(
        pool: &Pool<PostgresConnectionManager<NoTls>>,
        log_seq: LogSeq,
        shard: &str,
    ) -> Self {
        Self {
            pool: pool.clone(),
            log_seq,
            cache: Default::default(),
            shard: shard.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AtomicUpdateTrait for PostgresAtomicUpdate {
    async fn update_worker_cursor(&mut self, id: &str) -> Result<(), Error> {
        let _ = self.cache.insert(id.to_string());

        Ok(())
    }

    async fn commit(&mut self) -> Result<(), Error> {
        let mut conn =
            self.pool.get().await.map_err(|err| {
                Error::Store(format!("failed to get connection for store: {}", err))
            })?;

        let txn = conn
            .transaction()
            .await
            .map_err(|err| Error::Store(format!("failed to get connection for store: {}", err)))?;

        for worker in &self.cache {
            let _ = txn
                .query(
                    "INSERT INTO cursors (worker, logseq, shard)
                 VALUES ($1::TEXT, $2::BIGINT, $3::TEXT)
                 ON CONFLICT (worker, shard) 
                 DO UPDATE SET logseq = EXCLUDED.logseq;",
                    &[&worker, &(self.log_seq as i64), &self.shard],
                )
                .await
                .map_err(|err| Error::Store(format!("failed to query store: {}", err)))?;
        }

        txn.commit()
            .await
            .map_err(|err| Error::Store(format!("failed to commit transaction: {}", err)))?;

        Ok(())
    }
}
