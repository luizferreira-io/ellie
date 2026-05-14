use crate::database::queries_tuning::{DatabaseColumnDefinition, DatabaseTable};
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub(crate) enum ActivityKey {
    Activity,
}

pub(crate) static QUERIES_ACTIVITY: LazyLock<HashMap<ActivityKey, DatabaseTable>> = LazyLock::new(
    || {
        let mut map = HashMap::new();

        map.insert(
            ActivityKey::Activity,
            DatabaseTable {
                columns: vec![
                    DatabaseColumnDefinition { field: "pid",                title: "PID",                width:  7 },
                    DatabaseColumnDefinition { field: "backend_type",       title: "Backend Type",       width: 30 },
                    DatabaseColumnDefinition { field: "database_name",      title: "Database",           width: 20 },
                    DatabaseColumnDefinition { field: "user_name",          title: "User",               width: 15 },
                    DatabaseColumnDefinition { field: "application_name",   title: "Application",        width: 30 },
                    DatabaseColumnDefinition { field: "client",             title: "Client",             width: 25 },
                    DatabaseColumnDefinition { field: "state",              title: "State",              width: 15 },
                    DatabaseColumnDefinition { field: "query",              title: "Query",              width: 50 },
                    DatabaseColumnDefinition { field: "connection_start",   title: "Connection Start",   width: 22 },
                    DatabaseColumnDefinition { field: "transaction_start",  title: "Transaction Start",  width: 22 },
                    DatabaseColumnDefinition { field: "query_start",        title: "Query Start",        width: 22 },
                    DatabaseColumnDefinition { field: "wait_event_type",    title: "Wait Event Type",    width: 16 },
                    DatabaseColumnDefinition { field: "wait_event",         title: "Wait Event",         width: 20 },
                    DatabaseColumnDefinition { field: "transaction_id",     title: "Transaction Id",     width: 15 },
                    DatabaseColumnDefinition { field: "transaction_xmin",   title: "Transaction xmin",   width: 15 },
                ],
                query: r###"
                SELECT pid::TEXT,
                       backend_type,
                       datname AS database_name,
                       usename AS "user_name",
                       application_name,
                       CASE
                           WHEN client_hostname IS NOT NULL THEN client_hostname || ' (' || client_addr::TEXT || ')'
                           ELSE client_addr::TEXT
                       END AS client,
                       state,
                       regexp_replace(query, '\s+', ' ', 'g') AS query,
                       date_trunc('second', backend_start)::TEXT AS connection_start,
                       date_trunc('second', xact_start)::TEXT AS transaction_start,
                       date_trunc('second', query_start)::TEXT AS query_start,
                       wait_event_type,
                       wait_event,
                       backend_xid::TEXT AS transaction_id,
                       backend_xmin::TEXT AS transaction_xmin
                  FROM pg_stat_activity
                 ORDER BY backend_start DESC, query_start DESC
                 LIMIT 1000;
                "###,
            },
        );

        map
    },
);
