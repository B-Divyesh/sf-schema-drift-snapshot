use sha2::{Digest, Sha256};

use crate::model::{SchemaObject, Snapshot};

fn token(key: &str, scope: &str, value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(key.as_bytes());
    digest.update(b"\0");
    digest.update(scope.as_bytes());
    digest.update(b"\0");
    digest.update(value.as_bytes());
    format!("{}_{}", scope, &hex::encode(digest.finalize())[..10])
}

pub fn redact(mut snapshot: Snapshot, key: &str) -> Snapshot {
    for object in &mut snapshot.objects {
        redact_object(object, key);
    }
    let mut digest = Sha256::new();
    digest.update(key.as_bytes());
    snapshot.redacted = true;
    snapshot.redaction_key_id = Some(hex::encode(digest.finalize())[..12].to_owned());
    snapshot.source = "redacted".to_owned();
    snapshot.sort();
    snapshot
}

fn redact_object(object: &mut SchemaObject, key: &str) {
    object.schema = token(key, "schema", &object.schema);
    if let Some(table) = &object.table {
        object.table = Some(token(key, "table", table));
    }
    let scope = object.kind.to_string().replace(' ', "_");
    object.name = token(key, &scope, &object.name);

    for field in ["columns", "referenced_columns"] {
        if let Some(values) = object.details.get_mut(field).and_then(|v| v.as_array_mut()) {
            for value in values {
                if let Some(name) = value.as_str() {
                    *value = serde_json::Value::String(token(key, "column", name));
                }
            }
        }
    }
    if let Some(value) = object.details.get_mut("referenced_schema")
        && let Some(name) = value.as_str()
    {
        *value = serde_json::Value::String(token(key, "schema", name));
    }
    if let Some(value) = object.details.get_mut("referenced_table")
        && let Some(name) = value.as_str()
    {
        *value = serde_json::Value::String(token(key, "table", name));
    }
    for field in ["definition", "default", "data_type", "native_type"] {
        if let Some(value) = object.details.get_mut(field)
            && !value.is_null()
        {
            let raw = value.to_string();
            *value = serde_json::Value::String(token(key, "detail", &raw));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::model::{Dialect, ObjectKind, SCHEMA_VERSION};

    #[test]
    fn redaction_hides_names_and_definition_content_deterministically() {
        let input = Snapshot {
            schema_version: SCHEMA_VERSION,
            dialect: Dialect::PostgreSql,
            captured_at: "now".to_owned(),
            source: "test".to_owned(),
            redacted: false,
            redaction_key_id: None,
            objects: vec![SchemaObject {
                kind: ObjectKind::Index,
                schema: "private".to_owned(),
                table: Some("customers".to_owned()),
                name: "customers_email_idx".to_owned(),
                details: BTreeMap::from([(
                    "definition".to_owned(),
                    json!("CREATE INDEX customers_email_idx ON private.customers(email)"),
                )]),
            }],
        };
        let first = redact(input.clone(), "shared-key");
        let second = redact(input, "shared-key");
        let serialized = serde_json::to_string(&first).unwrap();
        assert_eq!(first, second);
        assert!(!serialized.contains("customers"));
        assert!(!serialized.contains("private"));
        assert!(!serialized.contains("email"));
    }

    #[test]
    fn redaction_hides_a_captured_view_definition() {
        let input = Snapshot {
            schema_version: SCHEMA_VERSION,
            dialect: Dialect::MySql,
            captured_at: "now".to_owned(),
            source: "catalog test".to_owned(),
            redacted: false,
            redaction_key_id: None,
            objects: vec![SchemaObject {
                kind: ObjectKind::View,
                schema: "private".to_owned(),
                table: None,
                name: "active_customers".to_owned(),
                details: BTreeMap::from([(
                    "definition".to_owned(),
                    json!("SELECT email FROM private.customers WHERE enabled = true"),
                )]),
            }],
        };

        let output = redact(input, "shared-key");
        let serialized = serde_json::to_string(&output).unwrap();
        assert!(!serialized.contains("active_customers"));
        assert!(!serialized.contains("private.customers"));
        assert!(!serialized.contains("enabled"));
        assert!(
            output.objects[0].details["definition"]
                .as_str()
                .unwrap()
                .starts_with("detail_")
        );
    }
}
