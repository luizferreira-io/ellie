pub(crate) fn get_str(row: &postgres::Row, col: &str) -> String {
    row.get::<_, Option<&str>>(col)
        .unwrap_or_default()
        .to_string()
}
