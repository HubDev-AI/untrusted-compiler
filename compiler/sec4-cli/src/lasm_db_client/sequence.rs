use crate::{LasmDbRecordsAdapter, LasmDynamicResponseState};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

pub(crate) enum LasmExecTxSource {
    AllocateFromDb(i64),
    ExistingTx(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LasmInternalDbOperationSequenceValidationError {
    InvalidMarker,
    MarkerValueTooSmall,
    IndexedMarkersRequireCount,
    ExceedsMaximum { maximum: usize },
}

#[derive(Debug, Default)]
pub(crate) struct LasmInternalDbSequenceState {
    tx_handles_by_source: BTreeMap<i64, i64>,
    tx_handles: BTreeSet<i64>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LasmInternalDbSequenceFailure {
    pub(crate) code: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) message: &'static str,
    pub(crate) status: u16,
}

#[derive(Default)]
pub(crate) struct LasmSequenceExecTxPreparation {
    pub(crate) tx_header: Option<String>,
    pub(crate) tx_db_header: Option<String>,
    pub(crate) retain_tx_for_sequence: bool,
    pub(crate) allocated_tx_source: Option<i64>,
}

impl LasmInternalDbSequenceFailure {
    fn exec_tx_validation(message: &'static str) -> Self {
        Self {
            code: "DB.EXEC_TX_INVALID",
            kind: "validation",
            message,
            status: 400,
        }
    }

    fn tx_internal(message: &'static str) -> Self {
        Self {
            code: "DB.TX_INTERNAL",
            kind: "internal",
            message,
            status: 500,
        }
    }
}

pub(crate) fn validate_lasm_internal_db_operation_sequence_count(
    raw_operation_count: Option<String>,
    has_indexed_headers: bool,
    operation_sequence_max: usize,
) -> Result<usize, LasmInternalDbOperationSequenceValidationError> {
    let operation_count = if let Some(raw_operation_count) = raw_operation_count {
        let trimmed = raw_operation_count.trim();
        let Some(parsed) = trimmed.parse::<usize>().ok() else {
            return Err(LasmInternalDbOperationSequenceValidationError::InvalidMarker);
        };
        if parsed < 2 {
            return Err(LasmInternalDbOperationSequenceValidationError::MarkerValueTooSmall);
        }
        parsed
    } else {
        0
    };

    if operation_count == 0 && has_indexed_headers {
        return Err(LasmInternalDbOperationSequenceValidationError::IndexedMarkersRequireCount);
    }
    if operation_count > operation_sequence_max {
        return Err(
            LasmInternalDbOperationSequenceValidationError::ExceedsMaximum {
                maximum: operation_sequence_max,
            },
        );
    }

    Ok(operation_count)
}

impl LasmInternalDbSequenceState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn tracked_tx_for_source(&self, db_source: i64) -> Option<i64> {
        self.tx_handles_by_source.get(&db_source).copied()
    }

    pub(crate) fn track_tx_handle(&mut self, tx_handle: i64) {
        self.tx_handles.insert(tx_handle);
    }

    pub(crate) fn track_source_tx_handle(&mut self, db_source: i64, tx_handle: i64) {
        self.tx_handles_by_source.insert(db_source, tx_handle);
    }

    pub(crate) fn cleanup(
        &self,
        dynamic_state: &Mutex<LasmDynamicResponseState>,
        db_records_adapter: LasmDbRecordsAdapter,
        operation_succeeded: bool,
    ) {
        super::cleanup_lasm_internal_db_sequence_tx_handles(
            dynamic_state,
            db_records_adapter,
            self.tx_handles_by_source.values().copied(),
            operation_succeeded,
        );
        super::cleanup_lasm_internal_db_sequence_tx_handles(
            dynamic_state,
            db_records_adapter,
            self.tx_handles.iter().copied(),
            operation_succeeded,
        );
    }

    pub(crate) fn prepare_exec_tx_sequence_operation<
        FMaterializeHeader,
        FParsePositiveI64,
        FIsValidDbCapHandle,
    >(
        &mut self,
        raw_tx_db: Option<String>,
        raw_tx_handle: Option<String>,
        mut materialize_header: FMaterializeHeader,
        parse_positive_i64: FParsePositiveI64,
        is_valid_db_cap_handle: FIsValidDbCapHandle,
    ) -> Result<LasmSequenceExecTxPreparation, LasmInternalDbSequenceFailure>
    where
        FMaterializeHeader: FnMut(String) -> String,
        FParsePositiveI64: Fn(&str) -> Option<i64>,
        FIsValidDbCapHandle: Fn(i64) -> bool,
    {
        if raw_tx_db.is_some() && raw_tx_handle.is_some() {
            return Err(LasmInternalDbSequenceFailure::exec_tx_validation(
                "db.execTx must include either tx handle or db.tx(dbCap) source, not both",
            ));
        }

        let mut preparation = LasmSequenceExecTxPreparation::default();
        if let Some(raw_tx_db_value) = raw_tx_db {
            let tx_db_source_raw = materialize_header(raw_tx_db_value.clone());
            let Some(tx_db_source) = parse_positive_i64(tx_db_source_raw.trim()) else {
                return Err(LasmInternalDbSequenceFailure::exec_tx_validation(
                    "db.execTx requires transaction and query handles",
                ));
            };
            if !is_valid_db_cap_handle(tx_db_source) {
                return Err(LasmInternalDbSequenceFailure::exec_tx_validation(
                    "db.execTx requires db.tx(dbCap) with valid db capability handle",
                ));
            }
            preparation.retain_tx_for_sequence = true;
            if let Some(existing_tx_handle) = self.tracked_tx_for_source(tx_db_source) {
                preparation.tx_header = Some(existing_tx_handle.to_string());
                self.track_tx_handle(existing_tx_handle);
            } else {
                preparation.tx_db_header = Some(raw_tx_db_value);
                preparation.allocated_tx_source = Some(tx_db_source);
            }
            return Ok(preparation);
        }

        if let Some(raw_tx_handle_value) = raw_tx_handle {
            let tx_handle_raw = materialize_header(raw_tx_handle_value.clone());
            let Some(tx_handle) = parse_positive_i64(tx_handle_raw.trim()) else {
                return Err(LasmInternalDbSequenceFailure::exec_tx_validation(
                    "db.execTx requires valid tx handle",
                ));
            };
            preparation.retain_tx_for_sequence = true;
            preparation.tx_header = Some(raw_tx_handle_value);
            self.track_tx_handle(tx_handle);
        }

        Ok(preparation)
    }

    pub(crate) fn track_sequence_tx_runtime_result<FMaterializeHeader, FParsePositiveI64>(
        &mut self,
        raw_db_source: Option<String>,
        raw_tx_result: Option<String>,
        mut materialize_header: FMaterializeHeader,
        parse_positive_i64: FParsePositiveI64,
    ) -> Result<(), LasmInternalDbSequenceFailure>
    where
        FMaterializeHeader: FnMut(String) -> String,
        FParsePositiveI64: Fn(&str) -> Option<i64>,
    {
        let Some(raw_db_source) = raw_db_source else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime did not preserve db handle marker",
            ));
        };
        let db_source_raw = materialize_header(raw_db_source);
        let Some(db_source) = parse_positive_i64(db_source_raw.trim()) else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime failure",
            ));
        };
        let Some(tx_handle_raw) = raw_tx_result else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime did not publish transaction handle marker",
            ));
        };
        let Some(tx_handle) = parse_positive_i64(tx_handle_raw.as_str()) else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime failure",
            ));
        };
        self.track_source_tx_handle(db_source, tx_handle);
        Ok(())
    }

    pub(crate) fn track_sequence_allocated_exec_tx_runtime_result<FParsePositiveI64>(
        &mut self,
        tx_db_source: i64,
        raw_tx_result: Option<String>,
        parse_positive_i64: FParsePositiveI64,
    ) -> Result<(), LasmInternalDbSequenceFailure>
    where
        FParsePositiveI64: Fn(&str) -> Option<i64>,
    {
        let Some(tx_handle_raw) = raw_tx_result else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.execTx runtime did not publish transaction handle marker",
            ));
        };
        let Some(tx_handle) = parse_positive_i64(tx_handle_raw.as_str()) else {
            return Err(LasmInternalDbSequenceFailure::tx_internal(
                "db.tx runtime failure",
            ));
        };
        self.track_source_tx_handle(tx_db_source, tx_handle);
        Ok(())
    }
}
