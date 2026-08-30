use assert_cmd::cargo::cargo_bin_cmd;
use mysql::{Opts, Pool, prelude::Queryable};
use predicates::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn reader_url(admin_url: &str, user: &str, password: &str) -> String {
    let (scheme, remainder) = admin_url
        .split_once("://")
        .expect("SDS_TEST_MYSQL_ADMIN_URL must be a MySQL URL");
    let address = remainder
        .rsplit_once('@')
        .map_or(remainder, |(_, address)| address);
    format!("{scheme}://{user}:{password}@{address}")
}

#[test]
fn mysql_reader_requires_view_metadata_then_detects_definition_only_drift() {
    let Ok(admin_url) = std::env::var("SDS_TEST_MYSQL_ADMIN_URL") else {
        eprintln!(
            "real MySQL/MariaDB regression skipped; set SDS_TEST_MYSQL_ADMIN_URL to a disposable database whose role can create users"
        );
        return;
    };

    let opts = Opts::from_url(&admin_url).expect("parse MySQL test admin URL");
    let database = opts
        .get_db_name()
        .expect("SDS_TEST_MYSQL_ADMIN_URL must select a database")
        .to_owned();
    assert!(
        database
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_'),
        "test database name must contain only letters, numbers, and underscores"
    );
    let suffix = format!(
        "{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let table = format!("sds_accounts_{suffix}");
    let view = format!("sds_active_{suffix}");
    let reader = format!("sds_reader_{suffix}");
    let password = "sds-reader-test-only";
    let pool = Pool::new(opts).expect("create MySQL test admin pool");
    let mut admin = pool.get_conn().expect("connect as MySQL test admin");

    admin
        .query_drop(format!(
            "CREATE USER '{reader}'@'127.0.0.1' IDENTIFIED BY '{password}'"
        ))
        .expect("create MySQL reader");
    admin
        .query_drop(format!(
            "CREATE TABLE `{database}`.`{table}` (
                id BIGINT PRIMARY KEY,
                email TEXT,
                enabled BOOLEAN NOT NULL DEFAULT TRUE
            )"
        ))
        .expect("create MySQL test table");
    admin
        .query_drop(format!(
            "CREATE VIEW `{database}`.`{view}` AS
             SELECT id, email FROM `{database}`.`{table}` WHERE enabled"
        ))
        .expect("create owner-managed MySQL view");
    admin
        .query_drop(format!(
            "GRANT SELECT ON `{database}`.* TO '{reader}'@'127.0.0.1'"
        ))
        .expect("grant SELECT-only data access");

    let reader_url = reader_url(&admin_url, &reader, password);
    let directory = tempfile::tempdir().unwrap();
    let incomplete_path = directory.path().join("incomplete.sds.json");
    cargo_bin_cmd!("sds")
        .args([
            "snapshot",
            "--url",
            &reader_url,
            "--output",
            incomplete_path.to_str().unwrap(),
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("incomplete catalog capture"))
        .stderr(predicate::str::contains("grant SHOW VIEW"));
    assert!(!incomplete_path.exists());

    admin
        .query_drop(format!(
            "GRANT SHOW VIEW ON `{database}`.* TO '{reader}'@'127.0.0.1'"
        ))
        .expect("grant read-only view metadata access");

    let before_path = directory.path().join("before.sds.json");
    let after_path = directory.path().join("after.sds.json");
    cargo_bin_cmd!("sds")
        .args([
            "snapshot",
            "--url",
            &reader_url,
            "--output",
            before_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    admin
        .query_drop(format!(
            "CREATE OR REPLACE VIEW `{database}`.`{view}` AS
             SELECT id, email FROM `{database}`.`{table}`
             WHERE enabled AND email IS NOT NULL"
        ))
        .expect("replace MySQL view as its owner");

    cargo_bin_cmd!("sds")
        .args([
            "snapshot",
            "--url",
            &reader_url,
            "--output",
            after_path.to_str().unwrap(),
        ])
        .assert()
        .success();

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

    let reader_pool = Pool::new(Opts::from_url(&reader_url).unwrap()).unwrap();
    let mut reader_connection = reader_pool.get_conn().unwrap();
    assert!(
        reader_connection
            .query_drop(format!(
                "INSERT INTO `{database}`.`{table}` (id) VALUES (1)"
            ))
            .is_err(),
        "metadata reader must not be able to write row data"
    );
    drop(reader_connection);
    drop(reader_pool);

    admin
        .query_drop(format!("DROP VIEW `{database}`.`{view}`"))
        .expect("drop MySQL test view");
    admin
        .query_drop(format!("DROP TABLE `{database}`.`{table}`"))
        .expect("drop MySQL test table");
    admin
        .query_drop(format!("DROP USER '{reader}'@'127.0.0.1'"))
        .expect("drop MySQL test reader");
}
