use crate::{
    ensure_lasm_dynamic_db_records_sqlite_schema, LasmDbRecord,
    LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE,
};
use postgres::{Client as PostgresClient, NoTls};
use rusqlite::Connection;
use std::path::Path;

pub(crate) fn load_lasm_dynamic_db_records_from_sqlite(path: &Path) -> Vec<LasmDbRecord> {
    if !path.exists() {
        return Vec::new();
    }
    let connection = match Connection::open(path) {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic sqlite records store open failed at `{}`: {err}",
                path.display()
            );
            return Vec::new();
        }
    };
    if let Err(err) = ensure_lasm_dynamic_db_records_sqlite_schema(&connection) {
        eprintln!(
            "warning: LASM dynamic sqlite records schema check failed at `{}`: {err}",
            path.display()
        );
        return Vec::new();
    }
    let mut statement = match connection.prepare(
        "SELECT id, op, db, template, params, tx, affected_rows, created_at_ms \
         FROM lasm_db_records \
         ORDER BY id ASC",
    ) {
        Ok(statement) => statement,
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic sqlite records query prep failed at `{}`: {err}",
                path.display()
            );
            return Vec::new();
        }
    };
    let entries = match statement.query_map([], |row| {
        let id = row.get::<_, i64>(0)?;
        if id < 0 {
            return Err(rusqlite::Error::IntegralValueOutOfRange(0, id));
        }
        let affected_rows = row.get::<_, i64>(6)?;
        if affected_rows < 0 {
            return Err(rusqlite::Error::IntegralValueOutOfRange(6, affected_rows));
        }
        let created_at_ms = row.get::<_, i64>(7)?;
        if created_at_ms < 0 {
            return Err(rusqlite::Error::IntegralValueOutOfRange(7, created_at_ms));
        }
        Ok(LasmDbRecord {
            id: id as u64,
            op: row.get(1)?,
            db: row.get(2)?,
            template: row.get(3)?,
            params: row.get(4)?,
            tx: row.get(5)?,
            affected_rows: affected_rows as u64,
            created_at_ms: created_at_ms as u64,
        })
    }) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!(
                "warning: LASM dynamic sqlite records query failed at `{}`: {err}",
                path.display()
            );
            return Vec::new();
        }
    };
    let mut records = Vec::new();
    for (index, entry) in entries.enumerate() {
        match entry {
            Ok(record) => records.push(record),
            Err(err) => {
                eprintln!(
                    "warning: LASM dynamic sqlite records parse failed at `{}` row {}: {err}",
                    path.display(),
                    index + 1
                );
            }
        }
    }
    records
}

pub(crate) fn connect_lasm_dynamic_db_records_postgres(
    dsn: &str,
) -> Result<PostgresClient, String> {
    PostgresClient::connect(dsn, NoTls)
        .map_err(|err| format!("could not connect LASM dynamic postgres records store: {err}"))
}

pub(crate) fn ensure_lasm_dynamic_db_records_postgres_schema(
    client: &mut PostgresClient,
) -> Result<(), String> {
    let statement = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            id BIGINT PRIMARY KEY NOT NULL,
            op TEXT NOT NULL,
            db BIGINT NOT NULL,
            template TEXT NOT NULL,
            params TEXT NOT NULL,
            tx BIGINT NOT NULL,
            affected_rows BIGINT NOT NULL DEFAULT 0,
            created_at_ms BIGINT NOT NULL
        );
        ALTER TABLE {} ADD COLUMN IF NOT EXISTS affected_rows BIGINT NOT NULL DEFAULT 0;",
        LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE, LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
    );
    client
        .batch_execute(statement.as_str())
        .map_err(|err| format!("could not initialize LASM dynamic postgres records schema: {err}"))
}

pub(crate) fn load_lasm_dynamic_db_records_from_postgres(
    client: &mut PostgresClient,
) -> Result<Vec<LasmDbRecord>, String> {
    let query = format!(
        "SELECT id, op, db, template, params, tx, affected_rows, created_at_ms \
         FROM {} \
         ORDER BY id ASC",
        LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
    );
    let rows = client
        .query(query.as_str(), &[])
        .map_err(|err| format!("could not query LASM dynamic postgres records store: {err}"))?;
    let mut records = Vec::with_capacity(rows.len());
    for (index, row) in rows.into_iter().enumerate() {
        let id: i64 = row.get(0);
        let affected_rows: i64 = row.get(6);
        let created_at_ms: i64 = row.get(7);
        if id < 0 {
            return Err(format!(
                "could not load LASM dynamic postgres record row {}: id out of u64 range",
                index + 1
            ));
        }
        if affected_rows < 0 {
            return Err(format!(
                "could not load LASM dynamic postgres record row {}: affected_rows out of u64 range",
                index + 1
            ));
        }
        if created_at_ms < 0 {
            return Err(format!(
                "could not load LASM dynamic postgres record row {}: created_at_ms out of u64 range",
                index + 1
            ));
        }
        records.push(LasmDbRecord {
            id: id as u64,
            op: row.get(1),
            db: row.get(2),
            template: row.get(3),
            params: row.get(4),
            tx: row.get(5),
            affected_rows: affected_rows as u64,
            created_at_ms: created_at_ms as u64,
        });
    }
    Ok(records)
}
