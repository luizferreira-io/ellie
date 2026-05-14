use crate::database::queries_tuning::{DatabaseColumnDefinition, DatabaseTable};
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub(crate) enum SettingsKey {
    Settings,
}

pub(crate) static QUERIES_SETTINGS: LazyLock<HashMap<SettingsKey, DatabaseTable>> = LazyLock::new(
    || {
        let mut map = HashMap::new();

        map.insert(
            SettingsKey::Settings,
            DatabaseTable {
                columns: vec![
                    DatabaseColumnDefinition { field: "name",            title: "Name",            width: 40 },
                    DatabaseColumnDefinition { field: "current_value",   title: "Current Value",   width: 15 },
                    DatabaseColumnDefinition { field: "reset_value",     title: "Reset Value",     width: 15 },
                    DatabaseColumnDefinition { field: "boot_value",      title: "Boot Value",      width: 15 },
                    DatabaseColumnDefinition { field: "type",            title: "Type",            width: 10 },
                    DatabaseColumnDefinition { field: "unit",            title: "Unit",            width:  6 },
                    DatabaseColumnDefinition { field: "pending_restart", title: "Pending Restart", width:  8 },
                    DatabaseColumnDefinition { field: "possible_values", title: "Possible Values", width: 30 },
                    DatabaseColumnDefinition { field: "context",         title: "Context",         width: 20 },
                    DatabaseColumnDefinition { field: "category",        title: "Category",        width: 64 },
                    DatabaseColumnDefinition { field: "source",          title: "Source",          width: 60 },
                    DatabaseColumnDefinition { field: "description",     title: "_description",    width:  0 }, // Ghost column
                ],
                query: r###"
                SELECT name,
                       setting AS current_value,
                       boot_val AS boot_value,
                       reset_val AS reset_value,
                       vartype AS type,
                       unit,
                       pending_restart::TEXT,
                       COALESCE((min_val || ' to ' || max_val)::TEXT, enumvals::TEXT) AS "possible_values",
                       context,
                       category,
                       COALESCE((sourcefile || ':' || sourceline)::TEXT, '') AS "source",
                       short_desc || COALESCE(' ' || extra_desc, '') AS description
                  FROM pg_settings
                 ORDER BY name
                 LIMIT 1000;
                "###,
            },
        );

        map
    },
);
