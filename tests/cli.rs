use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::io::{Read, Write};
use std::net::TcpListener;

const EXPECTED: &str = "examples/fixtures/expected.sds.json";
const OBSERVED: &str = "examples/fixtures/observed.sds.json";

#[test]
fn documented_compare_returns_classified_json() {
    cargo_bin_cmd!("sds")
        .args([
            "compare", "--before", EXPECTED, "--after", OBSERVED, "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\": 5"))
        .stdout(predicate::str::contains("\"destructive\": true"))
        .stdout(predicate::str::contains("\"orm_invisible\": true"));
}

#[test]
fn markdown_review_never_contains_repair_sql() {
    cargo_bin_cmd!("sds")
        .args(["compare", "--before", EXPECTED, "--after", OBSERVED])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pause before repair"))
        .stdout(predicate::str::contains("no executable repair SQL"))
        .stdout(predicate::str::contains("DROP TABLE").not())
        .stdout(predicate::str::contains("ALTER TABLE").not());
}

#[test]
fn unsupported_database_urls_fail_with_the_documented_exit_code() {
    let output = tempfile::NamedTempFile::new().unwrap();
    cargo_bin_cmd!("sds")
        .args([
            "snapshot",
            "--url",
            "sqlite:///tmp/example.db",
            "--output",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unsupported database URL"));
}

#[test]
fn redaction_key_is_validated_before_a_database_connection() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("capture.sds.json");

    cargo_bin_cmd!("sds")
        .args([
            "snapshot",
            "--url",
            "postgresql://127.0.0.1:1/unreachable",
            "--output",
            output.to_str().unwrap(),
            "--redact-names",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "--redact-names requires --redaction-key",
        ))
        .stderr(predicate::str::contains("could not connect to PostgreSQL").not());

    assert!(
        !output.exists(),
        "invalid redaction input must not create a snapshot file"
    );
}

#[test]
fn help_states_the_read_only_safety_boundary() {
    cargo_bin_cmd!("sds")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("never reads row data"))
        .stdout(predicate::str::contains("repair SQL"));
}

#[test]
fn pro_check_verifies_license_and_applies_threshold() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 2048];
        let length = stream.read(&mut request).unwrap();
        let request = String::from_utf8_lossy(&request[..length]);
        assert!(
            request.contains("/api/v1/products/schema-drift-snapshot/verify?license=test-license")
        );
        let body = r#"{"valid":true,"reason":"ok","expires_at":null}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
    });
    let config = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("sds")
        .args([
            "check",
            "--before",
            EXPECTED,
            "--after",
            OBSERVED,
            "--fail-on",
            "high",
        ])
        .env("SDS_LICENSE", "test-license")
        .env("SDS_BILLING_BASE_URL", format!("http://{address}"))
        .env("XDG_CONFIG_HOME", config.path())
        .assert()
        .code(1)
        .stdout(predicate::str::contains("5 differences"));
    server.join().unwrap();
}
