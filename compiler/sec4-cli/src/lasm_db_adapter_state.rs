use crate::{
    lasm_dynamic_postgres_client_mut, reconnect_lasm_dynamic_postgres_client, LasmDbRecord,
    LasmDynamicResponseState, LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE,
};
use postgres::{Client as PostgresClient, NoTls};
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

pub(crate) fn ensure_lasm_dynamic_db_records_sqlite_schema(
    connection: &Connection,
) -> rusqlite::Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS lasm_db_records (
            id INTEGER PRIMARY KEY NOT NULL,
            op TEXT NOT NULL,
            db INTEGER NOT NULL,
            template TEXT NOT NULL,
            params TEXT NOT NULL,
            tx INTEGER NOT NULL,
            affected_rows INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL
        );",
    )?;
    let mut statement = connection.prepare("PRAGMA table_info(lasm_db_records)")?;
    let mut rows = statement.query([])?;
    let mut has_affected_rows = false;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "affected_rows" {
            has_affected_rows = true;
            break;
        }
    }
    drop(rows);
    drop(statement);
    if !has_affected_rows {
        connection.execute_batch(
            "ALTER TABLE lasm_db_records ADD COLUMN affected_rows INTEGER NOT NULL DEFAULT 0;",
        )?;
    }
    Ok(())
}

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

pub(crate) fn persist_lasm_dynamic_db_records_to_sqlite(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    let Some(path) = state.db_records_sqlite_store_path.as_ref() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "could not create LASM dynamic sqlite records store directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    let mut connection = Connection::open(path).map_err(|err| {
        format!(
            "could not open LASM dynamic sqlite records store `{}`: {err}",
            path.display()
        )
    })?;
    ensure_lasm_dynamic_db_records_sqlite_schema(&connection).map_err(|err| {
        format!(
            "could not initialize LASM dynamic sqlite records schema `{}`: {err}",
            path.display()
        )
    })?;
    let tx = connection.transaction().map_err(|err| {
        format!(
            "could not start LASM dynamic sqlite records transaction `{}`: {err}",
            path.display()
        )
    })?;
    tx.execute("DELETE FROM lasm_db_records", [])
        .map_err(|err| {
            format!(
                "could not clear LASM dynamic sqlite records store `{}`: {err}",
                path.display()
            )
        })?;
    let mut ordered = state.db_records.clone();
    ordered.sort_by_key(|record| record.id);
    let mut statement = tx
        .prepare(
            "INSERT INTO lasm_db_records \
             (id, op, db, template, params, tx, affected_rows, created_at_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .map_err(|err| {
            format!(
                "could not prepare LASM dynamic sqlite records insert `{}`: {err}",
                path.display()
            )
        })?;
    for record in &ordered {
        let id = i64::try_from(record.id).map_err(|_| {
            format!(
                "could not persist LASM dynamic sqlite record id {}: out of i64 range",
                record.id
            )
        })?;
        let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
            format!(
                "could not persist LASM dynamic sqlite record timestamp {}: out of i64 range",
                record.created_at_ms
            )
        })?;
        let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
            format!(
                "could not persist LASM dynamic sqlite record affected_rows {}: out of i64 range",
                record.affected_rows
            )
        })?;
        statement
            .execute(params![
                id,
                record.op.as_str(),
                record.db,
                record.template.as_str(),
                record.params.as_str(),
                record.tx,
                affected_rows,
                created_at_ms
            ])
            .map_err(|err| {
                format!(
                    "could not insert LASM dynamic sqlite record {} into `{}`: {err}",
                    record.id,
                    path.display()
                )
            })?;
    }
    drop(statement);
    tx.commit().map_err(|err| {
        format!(
            "could not commit LASM dynamic sqlite records store `{}`: {err}",
            path.display()
        )
    })?;
    Ok(())
}

pub(crate) fn persist_lasm_dynamic_db_records_to_postgres(
    state: &mut LasmDynamicResponseState,
) -> Result<(), String> {
    let mut ordered = state.db_records.clone();
    ordered.sort_by_key(|record| record.id);

    let run_sync = |client: &mut PostgresClient| -> Result<(), String> {
        ensure_lasm_dynamic_db_records_postgres_schema(client)?;
        let mut tx = client.transaction().map_err(|err| {
            format!("could not start LASM dynamic postgres records transaction: {err}")
        })?;
        let delete_statement = format!("DELETE FROM {}", LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE);
        tx.execute(delete_statement.as_str(), &[])
            .map_err(|err| format!("could not clear LASM dynamic postgres records store: {err}"))?;
        let insert_statement = format!(
            "INSERT INTO {} (id, op, db, template, params, tx, affected_rows, created_at_ms) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
        );
        for record in &ordered {
            let id = i64::try_from(record.id).map_err(|_| {
                format!(
                    "could not persist LASM dynamic postgres record id {}: out of i64 range",
                    record.id
                )
            })?;
            let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
                format!(
                    "could not persist LASM dynamic postgres record timestamp {}: out of i64 range",
                    record.created_at_ms
                )
            })?;
            let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
                format!(
                    "could not persist LASM dynamic postgres record affected_rows {}: out of i64 range",
                    record.affected_rows
                )
            })?;
            tx.execute(
                insert_statement.as_str(),
                &[
                    &id,
                    &record.op,
                    &record.db,
                    &record.template,
                    &record.params,
                    &record.tx,
                    &affected_rows,
                    &created_at_ms,
                ],
            )
            .map_err(|err| {
                format!(
                    "could not insert LASM dynamic postgres record {}: {err}",
                    record.id
                )
            })?;
        }
        tx.commit().map_err(|err| {
            format!("could not commit LASM dynamic postgres records store: {err}")
        })?;
        Ok(())
    };

    let initial = {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        run_sync(client)
    };
    match initial {
        Ok(()) => {}
        Err(message) if message.contains("closed") || message.contains("broken pipe") => {
            reconnect_lasm_dynamic_postgres_client(state)?;
            let client = lasm_dynamic_postgres_client_mut(state)?;
            run_sync(client)?;
        }
        Err(message) => return Err(message),
    }
    Ok(())
}
