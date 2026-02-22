use crate::lasm_db_runtime_common::normalize_lasm_db_params;
use crate::LasmDbRecord;
use std::fs::{self, OpenOptions};
use std::io::Write;
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

pub(crate) fn persist_lasm_dynamic_db_record_append_to_records_log(
    path: Option<&Path>,
    record: &LasmDbRecord,
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
    let line = serde_json::to_string(&lasm_db_record_to_json(record))
        .map_err(|err| format!("could not serialize LASM dynamic records store: {err}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| {
            format!(
                "could not open LASM dynamic records store `{}` for append: {err}",
                path.display()
            )
        })?;
    file.write_all(line.as_bytes()).map_err(|err| {
        format!(
            "could not append LASM dynamic record {} to `{}`: {err}",
            record.id,
            path.display()
        )
    })?;
    file.write_all(b"\n").map_err(|err| {
        format!(
            "could not append newline to LASM dynamic records store `{}`: {err}",
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
        params: normalize_lasm_db_params(value.get("params")?.as_str()?),
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

#[cfg(test)]
mod tests {
    use super::load_lasm_dynamic_db_records_from_disk;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn load_records_log_normalizes_json_params_for_signature_stability() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic for tests")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("sec4-records-log-normalize-{suffix}"));
        fs::create_dir_all(&base).expect("temp records-log directory should be created");
        let path = base.join("records.log");
        let line = serde_json::json!({
            "id": 1,
            "op": "exec",
            "db": 1,
            "template": "SELECT $1::int",
            "params": "{\"b\":2,\"a\":1}",
            "tx": 0,
            "affected_rows": 0,
            "created_at_ms": 1
        });
        fs::write(&path, format!("{line}\n")).expect("records.log fixture should be written");

        let records = load_lasm_dynamic_db_records_from_disk(path.as_path());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].params, "{\"a\":1,\"b\":2}");

        fs::remove_dir_all(&base).expect("temp records-log directory should be removed");
    }
}
