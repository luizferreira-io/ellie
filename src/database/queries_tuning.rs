use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone)]
pub(crate) enum ColumnConstraint {
    Length,
    Min,
    Max,
    Percentage,
    Fill,
}

#[derive(Clone)]
pub(crate) struct DatabaseColumnDefinition {
    pub(crate) field: &'static str,
    pub(crate) title: &'static str,
    pub(crate) width: u16,
    pub(crate) constraint: ColumnConstraint,
}

pub(crate) struct DatabaseTable {
    pub(crate) query: &'static str,
    pub(crate) columns: Vec<DatabaseColumnDefinition>,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub(crate) enum TuningKey {
    // Disk allocation
    DiskAllocationByTablespace,
    DiskAllocationByDatabase,
    DiskAllocationBySchema,
    DiskAllocationByTable,
    // Fragmentation
    BloatedTables,
    UpdatedTables,
    NewpageUpdatedTables,
    HotUpdatedTables,
    // Indexing
    MissingIndexes,
    UnusedIndexes,
    RedundantIndexes,
    // Shared Buffers
    SharedBuffersContentServer,
    SharedBuffersContentDatabase,
    CacheHitByDb,
    CacheHitByTable,
    // Queries
    TimeConsumingQueriesTotalServer,
    TimeConsumingQueriesAverageServer,
    TimeConsumingQueriesTotalDatabase,
    TimeConsumingQueriesAverageDatabase,
}

pub(crate) static QUERIES_TUNING: LazyLock<HashMap<TuningKey, DatabaseTable>> = LazyLock::new(
    || {
        let mut map = HashMap::new();

        // Disk allocation by tablespace
        map.insert(
            TuningKey::DiskAllocationByTablespace,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "tablespace_name", title: "Tablespace", width: 25, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",           title: "Owner",      width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "size",            title: "Size",       width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "path",            title: "Path",       width: 50, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT spcname AS tablespace_name,
                        spcowner::regrole::TEXT AS owner,
                        pg_size_pretty(pg_tablespace_size(oid)) AS size,
                        CASE
                            WHEN pg_tablespace_location(oid) = '' THEN 'PGDATA'
                            ELSE pg_tablespace_location(oid)
                        END AS path
                    FROM pg_tablespace
                    ORDER BY pg_tablespace_size(oid) DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Disk allocation by database
        map.insert(
            TuningKey::DiskAllocationByDatabase,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "db_name",         title: "Database",   width: 25, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",           title: "Owner",      width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "size",            title: "Size",       width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tablespace_name", title: "Tablespace", width: 25, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "collate",         title: "Collate",    width: 20, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_database.datname AS db_name,
                        pg_database.datdba::regrole::TEXT AS owner,
                        pg_size_pretty(pg_database_size(pg_database.datname)) AS size,
                        pg_tablespace.spcname AS tablespace_name,
                        pg_database.datcollate AS collate,
                        pg_database.datctype AS ctype
                    FROM pg_database
                    INNER JOIN pg_tablespace ON pg_database.dattablespace = pg_tablespace.oid
                    ORDER BY pg_database_size(pg_database.datname) DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Disk allocation by schema
        map.insert(
            TuningKey::DiskAllocationBySchema,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",     title: "Schema",     width: 25, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",           title: "Owner",      width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_size",      title: "Size",       width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "percentage",      title: "% of DB",    width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tablespace_name", title: "Tablespace", width: 25, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_namespace.nspname AS schema_name,
                        pg_namespace.nspowner::regrole::TEXT AS owner,
                        pg_size_pretty(SUM(pg_total_relation_size(pg_class.oid))) AS total_size,
                        ROUND((SUM(pg_total_relation_size(pg_class.oid))::numeric / NULLIF(pg_database_size(current_database()), 0)) * 100, 2)::TEXT AS percentage,
                        COALESCE(string_agg(DISTINCT t.spcname, ', '), 'pg_default') AS tablespace_name
                    FROM pg_class
                    INNER JOIN pg_namespace ON pg_namespace.oid = pg_class.relnamespace
                    LEFT JOIN pg_tablespace t ON pg_class.reltablespace = t.oid
                    WHERE pg_class.relkind IN ('r', 'm')
                    GROUP BY pg_namespace.nspname, pg_namespace.nspowner
                    ORDER BY SUM(pg_total_relation_size(pg_class.oid)) DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Disk allocation by table
        map.insert(
            TuningKey::DiskAllocationByTable,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",       title: "Schema",           width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",        title: "Table",            width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",             title: "Owner",            width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "disk_size",         title: "Size",             width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "disk_total_size",   title: "Total Size",       width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "live_tuples",       title: "Live Tuples",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "dead_tuples",       title: "Dead Tuples",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "bloat_percentage",  title: "Bloat %",          width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "vacuum_count",      title: "Vacuums",          width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_vacuum",       title: "Last Vacuum",      width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autovacuum_count",  title: "Auto Vacuums",     width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autovacuum",   title: "Last Autovacuum",  width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "analyze_count",     title: "Analyzes",         width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_analyze",      title: "Last Analyze",     width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autoanalyze_count", title: "Auto Analyzes",    width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autoanalyze",  title: "Last Autoanalyze", width: 22, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_stat_user_tables.schemaname AS schema_name,
                        pg_stat_user_tables.relname AS table_name,
                        pg_get_userbyid(pg_class.relowner) AS owner,
                        pg_size_pretty(pg_relation_size(pg_stat_user_tables.relid)) AS disk_size,
                        pg_size_pretty(pg_total_relation_size(pg_stat_user_tables.relid)) AS disk_total_size,
                        pg_stat_user_tables.n_live_tup::TEXT AS live_tuples,
                        pg_stat_user_tables.n_dead_tup::TEXT AS dead_tuples,
                        COALESCE(
                            ROUND((pg_stat_user_tables.n_dead_tup::numeric / NULLIF(pg_stat_user_tables.n_live_tup + pg_stat_user_tables.n_dead_tup, 0)) * 100, 2),
                            0.00
                        )::TEXT AS bloat_percentage,
                        pg_stat_user_tables.vacuum_count::TEXT,
                        date_trunc('second', pg_stat_user_tables.last_vacuum)::TEXT AS last_vacuum,
                        pg_stat_user_tables.autovacuum_count::TEXT,
                        date_trunc('second', pg_stat_user_tables.last_autovacuum)::TEXT AS last_autovacuum,
                        pg_stat_user_tables.analyze_count::TEXT,
                        date_trunc('second', pg_stat_user_tables.last_analyze)::TEXT AS last_analyze,
                        pg_stat_user_tables.autoanalyze_count::TEXT,
                        date_trunc('second', pg_stat_user_tables.last_autoanalyze)::TEXT AS last_autoanalyze
                    FROM pg_stat_user_tables
                    INNER JOIN pg_class ON pg_stat_user_tables.relid = pg_class.oid
                    ORDER BY pg_total_relation_size(pg_stat_user_tables.relid) DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Fragmentation - Bloated tables
        map.insert(
            TuningKey::BloatedTables,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",       title: "Schema",           width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",        title: "Table",            width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",             title: "Owner",            width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "bloat_size",        title: "Bloat Size",       width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "bloat_percentage",  title: "Bloat %",          width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "disk_size",         title: "Size",             width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "disk_total_size",   title: "Total Size",       width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "live_tuples",       title: "Live Tuples",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "dead_tuples",       title: "Dead Tuples",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "fill_factor",       title: "Fill Factor %",    width: 13, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "vacuum_count",      title: "Vacuums",          width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_vacuum",       title: "Last Vacuum",      width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autovacuum_count",  title: "Auto Vacuums",     width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autovacuum",   title: "Last Autovacuum",  width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "analyze_count",     title: "Analyzes",         width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_analyze",      title: "Last Analyze",     width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autoanalyze_count", title: "Auto Analyzes",    width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autoanalyze",  title: "Last Autoanalyze", width: 22, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_stat_user_tables.schemaname AS schema_name,
                        pg_stat_user_tables.relname AS table_name,
                        pg_get_userbyid(pg_class.relowner) AS owner,
                        pg_size_pretty(
                            ROUND(
                                COALESCE(
                                    pg_stat_user_tables.n_dead_tup::numeric / NULLIF(pg_stat_user_tables.n_live_tup + pg_stat_user_tables.n_dead_tup, 0),
                                    0
                                ) * pg_total_relation_size(pg_stat_user_tables.relid)
                                , 0
                            )
                        ) AS bloat_size,
                        COALESCE(
                            ROUND(
                                (pg_stat_user_tables.n_dead_tup::numeric / NULLIF(pg_stat_user_tables.n_live_tup + pg_stat_user_tables.n_dead_tup, 0)) * 100,
                                2
                            ), 0.00
                        )::TEXT AS bloat_percentage,
                        pg_size_pretty(pg_relation_size(pg_stat_user_tables.relid)) AS disk_size,
                        pg_size_pretty(pg_total_relation_size(pg_stat_user_tables.relid)) AS disk_total_size,
                        pg_stat_user_tables.n_live_tup::TEXT AS live_tuples,
                        pg_stat_user_tables.n_dead_tup::TEXT AS dead_tuples,
                        COALESCE(
                            (SELECT option_value::int FROM pg_options_to_table(pg_class.reloptions) WHERE option_name = 'fillfactor'),
                            100
                        )::TEXT AS fill_factor,
                        pg_stat_user_tables.vacuum_count::TEXT,
                        date_trunc('second', pg_stat_user_tables.last_vacuum)::TEXT AS last_vacuum,
                        pg_stat_user_tables.autovacuum_count::TEXT,
                        date_trunc('second', pg_stat_user_tables.last_autovacuum)::TEXT AS last_autovacuum,
                        pg_stat_user_tables.analyze_count::TEXT,
                        date_trunc('second', pg_stat_user_tables.last_analyze)::TEXT AS last_analyze,
                        pg_stat_user_tables.autoanalyze_count::TEXT,
                        date_trunc('second', pg_stat_user_tables.last_autoanalyze)::TEXT AS last_autoanalyze
                    FROM pg_stat_user_tables
                    INNER JOIN pg_class ON pg_class.oid = pg_stat_user_tables.relid
                    INNER JOIN pg_namespace ON pg_namespace.oid = pg_class.relnamespace
                    ORDER BY
                        COALESCE(
                            ROUND(
                                (pg_stat_user_tables.n_dead_tup::numeric / NULLIF(pg_stat_user_tables.n_live_tup + pg_stat_user_tables.n_dead_tup, 0)) * 100,
                                2
                            ),
                            0.00
                        ) DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Fragmentation - Updated tables
        map.insert(
            TuningKey::UpdatedTables,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",        title: "Schema",            width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",         title: "Table",             width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",              title: "Owner",             width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_activity",     title: "Total Activity",    width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "inserts",            title: "Inserts",           width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "updates",            title: "Updates",           width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "deletes",            title: "Deletes",           width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "inserts_percentage", title: "Ins %",             width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "updates_percentage", title: "Upd %",             width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "deletes_percentage", title: "Del %",             width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "newpage_updates",    title: "New page Updates",  width: 18, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "hot_updates",        title: "HOT Updates",       width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "live_tuples",        title: "Live Tuples",       width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "dead_tuples",        title: "Dead Tuples",       width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "bloat_percentage",   title: "Bloat %",           width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "vacuum_count",       title: "Vacuums",           width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_vacuum",        title: "Last Vacuum",       width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autovacuum_count",   title: "Auto Vacuums",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autovacuum",    title: "Last Autovacuum",   width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "analyze_count",      title: "Analyzes",          width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_analyze",       title: "Last Analyze",      width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autoanalyze_count",  title: "Auto Analyzes",     width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autoanalyze",   title: "Last Autoanalyze",  width: 22, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_stat_user_tables.schemaname AS schema_name,
                        pg_stat_user_tables.relname AS table_name,
                        pg_get_userbyid(pg_class.relowner) AS owner,
                        (n_tup_ins + n_tup_upd + n_tup_del)::TEXT AS total_activity,
                        n_tup_ins::TEXT AS inserts,
                        n_tup_upd::TEXT AS updates,
                        n_tup_del::TEXT AS deletes,
                        COALESCE(ROUND((n_tup_ins::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS inserts_percentage,
                        COALESCE(ROUND((n_tup_upd::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS updates_percentage,
                        COALESCE(ROUND((n_tup_del::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS deletes_percentage,
                        n_tup_newpage_upd::TEXT AS newpage_updates,
                        n_tup_hot_upd::TEXT AS hot_updates,
                        n_live_tup::TEXT AS live_tuples,
                        n_dead_tup::TEXT AS dead_tuples,
                        COALESCE(ROUND((n_dead_tup::numeric / NULLIF(n_live_tup + n_dead_tup, 0)) * 100, 2), 0.00)::TEXT AS bloat_percentage,
                        vacuum_count::TEXT,
                        date_trunc('second', last_vacuum)::TEXT      AS last_vacuum,
                        autovacuum_count::TEXT,
                        date_trunc('second', last_autovacuum)::TEXT  AS last_autovacuum,
                        analyze_count::TEXT,
                        date_trunc('second', last_analyze)::TEXT     AS last_analyze,
                        autoanalyze_count::TEXT,
                        date_trunc('second', last_autoanalyze)::TEXT AS last_autoanalyze
                    FROM pg_stat_user_tables
                    INNER JOIN pg_class ON pg_class.oid = pg_stat_user_tables.relid
                    ORDER BY (n_tup_ins + n_tup_upd + n_tup_del) DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Fragmentation - New page updated tables
        map.insert(
            TuningKey::NewpageUpdatedTables,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",            title: "Schema",            width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",             title: "Table",             width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",                  title: "Owner",             width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "newpage_updates",        title: "New page Upd",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "newpage_upd_percentage", title: "New page Upd %",    width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_activity",         title: "Total Activity",    width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "inserts",                title: "Inserts",           width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "updates",                title: "Updates",           width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "deletes",                title: "Deletes",           width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "inserts_percentage",     title: "Ins %",             width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "updates_percentage",     title: "Upd %",             width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "deletes_percentage",     title: "Del %",             width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "hot_updates",            title: "HOT Updates",       width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "live_tuples",            title: "Live Tuples",       width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "dead_tuples",            title: "Dead Tuples",       width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "bloat_percentage",       title: "Bloat %",           width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "vacuum_count",           title: "Vacuums",           width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_vacuum",            title: "Last Vacuum",       width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autovacuum_count",       title: "Auto Vacuums",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autovacuum",        title: "Last Autovacuum",   width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "analyze_count",          title: "Analyzes",          width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_analyze",           title: "Last Analyze",      width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autoanalyze_count",      title: "Auto Analyzes",     width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autoanalyze",       title: "Last Autoanalyze",  width: 22, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_stat_user_tables.schemaname AS schema_name,
                        pg_stat_user_tables.relname AS table_name,
                        pg_get_userbyid(pg_class.relowner) AS owner,
                        n_tup_newpage_upd::TEXT AS newpage_updates,
                        COALESCE(ROUND((n_tup_newpage_upd::numeric / NULLIF(n_tup_upd, 0)) * 100, 2), 0.00)::TEXT AS newpage_upd_percentage,
                        (n_tup_ins + n_tup_upd + n_tup_del)::TEXT AS total_activity,
                        n_tup_ins::TEXT AS inserts,
                        n_tup_upd::TEXT AS updates,
                        n_tup_del::TEXT AS deletes,
                        COALESCE(ROUND((n_tup_ins::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS inserts_percentage,
                        COALESCE(ROUND((n_tup_upd::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS updates_percentage,
                        COALESCE(ROUND((n_tup_del::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS deletes_percentage,
                        n_tup_hot_upd::TEXT AS hot_updates,
                        n_live_tup::TEXT AS live_tuples,
                        n_dead_tup::TEXT AS dead_tuples,
                        COALESCE(ROUND((n_dead_tup::numeric / NULLIF(n_live_tup + n_dead_tup, 0)) * 100, 2), 0.00)::TEXT AS bloat_percentage,
                        vacuum_count::TEXT,
                        date_trunc('second', last_vacuum)::TEXT      AS last_vacuum,
                        autovacuum_count::TEXT,
                        date_trunc('second', last_autovacuum)::TEXT  AS last_autovacuum,
                        analyze_count::TEXT,
                        date_trunc('second', last_analyze)::TEXT     AS last_analyze,
                        autoanalyze_count::TEXT,
                        date_trunc('second', last_autoanalyze)::TEXT AS last_autoanalyze
                    FROM pg_stat_user_tables
                    INNER JOIN pg_class ON pg_class.oid = pg_stat_user_tables.relid
                    ORDER BY n_tup_newpage_upd DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Fragmentation - HOT updated tables
        map.insert(
            TuningKey::HotUpdatedTables,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",        title: "Schema",           width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",         title: "Table",            width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",              title: "Owner",            width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "hot_updates",        title: "HOT Upd",          width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "hot_upd_percentage", title: "HOT Upd %",        width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_activity",     title: "Total Activity",   width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "inserts",            title: "Inserts",          width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "updates",            title: "Updates",          width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "deletes",            title: "Deletes",          width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "inserts_percentage", title: "Ins %",            width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "updates_percentage", title: "Upd %",            width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "deletes_percentage", title: "Del %",            width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "newpage_updates",    title: "Newpage Updates",  width: 18, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "live_tuples",        title: "Live Tuples",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "dead_tuples",        title: "Dead Tuples",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "bloat_percentage",   title: "Bloat %",          width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "vacuum_count",       title: "Vacuums",          width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_vacuum",        title: "Last Vacuum",      width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autovacuum_count",   title: "Auto Vacuums",     width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autovacuum",    title: "Last Autovacuum",  width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "analyze_count",      title: "Analyzes",         width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_analyze",       title: "Last Analyze",     width: 22, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "autoanalyze_count",  title: "Auto Analyzes",    width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "last_autoanalyze",   title: "Last Autoanalyze", width: 22, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_stat_user_tables.schemaname AS schema_name,
                        pg_stat_user_tables.relname AS table_name,
                        pg_get_userbyid(pg_class.relowner) AS owner,
                        n_tup_hot_upd::TEXT AS hot_updates,
                        COALESCE(ROUND((n_tup_hot_upd::numeric / NULLIF(n_tup_upd, 0)) * 100, 2), 0.00)::TEXT AS hot_upd_percentage,
                        (n_tup_ins + n_tup_upd + n_tup_del)::TEXT AS total_activity,
                        n_tup_ins::TEXT AS inserts,
                        n_tup_upd::TEXT AS updates,
                        n_tup_del::TEXT AS deletes,
                        COALESCE(ROUND((n_tup_ins::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS inserts_percentage,
                        COALESCE(ROUND((n_tup_upd::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS updates_percentage,
                        COALESCE(ROUND((n_tup_del::numeric / NULLIF(n_tup_ins + n_tup_upd + n_tup_del, 0)) * 100, 2), 0.00)::TEXT AS deletes_percentage,
                        n_tup_newpage_upd::TEXT AS newpage_updates,
                        n_live_tup::TEXT AS live_tuples,
                        n_dead_tup::TEXT AS dead_tuples,
                        COALESCE(ROUND((n_dead_tup::numeric / NULLIF(n_live_tup + n_dead_tup, 0)) * 100, 2), 0.00)::TEXT AS bloat_percentage,
                        vacuum_count::TEXT,
                        date_trunc('second', last_vacuum)::TEXT      AS last_vacuum,
                        autovacuum_count::TEXT,
                        date_trunc('second', last_autovacuum)::TEXT  AS last_autovacuum,
                        analyze_count::TEXT,
                        date_trunc('second', last_analyze)::TEXT     AS last_analyze,
                        autoanalyze_count::TEXT,
                        date_trunc('second', last_autoanalyze)::TEXT AS last_autoanalyze
                    FROM pg_stat_user_tables
                    INNER JOIN pg_class ON pg_class.oid = pg_stat_user_tables.relid
                    ORDER BY n_tup_hot_upd DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Indexing - Missing indexes
        map.insert(
            TuningKey::MissingIndexes,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",    title: "Schema",           width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",     title: "Table",            width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",          title: "Owner",            width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "seq_scans",      title: "Seq Scans",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuple_reads",    title: "Tuple Reads",      width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuples_average", title: "Avg Tup/Seq Scan", width: 16, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "index_scans",    title: "Index Scans",      width: 12, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_stat_user_tables.schemaname AS schema_name,
                        pg_stat_user_tables.relname AS table_name,
                        pg_get_userbyid(pg_class.relowner) AS owner,
                        seq_scan::TEXT AS seq_scans,
                        seq_tup_read::TEXT AS tuple_reads,
                        (seq_tup_read / NULLIF(seq_scan, 0))::TEXT AS tuples_average,
                        idx_scan::TEXT AS index_scans
                    FROM pg_stat_user_tables
                    INNER JOIN pg_class ON pg_class.oid = pg_stat_user_tables.relid
                    WHERE seq_scan > 0
                    ORDER BY seq_tup_read DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Indexing - Unused indexes
        map.insert(
            TuningKey::UnusedIndexes,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",  title: "Schema",      width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",   title: "Table",       width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",        title: "Owner",       width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "index_name",   title: "Index",       width: 40, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "used_times",   title: "Used Times",  width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "index_size",   title: "Index Size",  width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuples_read",  title: "Tuples Read", width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "tuples_fetch", title: "Tuples Fetch", width: 15, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_stat_user_indexes.schemaname AS schema_name,
                        pg_stat_user_indexes.relname AS table_name,
                        pg_get_userbyid(pg_class.relowner) AS owner,
                        pg_stat_user_indexes.indexrelname AS index_name,
                        pg_stat_user_indexes.idx_scan::TEXT AS used_times,
                        pg_size_pretty(pg_relation_size(pg_stat_user_indexes.indexrelid)) AS index_size,
                        pg_stat_user_indexes.idx_tup_read::TEXT AS tuples_read,
                        pg_stat_user_indexes.idx_tup_fetch::TEXT AS tuples_fetch
                    FROM pg_stat_user_indexes
                    INNER JOIN pg_index ON pg_stat_user_indexes.indexrelid = pg_index.indexrelid
                    INNER JOIN pg_class ON pg_class.oid = pg_stat_user_indexes.relid
                    WHERE pg_index.indisunique = false
                    ORDER BY pg_stat_user_indexes.idx_scan ASC,
                        pg_relation_size(pg_stat_user_indexes.indexrelid) DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Indexing - Redundant indexes
        map.insert(
            TuningKey::RedundantIndexes,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",          title: "Schema",          width:  15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",           title: "Table",           width:  30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",                title: "Owner",           width:  20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "redundant_index_name", title: "Redundant Index", width:  30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "covering_index_name",  title: "Covering Index",  width:  30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "redundant_index_ddl",  title: "Redundant DDL",   width: 120, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "covering_index_ddl",   title: "Covering DDL",    width: 120, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT tableschema.nspname AS schema_name,
                        indextable.relname AS table_name,
                        pg_get_userbyid(indextable.relowner) AS owner,
                        old_index.relname AS redundant_index_name,
                        pg_get_indexdef(redundant_index.indexrelid, 0, true) AS redundant_index_ddl,
                        new_index.relname AS covering_index_name,
                        pg_get_indexdef(covering_index.indexrelid, 0, true) AS covering_index_ddl
                    FROM pg_index AS redundant_index
                    INNER JOIN pg_index AS covering_index ON covering_index.indrelid = redundant_index.indrelid
                    INNER JOIN pg_class AS indextable ON indextable.oid = redundant_index.indrelid
                    INNER JOIN pg_class AS old_index ON old_index.oid = redundant_index.indexrelid
                    INNER JOIN pg_class AS new_index ON new_index.oid = covering_index.indexrelid
                    INNER JOIN pg_namespace AS tableschema ON tableschema.oid = indextable.relnamespace
                    WHERE redundant_index.indexrelid <> covering_index.indexrelid
                        AND redundant_index.indkey[0:array_upper(redundant_index.indkey, 1)] = covering_index.indkey[0:array_upper(redundant_index.indkey, 1)]
                        AND array_upper(redundant_index.indkey, 1) < array_upper(covering_index.indkey, 1)
                        AND indextable.relkind = 'r'
                    ORDER BY redundant_index_name
                    LIMIT 1000;
                "###,
            },
        );

        // Shared Buffers - Content (Server)
        map.insert(
            TuningKey::SharedBuffersContentServer,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "database_name", title: "Database",   width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "schema_name",   title: "Schema",     width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "object_name",   title: "Object",     width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",         title: "Owner",      width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "object_type",   title: "Type",       width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cache_size",    title: "Cache Size", width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cache_percent", title: "Cache %",    width: 10, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_database.datname AS database_name,
                        pg_namespace.nspname AS schema_name,
                        pg_class.relname AS object_name,
                        COALESCE(pg_get_userbyid(pg_class.relowner), '') AS owner,
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
                    LIMIT 1000;
                "###,
            },
        );

        // Shared Buffers - Content (Database)
        map.insert(
            TuningKey::SharedBuffersContentDatabase,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "schema_name",   title: "Schema",     width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "object_name",   title: "Object",     width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",         title: "Owner",      width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "object_type",   title: "Type",       width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cache_size",    title: "Cache Size", width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cache_percent", title: "Cache %",    width: 10, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_namespace.nspname AS schema_name,
                        pg_class.relname AS object_name,
                        COALESCE(pg_get_userbyid(pg_class.relowner), '') AS owner,
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
                    WHERE pg_database.datname = current_database()
                    GROUP BY pg_database.datname, pg_namespace.nspname, pg_class.relname, pg_class.relowner, pg_class.relkind
                    ORDER BY cache_percent DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Shared Buffers - Cache Hit by Database
        map.insert(
            TuningKey::CacheHitByDb,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "database_name",   title: "Database",    width: 25, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "owner",           title: "Owner",       width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "disk_reads",      title: "Disk Reads",  width: 14, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cache_reads",     title: "Cache Reads", width: 14, constraint: ColumnConstraint::Length },
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
                    ORDER BY cache_hit_ratio DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Shared Buffers - Cache Hit by Table
        map.insert(
            TuningKey::CacheHitByTable,
            DatabaseTable {
                columns: vec![
                    DatabaseColumnDefinition { field: "database_name",         title: "Database",             width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "schema_name",           title: "Schema",               width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_name",            title: "Table",                width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "table_owner",           title: "Owner",                width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_cache_hit_ratio", title: "Cache Hit %",          width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "disk_reads_heap",       title: "Disk Reads (Heap)",    width: 16, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cache_reads_heap",      title: "Cache Reads (Heap)",   width: 18, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "disk_reads_index",      title: "Disk Reads (Index)",   width: 17, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "cache_reads_index",     title: "Cache Reads (Index)",  width: 19, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT current_database() AS database_name,
                        pg_statio_user_tables.schemaname AS schema_name,
                        pg_statio_user_tables.relname AS table_name,
                        COALESCE(pg_get_userbyid(pg_class.relowner), '') AS table_owner,
                        pg_statio_user_tables.heap_blks_read::TEXT AS disk_reads_heap,
                        pg_statio_user_tables.heap_blks_hit::TEXT AS cache_reads_heap,
                        pg_statio_user_tables.idx_blks_read::TEXT AS disk_reads_index,
                        pg_statio_user_tables.idx_blks_hit::TEXT AS cache_reads_index,
                        COALESCE(ROUND(100.0 * (pg_statio_user_tables.heap_blks_hit + pg_statio_user_tables.idx_blks_hit) /
                        NULLIF(pg_statio_user_tables.heap_blks_hit + pg_statio_user_tables.heap_blks_read + pg_statio_user_tables.idx_blks_hit + pg_statio_user_tables.idx_blks_read, 0), 2), 0.00)::TEXT AS total_cache_hit_ratio
                    FROM pg_statio_user_tables
                    JOIN pg_class ON pg_statio_user_tables.relid = pg_class.oid
                    ORDER BY (pg_statio_user_tables.heap_blks_read + pg_statio_user_tables.idx_blks_read) DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Queries - Time consuming queries total - Instance
        map.insert(
            TuningKey::TimeConsumingQueriesTotalServer,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "database",            title: "Database",            width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "username",            title: "User",                width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "query",               title: "Query",               width: 60, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_exec_sec",      title: "Total Exec (s)",      width: 16, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "calls",               title: "Calls",               width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "min_exec_sec",        title: "Min Exec (s)",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "max_exec_sec",        title: "Max Exec (s)",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "mean_exec_sec",       title: "Mean Exec (s)",       width: 13, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "rows",                title: "Rows",                width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "shared_blks_read",    title: "Shared Blks Read",    width: 18, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "shared_blks_written", title: "Shared Blks Written", width: 20, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_database.datname AS database,
                        pg_authid.rolname AS username,
                        regexp_replace(pg_stat_statements.query, '\s+', ' ', 'g') AS query,
                        ROUND((pg_stat_statements.total_exec_time / 1000.0)::numeric, 3)::TEXT AS total_exec_sec,
                        pg_stat_statements.calls::TEXT,
                        ROUND((pg_stat_statements.min_exec_time / 1000.0)::numeric, 3)::TEXT AS min_exec_sec,
                        ROUND((pg_stat_statements.max_exec_time / 1000.0)::numeric, 3)::TEXT AS max_exec_sec,
                        ROUND((pg_stat_statements.mean_exec_time / 1000.0)::numeric, 3)::TEXT AS mean_exec_sec,
                        pg_stat_statements.rows::TEXT,
                        pg_stat_statements.shared_blks_read::TEXT,
                        pg_stat_statements.shared_blks_written::TEXT
                    FROM pg_stat_statements
                    JOIN pg_database ON pg_stat_statements.dbid = pg_database.oid
                    JOIN pg_authid ON pg_stat_statements.userid = pg_authid.oid
                    ORDER BY pg_stat_statements.total_exec_time DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Queries - Time consuming queries average - Instance
        map.insert(
            TuningKey::TimeConsumingQueriesAverageServer,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "database",            title: "Database",            width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "username",            title: "User",                width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "query",               title: "Query",               width: 60, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "mean_exec_sec",       title: "Mean Exec (s)",       width: 13, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_exec_sec",      title: "Total Exec (s)",      width: 16, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "calls",               title: "Calls",               width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "min_exec_sec",        title: "Min Exec (s)",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "max_exec_sec",        title: "Max Exec (s)",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "rows",                title: "Rows",                width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "shared_blks_read",    title: "Shared Blks Read",    width: 18, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "shared_blks_written", title: "Shared Blks Written", width: 20, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                   SELECT pg_database.datname AS database,
                        pg_authid.rolname AS username,
                        regexp_replace(pg_stat_statements.query, '\s+', ' ', 'g') AS query,
                        ROUND((pg_stat_statements.mean_exec_time / 1000.0)::numeric, 3)::TEXT AS mean_exec_sec,
                        ROUND((pg_stat_statements.total_exec_time / 1000.0)::numeric, 3)::TEXT AS total_exec_sec,
                        pg_stat_statements.calls::TEXT,
                        ROUND((pg_stat_statements.min_exec_time / 1000.0)::numeric, 3)::TEXT AS min_exec_sec,
                        ROUND((pg_stat_statements.max_exec_time / 1000.0)::numeric, 3)::TEXT AS max_exec_sec,
                        pg_stat_statements.rows::TEXT,
                        pg_stat_statements.shared_blks_read::TEXT,
                        pg_stat_statements.shared_blks_written::TEXT
                    FROM pg_stat_statements
                    JOIN pg_database ON pg_stat_statements.dbid = pg_database.oid
                    JOIN pg_authid ON pg_stat_statements.userid = pg_authid.oid
                    ORDER BY pg_stat_statements.mean_exec_time DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Queries - Time consuming queries total - Database
        map.insert(
            TuningKey::TimeConsumingQueriesTotalDatabase,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "username",            title: "User",                width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "query",               title: "Query",               width: 60, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_exec_sec",      title: "Total Exec (s)",      width: 16, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "calls",               title: "Calls",               width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "min_exec_sec",        title: "Min Exec (s)",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "max_exec_sec",        title: "Max Exec (s)",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "mean_exec_sec",       title: "Mean Exec (s)",       width: 13, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "rows",                title: "Rows",                width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "shared_blks_read",    title: "Shared Blks Read",    width: 18, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "shared_blks_written", title: "Shared Blks Written", width: 20, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                    SELECT pg_database.datname AS database,
                        pg_authid.rolname AS username,
                        regexp_replace(pg_stat_statements.query, '\s+', ' ', 'g') AS query,
                        ROUND((pg_stat_statements.total_exec_time / 1000.0)::numeric, 3)::TEXT AS total_exec_sec,
                        pg_stat_statements.calls::TEXT,
                        ROUND((pg_stat_statements.min_exec_time / 1000.0)::numeric, 3)::TEXT AS min_exec_sec,
                        ROUND((pg_stat_statements.max_exec_time / 1000.0)::numeric, 3)::TEXT AS max_exec_sec,
                        ROUND((pg_stat_statements.mean_exec_time / 1000.0)::numeric, 3)::TEXT AS mean_exec_sec,
                        pg_stat_statements.rows::TEXT,
                        pg_stat_statements.shared_blks_read::TEXT,
                        pg_stat_statements.shared_blks_written::TEXT
                    FROM pg_stat_statements
                    JOIN pg_database ON pg_stat_statements.dbid = pg_database.oid
                    JOIN pg_authid ON pg_stat_statements.userid = pg_authid.oid
                    WHERE pg_database.datname = current_database()
                    ORDER BY pg_stat_statements.total_exec_time DESC
                    LIMIT 1000;
                "###,
            },
        );

        // Queries - Time consuming queries average - Database
        map.insert(
            TuningKey::TimeConsumingQueriesAverageDatabase,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "username",            title: "User",                width: 15, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "query",               title: "Query",               width: 60, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "mean_exec_sec",       title: "Mean Exec (s)",       width: 13, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "total_exec_sec",      title: "Total Exec (s)",      width: 16, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "calls",               title: "Calls",               width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "min_exec_sec",        title: "Min Exec (s)",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "max_exec_sec",        title: "Max Exec (s)",        width: 12, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "rows",                title: "Rows",                width: 10, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "shared_blks_read",    title: "Shared Blks Read",    width: 18, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "shared_blks_written", title: "Shared Blks Written", width: 20, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                   SELECT pg_database.datname AS database,
                        pg_authid.rolname AS username,
                        regexp_replace(pg_stat_statements.query, '\s+', ' ', 'g') AS query,
                        ROUND((pg_stat_statements.mean_exec_time / 1000.0)::numeric, 3)::TEXT AS mean_exec_sec,
                        ROUND((pg_stat_statements.total_exec_time / 1000.0)::numeric, 3)::TEXT AS total_exec_sec,
                        pg_stat_statements.calls::TEXT,
                        ROUND((pg_stat_statements.min_exec_time / 1000.0)::numeric, 3)::TEXT AS min_exec_sec,
                        ROUND((pg_stat_statements.max_exec_time / 1000.0)::numeric, 3)::TEXT AS max_exec_sec,
                        pg_stat_statements.rows::TEXT,
                        pg_stat_statements.shared_blks_read::TEXT,
                        pg_stat_statements.shared_blks_written::TEXT
                    FROM pg_stat_statements
                    JOIN pg_database ON pg_stat_statements.dbid = pg_database.oid
                    JOIN pg_authid ON pg_stat_statements.userid = pg_authid.oid
                    WHERE pg_database.datname = current_database()
                    ORDER BY pg_stat_statements.mean_exec_time DESC
                    LIMIT 1000;
                "###,
            },
        );

        map
    },
);
