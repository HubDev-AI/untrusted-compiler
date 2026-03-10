use super::operations::LasmPreparedDbOperationParams;
use crate::lasm_db_runtime_postgres::{
    parse_lasm_postgres_query_template_and_params,
    parse_lasm_postgres_query_template_and_params_value,
};
use crate::lasm_db_runtime_sqlite::{
    parse_lasm_sqlite_query_params, parse_lasm_sqlite_query_params_value,
};
use crate::LasmDbRecordsAdapter;

pub(crate) fn parse_lasm_db_template_and_params(
    adapter: LasmDbRecordsAdapter,
    template: &str,
    params: &str,
    parsed_params: Option<&serde_json::Value>,
) -> Result<LasmPreparedDbOperationParams, String> {
    match adapter {
        LasmDbRecordsAdapter::Postgres => {
            let result = if let Some(parsed) = parsed_params {
                parse_lasm_postgres_query_template_and_params_value(template, parsed)
            } else {
                parse_lasm_postgres_query_template_and_params(template, params)
            };
            let (template, query_params) = result?;
            Ok(LasmPreparedDbOperationParams::Postgres {
                template,
                params: query_params,
            })
        }
        LasmDbRecordsAdapter::Sqlite => {
            let query_params = if let Some(parsed) = parsed_params {
                parse_lasm_sqlite_query_params_value(parsed)
            } else {
                parse_lasm_sqlite_query_params(params)
            }?;
            Ok(LasmPreparedDbOperationParams::Sqlite {
                params: query_params,
            })
        }
        LasmDbRecordsAdapter::RecordsLog => Ok(LasmPreparedDbOperationParams::None),
    }
}
