use assert_cmd::cargo::cargo_bin_cmd;
use postgres::{Client, NoTls, error::SqlState};
use predicates::prelude::*;
use schema_drift_snapshot::model::{ObjectKind, Snapshot};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn reader_url(admin_url: &str, role: &str, password: &str) -> String {
    let (scheme, remainder) = admin_url
        .split_once("://")
        .expect("SDS_TEST_POSTGRES_ADMIN_URL must be a PostgreSQL URL");
    let address = remainder
        .rsplit_once('@')
        .map_or(remainder, |(_, address)| address);
    format!("{scheme}://{role}:{password}@{address}")
}

#[test]
fn postgres_non_owner_reader_captures_definition_only_view_drift() {
    let Ok(admin_url) = std::env::var("SDS_TEST_POSTGRES_ADMIN_URL") else {
        eprintln!(
            "real PostgreSQL regression skipped; set SDS_TEST_POSTGRES_ADMIN_URL to a disposable database whose role can create roles"
        );
        return;
    };

    let suffix = format!(
        "{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let schema = format!("sds_it_{suffix}");
    let reader = format!("sds_reader_{suffix}");
    let password = "sds-reader-test-only";
    let mut admin = Client::connect(&admin_url, NoTls).expect("connect as PostgreSQL test admin");

    admin
        .batch_execute(&format!(
            "CREATE ROLE {reader} LOGIN PASSWORD '{password}';
             CREATE SCHEMA {schema};
             CREATE TABLE {schema}.accounts (
                 id bigint PRIMARY KEY,
                 email text,
                 enabled boolean NOT NULL DEFAULT true
             );
             CREATE VIEW {schema}.active_accounts AS
                 SELECT id, email FROM {schema}.accounts WHERE enabled;
             GRANT USAGE ON SCHEMA {schema} TO {reader};
             GRANT SELECT ON ALL TABLES IN SCHEMA {schema} TO {reader};"
        ))
        .expect("create owner-managed schema and SELECT-only reader");

    let reader_url = reader_url(&admin_url, &reader, password);
    let mut reader_client = Client::connect(&reader_url, NoTls).expect("connect as reader");
    let insert_error = reader_client
        .execute(
            &format!("INSERT INTO {schema}.accounts (id) VALUES (1)"),
            &[],
        )
        .expect_err("reader must not be able to write row data");
    assert_eq!(insert_error.code(), Some(&SqlState::INSUFFICIENT_PRIVILEGE));
    drop(reader_client);

    let directory = tempfile::tempdir().unwrap();
    let before_path = directory.path().join("before.sds.json");
    let after_path = directory.path().join("after.sds.json");

    cargo_bin_cmd!("sds")
        .args([
            "snapshot",
            "--url",
            &reader_url,
            "--schema",
            &schema,
            "--output",
            before_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    admin
        .batch_execute(&format!(
            "CREATE OR REPLACE VIEW {schema}.active_accounts AS
             SELECT id, email FROM {schema}.accounts
             WHERE enabled AND email IS NOT NULL;"
        ))
        .expect("replace view as its owner");

    cargo_bin_cmd!("sds")
        .args([
            "snapshot",
            "--url",
            &reader_url,
            "--schema",
            &schema,
            "--output",
            after_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    for path in [&before_path, &after_path] {
        let snapshot = Snapshot::read(path).unwrap();
        let view = snapshot
            .objects
            .iter()
            .find(|object| object.kind == ObjectKind::View)
            .expect("snapshot includes the view");
        assert!(view.details["definition"].as_str().is_some());
        assert!(!fs::read_to_string(path).unwrap().contains(password));
    }

    cargo_bin_cmd!("sds")
        .args([
            "compare",
            "--before",
            before_path.to_str().unwrap(),
            "--after",
            after_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\": 1"))
        .stdout(predicate::str::contains("\"change\": \"modified\""))
        .stdout(predicate::str::contains("\"object_kind\": \"view\""))
        .stdout(predicate::str::contains("\"orm_invisible\": true"));

    admin
        .batch_execute(&format!(
            "DROP SCHEMA {schema} CASCADE; DROP ROLE {reader};"
        ))
        .expect("clean PostgreSQL integration fixture");
}
