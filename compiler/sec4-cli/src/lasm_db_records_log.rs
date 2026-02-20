use crate::LasmDbRecord;
use std::fs;
use std::path::Path;

pub(crate) fn load_lasm_dynamic_db_records_from_disk(path: &Path) -> Vec<LasmDbRecord> {
    let raw = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic records store load failed at `{}`: {err}",
                path.display()
            );
            return Vec::new();
        }
    };
    let mut records = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        let candidate = line.trim();
        if candidate.is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(candidate) {
            Ok(value) => {
                if let Some(record) = lasm_db_record_from_json(&value) {
                    records.push(record);
                } else {
                    eprintln!(
                        "warning: LASM dynamic records store parse failed at `{}` line {}: invalid record shape",
                        path.display(),
                        index + 1
                    );
                }
            }
            Err(err) => {
                eprintln!(
                    "warning: LASM dynamic records store parse failed at `{}` line {}: {err}",
                    path.display(),
                    index + 1
                );
            }
        }
    }
    records.sort_by_key(|record| record.id);
    records
}

pub(crate) fn persist_lasm_dynamic_db_records_to_records_log(
    path: Option<&Path>,
    records: &[LasmDbRecord],
) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "could not create LASM dynamic records store directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    let mut ordered = records.to_vec();
    ordered.sort_by_key(|record| record.id);
    let mut raw = String::new();
    for record in ordered {
        let line = serde_json::to_string(&lasm_db_record_to_json(&record))
            .map_err(|err| format!("could not serialize LASM dynamic records store: {err}"))?;
        raw.push_str(line.as_str());
        raw.push('\n');
    }
    fs::write(path, raw.as_bytes()).map_err(|err| {
        format!(
            "could not write LASM dynamic records store `{}`: {err}",
            path.display()
        )
    })?;
    Ok(())
}

fn lasm_db_record_from_json(value: &serde_json::Value) -> Option<LasmDbRecord> {
    Some(LasmDbRecord {
        id: value.get("id")?.as_u64()?,
        op: value.get("op")?.as_str()?.to_string(),
        db: value.get("db")?.as_i64()?,
        template: value.get("template")?.as_str()?.to_string(),
        params: value.get("params")?.as_str()?.to_string(),
        tx: value.get("tx")?.as_i64()?,
        affected_rows: value
            .get("affected_rows")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        created_at_ms: value.get("created_at_ms")?.as_u64()?,
    })
}

pub(crate) fn lasm_db_record_to_json(record: &LasmDbRecord) -> serde_json::Value {
    serde_json::json!({
        "id": record.id,
        "op": record.op,
        "db": record.db,
        "template": record.template,
        "params": record.params,
        "tx": record.tx,
        "affected_rows": record.affected_rows,
        "created_at_ms": record.created_at_ms,
    })
}
