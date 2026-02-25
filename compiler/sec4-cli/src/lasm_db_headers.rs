use std::collections::BTreeMap;

pub(crate) const LASM_INTERNAL_DB_OP_HEADER: &str = "X-Sec4-Internal-Db-Op";
pub(crate) const LASM_INTERNAL_DB_HANDLE_HEADER: &str = "X-Sec4-Internal-Db";
pub(crate) const LASM_INTERNAL_DB_TEMPLATE_HEADER: &str = "X-Sec4-Internal-Db-Template";
pub(crate) const LASM_INTERNAL_DB_PARAMS_HEADER: &str = "X-Sec4-Internal-Db-Params";
pub(crate) const LASM_INTERNAL_DB_TX_HEADER: &str = "X-Sec4-Internal-Db-Tx";
pub(crate) const LASM_INTERNAL_DB_TX_DB_HEADER: &str = "X-Sec4-Internal-Db-Tx-Db";
pub(crate) const LASM_INTERNAL_DB_ROW_SCHEMA_HEADER: &str = "X-Sec4-Internal-Db-Row-Schema";
pub(crate) const LASM_INTERNAL_DB_OP_COUNT_HEADER: &str = "X-Sec4-Internal-Db-Op-Count";

pub(crate) fn clear_lasm_internal_db_response_markers(headers: &mut BTreeMap<String, String>) {
    headers.remove(LASM_INTERNAL_DB_OP_HEADER);
    headers.remove(LASM_INTERNAL_DB_HANDLE_HEADER);
    headers.remove(LASM_INTERNAL_DB_TEMPLATE_HEADER);
    headers.remove(LASM_INTERNAL_DB_PARAMS_HEADER);
    headers.remove(LASM_INTERNAL_DB_TX_HEADER);
    headers.remove(LASM_INTERNAL_DB_TX_DB_HEADER);
    headers.remove(LASM_INTERNAL_DB_ROW_SCHEMA_HEADER);
    headers.remove(LASM_INTERNAL_DB_OP_COUNT_HEADER);
}
