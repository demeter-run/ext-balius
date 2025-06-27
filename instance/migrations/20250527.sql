CREATE TABLE kv (
  worker VARCHAR(255) NOT NULL,
  key VARCHAR(255) NOT NULL,
  value BYTEA,
  PRIMARY KEY (worker, key)
);

CREATE TABLE logs (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  worker VARCHAR(100) NOT NULL,
  level VARCHAR(50) NOT NULL,
  message TEXT NOT NULL,
  context TEXT NOT NULL
);

CREATE TABLE cursors (
    worker VARCHAR(100) NOT NULL,
    shard VARCHAR(100) NOT NULL,
    logseq BIGINT NOT NULL,
    PRIMARY KEY (worker, shard)
);

CREATE TABLE wal (
    logseq BIGSERIAL NOT NULL,
    shard VARCHAR(100) NOT NULL,
    logentry BYTEA NOT NULL,
    PRIMARY KEY (logseq, shard)
);
