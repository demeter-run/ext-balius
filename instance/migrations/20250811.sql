-- NOTE: This should be executed in the db where pg_cron is installed (likely `postgres`)
SELECT cron.schedule_in_database(
    'hourly-multi-shard-wal-cleanup',
    '@hourly',
    $$
        DELETE FROM wal w
        WHERE EXISTS (
            SELECT 1
            FROM (
                SELECT shard, MAX(logseq) AS max_logseq
                FROM wal
                GROUP BY shard
            ) AS s
            WHERE s.shard = w.shard
            AND w.logseq < s.max_logseq - 6480  -- Approximate maximum amount of mutable blocks
        );
    $$,
    'balius'
);
