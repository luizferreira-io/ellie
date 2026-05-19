use crate::database::queries_tuning::{ColumnConstraint, DatabaseColumnDefinition, DatabaseTable};
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub(crate) enum FileSettingsKey {
    FileSettings,
}

pub(crate) static QUERIES_FILE_SETTINGS: LazyLock<HashMap<FileSettingsKey, DatabaseTable>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();

        map.insert(
            FileSettingsKey::FileSettings,
            DatabaseTable {
                #[rustfmt::skip]
                columns: vec![
                    DatabaseColumnDefinition { field: "sequence", title: "Seq",     width: 5,  constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "name",     title: "Name",    width: 35, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "value",    title: "Value",   width: 20, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "applied",  title: "Applied", width: 8,  constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "error",    title: "Error",   width: 30, constraint: ColumnConstraint::Length },
                    DatabaseColumnDefinition { field: "source",   title: "Source",  width: 60, constraint: ColumnConstraint::Length },
                ],
                query: r###"
                SELECT seqno::TEXT AS sequence,
                       name,
                       setting AS value,
                       applied::TEXT,
                       error,
                       COALESCE((sourcefile || ':' || sourceline)::TEXT, '') AS source
                  FROM pg_file_settings
                 ORDER BY seqno::INT
                 LIMIT 1000;
                "###,
            },
        );

        map
    });
