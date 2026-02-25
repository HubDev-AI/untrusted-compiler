use std::collections::BTreeMap;

pub(crate) const LASM_INTERNAL_DB_OP_HEADER: &str = "X-Sec4-Internal-Db-Op";
pub(crate) const LASM_INTERNAL_DB_HANDLE_HEADER: &str = "X-Sec4-Internal-Db";
pub(crate) const LASM_INTERNAL_DB_TEMPLATE_HEADER: &str = "X-Sec4-Internal-Db-Template";
pub(crate) const LASM_INTERNAL_DB_PARAMS_HEADER: &str = "X-Sec4-Internal-Db-Params";
pub(crate) const LASM_INTERNAL_DB_TX_HEADER: &str = "X-Sec4-Internal-Db-Tx";
pub(crate) const LASM_INTERNAL_DB_TX_DB_HEADER: &str = "X-Sec4-Internal-Db-Tx-Db";
pub(crate) const LASM_INTERNAL_DB_ROW_SCHEMA_HEADER: &str = "X-Sec4-Internal-Db-Row-Schema";
pub(crate) const LASM_INTERNAL_DB_OP_COUNT_HEADER: &str = "X-Sec4-Internal-Db-Op-Count";

pub(crate) fn lasm_internal_db_indexed_header(header_name: &str, index: usize) -> String {
    format!("{header_name}-{index}")
}

pub(crate) fn clear_lasm_internal_db_response_markers(headers: &mut BTreeMap<String, String>) {
    let exact_headers = [
        LASM_INTERNAL_DB_OP_HEADER,
        LASM_INTERNAL_DB_HANDLE_HEADER,
        LASM_INTERNAL_DB_TEMPLATE_HEADER,
        LASM_INTERNAL_DB_PARAMS_HEADER,
        LASM_INTERNAL_DB_TX_HEADER,
        LASM_INTERNAL_DB_TX_DB_HEADER,
        LASM_INTERNAL_DB_ROW_SCHEMA_HEADER,
        LASM_INTERNAL_DB_OP_COUNT_HEADER,
    ];
    let exact_headers_lower = exact_headers
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let indexed_prefixes_lower = [
        LASM_INTERNAL_DB_OP_HEADER,
        LASM_INTERNAL_DB_HANDLE_HEADER,
        LASM_INTERNAL_DB_TEMPLATE_HEADER,
        LASM_INTERNAL_DB_PARAMS_HEADER,
        LASM_INTERNAL_DB_TX_HEADER,
        LASM_INTERNAL_DB_TX_DB_HEADER,
        LASM_INTERNAL_DB_ROW_SCHEMA_HEADER,
    ]
    .iter()
    .map(|value| format!("{}-", value.to_ascii_lowercase()))
    .collect::<Vec<_>>();

    headers.retain(|key, _| {
        let lowered = key.to_ascii_lowercase();
        if exact_headers_lower.iter().any(|entry| entry == &lowered) {
            return false;
        }
        !indexed_prefixes_lower
            .iter()
            .any(|entry| lowered.starts_with(entry))
    });
}
