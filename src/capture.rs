use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use mysql::{Opts, Pool, prelude::Queryable};
use native_tls::TlsConnector;
use postgres::Client;
use postgres_native_tls::MakeTlsConnector;
use serde_json::{Value, json};

use crate::model::{Dialect, ObjectKind, SCHEMA_VERSION, SchemaObject, Snapshot};

/// Detect a supported database dialect without opening a connection.
pub fn dialect_for_url(url: &str) -> Result<Dialect> {
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Ok(Dialect::PostgreSql)
    } else if url.starts_with("mysql://") {
        Ok(Dialect::MySql)
    } else {
        bail!("unsupported database URL; expected postgres://, postgresql://, or mysql://")
    }
}

/// Capture catalog metadata through a connection forced into read-only mode.
pub fn capture(url: &str, schemas: &[String]) -> Result<Snapshot> {
    let dialect = dialect_for_url(url)?;
    let objects = match dialect {
        Dialect::PostgreSql => capture_postgres(url, schemas)?,
        Dialect::MySql => capture_mysql(url, schemas)?,
    };
    let mut snapshot = Snapshot {
        schema_version: SCHEMA_VERSION,
        dialect,
        captured_at: Utc::now().to_rfc3339(),
        source: "database catalog".to_owned(),
        redacted: false,
        redaction_key_id: None,
        objects,
    };
    snapshot.sort();
    Ok(snapshot)
}

fn keep_schema(schema: &str, allow: &BTreeSet<&str>) -> bool {
    allow.is_empty() || allow.contains(schema)
}

fn details(entries: impl IntoIterator<Item = (&'static str, Value)>) -> BTreeMap<String, Value> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

const POSTGRES_CATALOG_OBJECTS_QUERY: &str = "SELECT tables.table_schema, tables.table_name, tables.table_type, views.view_definition \
     FROM information_schema.tables AS tables \
     LEFT JOIN information_schema.views AS views \
       ON views.table_schema = tables.table_schema AND views.table_name = tables.table_name \
     WHERE tables.table_schema NOT IN ('pg_catalog', 'information_schema') \
     ORDER BY tables.table_schema, tables.table_name";

const MYSQL_CATALOG_OBJECTS_QUERY: &str = "SELECT tables.TABLE_SCHEMA, tables.TABLE_NAME, tables.TABLE_TYPE, views.VIEW_DEFINITION \
     FROM information_schema.TABLES AS tables \
     LEFT JOIN information_schema.VIEWS AS views \
       ON views.TABLE_SCHEMA = tables.TABLE_SCHEMA AND views.TABLE_NAME = tables.TABLE_NAME \
     WHERE tables.TABLE_SCHEMA = DATABASE() ORDER BY tables.TABLE_SCHEMA, tables.TABLE_NAME";

/// Convert a row from either dialect's table/view catalog query into the
/// portable snapshot representation. Views deliberately retain their query
/// definition: their projected columns alone do not describe joins,
/// predicates, or security-sensitive expressions.
fn catalog_object(
    schema: String,
    name: String,
    table_type: String,
    view_definition: Option<String>,
) -> SchemaObject {
    let kind = if table_type.contains("VIEW") {
        ObjectKind::View
    } else {
        ObjectKind::Table
    };
    let details = if kind == ObjectKind::View {
        details([
            ("table_type", json!(table_type)),
            ("definition", json!(view_definition)),
        ])
    } else {
        details([("table_type", json!(table_type))])
    };
    SchemaObject {
        kind,
        schema,
        table: None,
        name,
        details,
    }
}

fn capture_postgres(url: &str, schemas: &[String]) -> Result<Vec<SchemaObject>> {
    let connector = TlsConnector::builder()
        .build()
        .context("could not initialize PostgreSQL TLS")?;
    let tls = MakeTlsConnector::new(connector);
    let mut client = Client::connect(url, tls)
        .context("could not connect to PostgreSQL; check the URL, TLS mode, and read-only role")?;
    client
        .batch_execute("SET SESSION CHARACTERISTICS AS TRANSACTION READ ONLY")
        .context("database role could not enter read-only mode")?;

    let allow = schemas.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut objects = Vec::new();

    let table_rows = client.query(POSTGRES_CATALOG_OBJECTS_QUERY, &[])?;
    for row in table_rows {
        let schema: String = row.get(0);
        if !keep_schema(&schema, &allow) {
            continue;
        }
        objects.push(catalog_object(schema, row.get(1), row.get(2), row.get(3)));
    }

    let column_rows = client.query(
        "SELECT table_schema, table_name, column_name, ordinal_position, is_nullable, \
                data_type, udt_name, column_default \
         FROM information_schema.columns \
         WHERE table_schema NOT IN ('pg_catalog', 'information_schema') \
         ORDER BY table_schema, table_name, ordinal_position",
        &[],
    )?;
    for row in column_rows {
        let schema: String = row.get(0);
        if !keep_schema(&schema, &allow) {
            continue;
        }
        let default: Option<String> = row.get(7);
        objects.push(SchemaObject {
            kind: ObjectKind::Column,
            schema,
            table: Some(row.get(1)),
            name: row.get(2),
            details: details([
                ("ordinal", json!(row.get::<_, i32>(3))),
                ("nullable", json!(row.get::<_, String>(4) == "YES")),
                ("data_type", json!(row.get::<_, String>(5))),
                ("native_type", json!(row.get::<_, String>(6))),
                ("default", json!(default)),
            ]),
        });
    }

    let index_rows = client.query(
        "SELECT schemaname, tablename, indexname, indexdef \
         FROM pg_indexes WHERE schemaname NOT IN ('pg_catalog', 'information_schema') \
         ORDER BY schemaname, tablename, indexname",
        &[],
    )?;
    for row in index_rows {
        let schema: String = row.get(0);
        if !keep_schema(&schema, &allow) {
            continue;
        }
        let definition: String = row.get(3);
        objects.push(SchemaObject {
            kind: ObjectKind::Index,
            schema,
            table: Some(row.get(1)),
            name: row.get(2),
            details: details([
                ("unique", json!(definition.contains("CREATE UNIQUE INDEX"))),
                ("definition", json!(definition)),
            ]),
        });
    }

    let foreign_key_rows = client.query(
        "SELECT ns.nspname, cls.relname, con.conname, pg_get_constraintdef(con.oid, true) \
         FROM pg_constraint con \
         JOIN pg_class cls ON cls.oid = con.conrelid \
         JOIN pg_namespace ns ON ns.oid = cls.relnamespace \
         WHERE con.contype = 'f' AND ns.nspname NOT IN ('pg_catalog', 'information_schema') \
         ORDER BY ns.nspname, cls.relname, con.conname",
        &[],
    )?;
    for row in foreign_key_rows {
        let schema: String = row.get(0);
        if !keep_schema(&schema, &allow) {
            continue;
        }
        objects.push(SchemaObject {
            kind: ObjectKind::ForeignKey,
            schema,
            table: Some(row.get(1)),
            name: row.get(2),
            details: details([("definition", json!(row.get::<_, String>(3)))]),
        });
    }
    Ok(objects)
}

fn capture_mysql(url: &str, schemas: &[String]) -> Result<Vec<SchemaObject>> {
    let opts = Opts::from_url(url).context("invalid MySQL URL")?;
    let pool = Pool::new(opts).context("could not create MySQL connection pool")?;
    let mut conn = pool
        .get_conn()
        .context("could not connect to MySQL; check the URL, TLS mode, and read-only role")?;
    conn.query_drop("SET SESSION TRANSACTION READ ONLY")
        .context("database role could not enter read-only mode")?;
    let allow = schemas.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut objects = Vec::new();

    let tables: Vec<(String, String, String, Option<String>)> =
        conn.query(MYSQL_CATALOG_OBJECTS_QUERY)?;
    for (schema, name, table_type, view_definition) in tables {
        if !keep_schema(&schema, &allow) {
            continue;
        }
        objects.push(catalog_object(schema, name, table_type, view_definition));
    }

    type MySqlColumn = (String, String, String, u32, String, String, Option<String>);
    let columns: Vec<MySqlColumn> = conn.query(
        "SELECT TABLE_SCHEMA, TABLE_NAME, COLUMN_NAME, ORDINAL_POSITION, IS_NULLABLE, \
                COLUMN_TYPE, COLUMN_DEFAULT FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() ORDER BY TABLE_SCHEMA, TABLE_NAME, ORDINAL_POSITION",
    )?;
    for (schema, table, name, ordinal, nullable, data_type, default) in columns {
        if !keep_schema(&schema, &allow) {
            continue;
        }
        objects.push(SchemaObject {
            kind: ObjectKind::Column,
            schema,
            table: Some(table),
            name,
            details: details([
                ("ordinal", json!(ordinal)),
                ("nullable", json!(nullable == "YES")),
                ("data_type", json!(data_type)),
                ("default", json!(default)),
            ]),
        });
    }

    type IndexRow = (String, String, String, u64, String, u64);
    let index_rows: Vec<IndexRow> = conn.query(
        "SELECT TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, NON_UNIQUE, COLUMN_NAME, SEQ_IN_INDEX \
         FROM information_schema.STATISTICS WHERE TABLE_SCHEMA = DATABASE() \
         ORDER BY TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX",
    )?;
    let mut indexes: BTreeMap<(String, String, String), (bool, Vec<String>)> = BTreeMap::new();
    for (schema, table, name, non_unique, column, _) in index_rows {
        if keep_schema(&schema, &allow) {
            let entry = indexes
                .entry((schema, table, name))
                .or_insert((non_unique == 0, Vec::new()));
            entry.1.push(column);
        }
    }
    for ((schema, table, name), (unique, columns)) in indexes {
        objects.push(SchemaObject {
            kind: ObjectKind::Index,
            schema,
            table: Some(table),
            name,
            details: details([("unique", json!(unique)), ("columns", json!(columns))]),
        });
    }

    type ForeignKeyRow = (String, String, String, String, String, String, String, u64);
    let foreign_key_rows: Vec<ForeignKeyRow> = conn.query(
        "SELECT k.TABLE_SCHEMA, k.TABLE_NAME, k.CONSTRAINT_NAME, k.COLUMN_NAME, \
                k.REFERENCED_TABLE_SCHEMA, k.REFERENCED_TABLE_NAME, k.REFERENCED_COLUMN_NAME, \
                k.ORDINAL_POSITION FROM information_schema.KEY_COLUMN_USAGE k \
         WHERE k.TABLE_SCHEMA = DATABASE() AND k.REFERENCED_TABLE_NAME IS NOT NULL \
         ORDER BY k.TABLE_SCHEMA, k.TABLE_NAME, k.CONSTRAINT_NAME, k.ORDINAL_POSITION",
    )?;
    type ForeignKeyValue = (String, String, Vec<String>, Vec<String>);
    let mut foreign_keys: BTreeMap<(String, String, String), ForeignKeyValue> = BTreeMap::new();
    for (schema, table, name, column, ref_schema, ref_table, ref_column, _) in foreign_key_rows {
        if keep_schema(&schema, &allow) {
            let entry = foreign_keys.entry((schema, table, name)).or_insert((
                ref_schema,
                ref_table,
                Vec::new(),
                Vec::new(),
            ));
            entry.2.push(column);
            entry.3.push(ref_column);
        }
    }
    for ((schema, table, name), (ref_schema, ref_table, columns, ref_columns)) in foreign_keys {
        objects.push(SchemaObject {
            kind: ObjectKind::ForeignKey,
            schema,
            table: Some(table),
            name,
            details: details([
                ("columns", json!(columns)),
                ("referenced_schema", json!(ref_schema)),
                ("referenced_table", json!(ref_table)),
                ("referenced_columns", json!(ref_columns)),
            ]),
        });
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        diff::{ChangeKind, compare},
        model::{SCHEMA_VERSION, Snapshot},
    };

    fn view_snapshot(dialect: Dialect, definition: &str) -> Snapshot {
        Snapshot {
            schema_version: SCHEMA_VERSION,
            dialect,
            captured_at: "2026-08-28T00:00:00Z".to_owned(),
            source: "catalog test".to_owned(),
            redacted: false,
            redaction_key_id: None,
            objects: vec![catalog_object(
                "app".to_owned(),
                "active_accounts".to_owned(),
                "VIEW".to_owned(),
                Some(definition.to_owned()),
            )],
        }
    }

    #[test]
    fn detects_documented_database_urls() {
        assert_eq!(
            dialect_for_url("postgresql://localhost/db").unwrap(),
            Dialect::PostgreSql
        );
        assert_eq!(
            dialect_for_url("postgres://localhost/db").unwrap(),
            Dialect::PostgreSql
        );
        assert_eq!(
            dialect_for_url("mysql://localhost/db").unwrap(),
            Dialect::MySql
        );
        assert!(dialect_for_url("sqlite:///tmp/db").is_err());
    }

    #[test]
    fn postgres_definition_only_view_change_is_captured_and_classified() {
        assert!(POSTGRES_CATALOG_OBJECTS_QUERY.contains("information_schema.views"));
        assert!(POSTGRES_CATALOG_OBJECTS_QUERY.contains("views.view_definition"));
        let before = view_snapshot(
            Dialect::PostgreSql,
            " SELECT accounts.id FROM accounts WHERE accounts.enabled ",
        );
        let after = view_snapshot(
            Dialect::PostgreSql,
            " SELECT accounts.id FROM accounts WHERE accounts.enabled AND accounts.verified ",
        );

        let review = compare(&before, &after).unwrap();
        assert_eq!(review.summary.total, 1);
        assert_eq!(review.changes[0].change, ChangeKind::Modified);
        assert_eq!(review.changes[0].object_kind, ObjectKind::View);
        assert!(review.changes[0].orm_invisible);
        assert_eq!(
            review.changes[0].after.as_ref().unwrap().details["definition"],
            json!(
                " SELECT accounts.id FROM accounts WHERE accounts.enabled AND accounts.verified "
            )
        );
    }

    #[test]
    fn mysql_definition_only_view_change_is_captured_and_classified() {
        assert!(MYSQL_CATALOG_OBJECTS_QUERY.contains("information_schema.VIEWS"));
        assert!(MYSQL_CATALOG_OBJECTS_QUERY.contains("views.VIEW_DEFINITION"));
        let before = view_snapshot(
            Dialect::MySql,
            "select `accounts`.`id` from `accounts` where (`accounts`.`enabled` = 1)",
        );
        let after = view_snapshot(
            Dialect::MySql,
            "select `accounts`.`id` from `accounts` where ((`accounts`.`enabled` = 1) and (`accounts`.`verified` = 1))",
        );

        let review = compare(&before, &after).unwrap();
        assert_eq!(review.summary.total, 1);
        assert_eq!(review.changes[0].change, ChangeKind::Modified);
        assert_eq!(review.changes[0].object_kind, ObjectKind::View);
        assert!(review.changes[0].orm_invisible);
    }
}
