use crate::database::queries_tuning::{ColumnConstraint, DatabaseColumnDefinition, DatabaseTable};
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub(crate) enum DashboardKey {
    Instance,
    Settings,
    Extensions,
    Metrics,

    SessionsMetrics,
    // Dashboard tables
    ActivityByDatabase,
    SessionsByDatabase,
    SessionsByApplication,
    CacheHitByDatabase,
    SharedBuffersContentTop10,
}

pub(crate) static QUERIES_DASHBOARD: LazyLock<HashMap<DashboardKey, DatabaseTable>> = LazyLock::new(
    || {
        let mut map = HashMap::new();

        map.insert(
            DashboardKey::Instance,
            DatabaseTable {
                columns: vec![],
                query: r###"
                    SELECT version()                                                           AS version,
                        date_trunc('second', pg_postmaster_start_time())::TEXT                 AS start_time,
                        extract(epoch FROM (now() - pg_postmaster_start_time()))::BIGINT::TEXT AS uptime_secs
                "###,
            },
        );

        map.insert(
            DashboardKey::Settings,
            DatabaseTable {
                columns: vec![],
                query: r###"
                    SELECT name,
                        CASE
                            WHEN unit IS NULL OR unit = '' THEN setting
                            WHEN unit = 'kB'              THEN pg_size_pretty(setting::bigint * 1024)
                            WHEN unit = '8kB'             THEN pg_size_pretty(setting::bigint * 8192)
                            WHEN unit = 'MB'              THEN pg_size_pretty(setting::bigint * 1048576)
                            WHEN unit = 'GB'              THEN pg_size_pretty(setting::bigint * 1073741824)
                            ELSE setting || ' ' || unit
                        END AS value
                    FROM pg_settings
                    WHERE name IN (
                        'shared_buffers',
                        'effective_cache_size',
                        'maintenance_work_mem',
                        'work_mem',
                        'max_wal_size',
                        'max_worker_processes',
                        'max_parallel_workers'
                    )
                "###,
            },
        );

        map.insert(
            DashboardKey::Extensions,
            DatabaseTable {
                columns: vec![],
                query: r###"
                    SELECT extname || ' ' || extversion AS name_extension
                    FROM pg_extension
                    ORDER BY extname
                "###,
            },
        );

        map.insert(
            DashboardKey::Metrics,
            DatabaseTable {
                columns: vec![],
                query: r###"
                    SELECT
                        -- Chart: Cache hit ratio
                        COALESCE((SELECT SUM(blks_hit)::BIGINT            FROM pg_stat_database), 0)::TEXT AS blocks_hit,
                        COALESCE((SELECT SUM(blks_read)::BIGINT           FROM pg_stat_database), 0)::TEXT AS blocks_read,

                        -- Chart: Transactions/s
                        COALESCE((SELECT SUM(xact_commit + xact_rollback) FROM pg_stat_database), 0)::TEXT AS transactions,

                        -- Chart: Rollback/s
                        COALESCE((SELECT SUM(xact_rollback) FROM pg_stat_database), 0)::TEXT AS rollbacks,

                        -- Chart: Locks
                        (SELECT COUNT(*)::BIGINT FROM pg_locks)::TEXT AS locks,

                        -- Chart: Conflicts & deadlocks/s
                        COALESCE((SELECT SUM(conflicts) + SUM(deadlocks)  FROM pg_stat_database), 0)::TEXT AS conflicts

                        -- COALESCE((SELECT SUM(tup_fetched)                 FROM pg_stat_database), 0)::TEXT AS fetched,
                        -- COALESCE((SELECT SUM(tup_inserted)                FROM pg_stat_database), 0)::TEXT AS inserted,
                        -- COALESCE((SELECT SUM(tup_updated)                 FROM pg_stat_database), 0)::TEXT AS updated,
                        -- COALESCE((SELECT SUM(tup_deleted)                 FROM pg_stat_database), 0)::TEXT AS deleted,
                        -- COALESCE((SELECT SUM(temp_bytes)                  FROM pg_stat_database), 0)::TEXT AS temp_bytes,
                        -- COALESCE((SELECT buffers_clean::BIGINT            FROM pg_stat_bgwriter), 0)::TEXT AS buffers
                "###,
            },
        );

        map.insert(
            DashboardKey::SessionsMetrics,
            DatabaseTable {
                columns: vec![],
                query: r###"
                    SELECT (SELECT COUNT(*) FROM pg_stat_activity)::TEXT AS total_sessions,
                        (SELECT setting FROM pg_settings WHERE name = 'max_connections')::TEXT  AS max_connections
                "###,
            },
        );

        map.insert(
            DashboardKey::CacheHitByDatabase,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "database_name",   title: "Database",    width: 1,  constraint: ColumnConstraint::Fill   },
                    DatabaseColumnDefinition { field: "owner",           title: "Owner",       width: 1,  constraint: ColumnConstraint::Fill   },
                    DatabaseColumnDefinition { field: "cache_hit_ratio", title: "Cache Hit %", width: 12, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_database.datname AS database_name,
                        pg_get_userbyid(pg_database.datdba) AS owner,
                        pg_stat_database.blks_read::TEXT AS disk_reads,
                        pg_stat_database.blks_hit::TEXT AS cache_reads,
                        COALESCE(ROUND(100.0 * pg_stat_database.blks_hit / NULLIF(pg_stat_database.blks_hit + pg_stat_database.blks_read, 0), 2), 0.00)::TEXT AS cache_hit_ratio
                    FROM pg_stat_database
                    INNER JOIN pg_database ON pg_stat_database.datname = pg_database.datname
                    ORDER BY COALESCE(ROUND(100.0 * pg_stat_database.blks_hit / NULLIF(pg_stat_database.blks_hit + pg_stat_database.blks_read, 0), 2), 0.00) DESC
                    LIMIT 1000;
                "###,
            },
        );

        map.insert(
            DashboardKey::SharedBuffersContentTop10,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "database_name", title: "Database",   width: 1,  constraint: ColumnConstraint::Fill   },
                    DatabaseColumnDefinition { field: "object_name",   title: "Object",     width: 1,  constraint: ColumnConstraint::Fill   },
                    DatabaseColumnDefinition { field: "object_type",   title: "Type",       width: 1,  constraint: ColumnConstraint::Fill   },
                    DatabaseColumnDefinition { field: "cache_size",    title: "Cache Size", width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cache_percent", title: "Cache %",    width: 10, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_database.datname AS database_name,
                        pg_class.relname AS object_name,
                        CASE pg_class.relkind
                            WHEN 'r' THEN 'Table'
                            WHEN 'i' THEN 'Index'
                            WHEN 'S' THEN 'Sequence'
                            WHEN 'v' THEN 'View'
                            WHEN 'm' THEN 'Materialized View'
                            WHEN 't' THEN 'TOAST Table'
                            WHEN 'p' THEN 'Table Partition'
                            ELSE 'Other (' || pg_class.relkind::TEXT || ')'
                        END AS object_type,
                        pg_size_pretty(count(*) * 8192) AS cache_size,
                        ROUND(100.0 * count(*) / (SELECT setting FROM pg_settings WHERE name = 'shared_buffers')::integer, 2)::TEXT AS cache_percent
                    FROM pg_buffercache
                    INNER JOIN pg_database ON pg_buffercache.reldatabase = pg_database.oid
                    INNER JOIN pg_class ON pg_buffercache.relfilenode = pg_relation_filenode(pg_class.oid)
                    INNER JOIN pg_namespace ON pg_namespace.oid = pg_class.relnamespace
                    GROUP BY pg_database.datname, pg_namespace.nspname, pg_class.relname, pg_class.relowner, pg_class.relkind
                    ORDER BY cache_percent DESC
                    LIMIT 10
                "###,
            },
        );

        map.insert(
            DashboardKey::SessionsByDatabase,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "database_name", title: "Database", width: 20, constraint: ColumnConstraint::Fill   },
                    DatabaseColumnDefinition { field: "max_sessions",  title: "Maximum",  width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cur_sessions",  title: "Current",  width: 15, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_database.datname AS database_name,
                        CASE
						    WHEN pg_database.datconnlimit = -1 THEN '—' ELSE pg_database.datconnlimit::TEXT
						END AS max_sessions,
                        COALESCE(pg_stat_database.numbackends, 0)::TEXT AS cur_sessions
                    FROM pg_database
                    LEFT JOIN pg_stat_database ON pg_stat_database.datid = pg_database.oid
                    WHERE pg_database.datistemplate = false
                    ORDER BY pg_database.datname
                    LIMIT 1000;
                "###,
            },
        );

        map.insert(
            DashboardKey::SessionsByApplication,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "application_name",  title: "Application",  width: 25, constraint: ColumnConstraint::Fill   },
                    DatabaseColumnDefinition { field: "sessions",          title: "Sessions",     width: 15, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT COALESCE(NULLIF(application_name, ''), '(none)') AS application_name,
                        COUNT(*)::TEXT AS sessions
                    FROM pg_stat_activity
                    WHERE state IS NOT NULL
                    GROUP BY application_name
                    ORDER BY COUNT(*) DESC
                    LIMIT 1000;
                "###,
            },
        );

        map.insert(
            DashboardKey::ActivityByDatabase,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "database_name",   title: "Database",        width: 1,  constraint: ColumnConstraint::Fill   },
                    DatabaseColumnDefinition { field: "sessions",        title: "Sessions",        width: 11, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "commits",         title: "Commits",         width: 11, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "rollbacks",       title: "Rollbacks",       width: 11, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuples_returned", title: "Tuples Returned", width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuples_fetched",  title: "Tuples Fetched",  width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuples_inserted", title: "Tuples Inserted", width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuples_updated",  title: "Tuples Updated",  width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuples_deleted",  title: "Tuples Deleted",  width: 15, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT datname                AS database_name,
                        numbackends::TEXT      AS sessions,
                        xact_commit::TEXT      AS commits,
                        xact_rollback::TEXT    AS rollbacks,
                        tup_returned::TEXT     AS tuples_returned,
                        tup_fetched::TEXT      AS tuples_fetched,
                        tup_inserted::TEXT     AS tuples_inserted,
                        tup_updated::TEXT      AS tuples_updated,
                        tup_deleted::TEXT      AS tuples_deleted
                    FROM pg_stat_database
                    WHERE datname IS NOT NULL
                        AND datname NOT IN ('template0', 'template1')
                    ORDER BY datname
                    LIMIT 1000;
                "###,
            },
        );

        map
    },
);
