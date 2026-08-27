use std::collections::BTreeMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::model::{ObjectKind, SchemaObject, Snapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Risk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Added,
    Removed,
    Modified,
}

impl std::fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Modified => write!(f, "modified"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub change: ChangeKind,
    pub object_kind: ObjectKind,
    pub object: String,
    pub risk: Risk,
    pub destructive: bool,
    pub orm_invisible: bool,
    pub likely_owner: String,
    pub explanation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<SchemaObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<SchemaObject>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub total: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub destructive: usize,
    pub orm_invisible: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Review {
    pub dialect: String,
    pub before_captured_at: String,
    pub after_captured_at: String,
    pub summary: Summary,
    pub changes: Vec<Change>,
}

impl Review {
    pub fn reaches(&self, threshold: Risk) -> bool {
        self.changes.iter().any(|change| change.risk >= threshold)
    }
}

/// Compare two compatible snapshots and classify every catalog difference.
pub fn compare(before: &Snapshot, after: &Snapshot) -> Result<Review> {
    before.validate()?;
    after.validate()?;
    if before.dialect != after.dialect {
        bail!(
            "cannot compare {} and {} snapshots",
            before.dialect,
            after.dialect
        );
    }
    if before.redacted != after.redacted {
        bail!("cannot compare a redacted snapshot with an unredacted snapshot");
    }
    if before.redaction_key_id != after.redaction_key_id {
        bail!("redacted snapshots were made with different redaction keys");
    }

    let old = before
        .objects
        .iter()
        .map(|object| (object.key(), object))
        .collect::<BTreeMap<_, _>>();
    let new = after
        .objects
        .iter()
        .map(|object| (object.key(), object))
        .collect::<BTreeMap<_, _>>();
    let mut changes = Vec::new();

    for (key, old_object) in &old {
        match new.get(key) {
            None => changes.push(classify(
                ChangeKind::Removed,
                Some((*old_object).clone()),
                None,
            )),
            Some(new_object) if old_object.details != new_object.details => changes.push(classify(
                ChangeKind::Modified,
                Some((*old_object).clone()),
                Some((*new_object).clone()),
            )),
            _ => {}
        }
    }
    for (key, new_object) in &new {
        if !old.contains_key(key) {
            changes.push(classify(
                ChangeKind::Added,
                None,
                Some((*new_object).clone()),
            ));
        }
    }
    changes.sort_by(|a, b| b.risk.cmp(&a.risk).then_with(|| a.object.cmp(&b.object)));

    let summary = Summary {
        total: changes.len(),
        high: changes.iter().filter(|c| c.risk == Risk::High).count(),
        medium: changes.iter().filter(|c| c.risk == Risk::Medium).count(),
        low: changes.iter().filter(|c| c.risk == Risk::Low).count(),
        destructive: changes.iter().filter(|c| c.destructive).count(),
        orm_invisible: changes.iter().filter(|c| c.orm_invisible).count(),
    };
    Ok(Review {
        dialect: before.dialect.to_string(),
        before_captured_at: before.captured_at.clone(),
        after_captured_at: after.captured_at.clone(),
        summary,
        changes,
    })
}

fn classify(
    change: ChangeKind,
    before: Option<SchemaObject>,
    after: Option<SchemaObject>,
) -> Change {
    let object = after
        .as_ref()
        .or(before.as_ref())
        .expect("change has an object");
    let modified_column_risk = || {
        let old = before.as_ref().expect("modified change has before");
        let new = after.as_ref().expect("modified change has after");
        let old_type = old.details.get("data_type");
        let new_type = new.details.get("data_type");
        let made_required = old.details.get("nullable").and_then(|v| v.as_bool()) == Some(true)
            && new.details.get("nullable").and_then(|v| v.as_bool()) == Some(false);
        old_type != new_type || made_required
    };
    let (risk, destructive, explanation) = match (change, object.kind) {
        (ChangeKind::Removed, ObjectKind::Table | ObjectKind::View | ObjectKind::Column) => (
            Risk::High,
            true,
            format!("A {} disappeared; applications or stored queries may still depend on it.", object.kind),
        ),
        (ChangeKind::Modified, ObjectKind::Column) if modified_column_risk() => (
            Risk::High,
            true,
            "The column type or nullability became stricter; existing data or writes may no longer fit.".to_owned(),
        ),
        (ChangeKind::Added, ObjectKind::Column)
            if object.details.get("nullable").and_then(|v| v.as_bool()) == Some(false)
                && object.details.get("default").is_none_or(|v| v.is_null()) =>
        {
            (
                Risk::High,
                false,
                "A required column was added without a default; deploy ordering and existing rows need review.".to_owned(),
            )
        }
        (ChangeKind::Removed, ObjectKind::Index | ObjectKind::ForeignKey) => (
            Risk::Medium,
            false,
            format!("A {} was removed; query behavior or integrity guarantees may change.", object.kind),
        ),
        (ChangeKind::Modified, ObjectKind::View | ObjectKind::Index | ObjectKind::ForeignKey) => (
            Risk::Medium,
            false,
            format!("The {} definition changed and should be reconciled with migration history.", object.kind),
        ),
        (ChangeKind::Added, ObjectKind::View) => (
            Risk::Medium,
            false,
            "A database view appeared; many ORMs do not represent view relationships completely.".to_owned(),
        ),
        (ChangeKind::Modified, ObjectKind::Table) => (
            Risk::Medium,
            false,
            "The table kind or catalog properties changed.".to_owned(),
        ),
        (ChangeKind::Added, ObjectKind::Index | ObjectKind::ForeignKey) => (
            Risk::Low,
            false,
            format!("An additive {} appeared; confirm it belongs to the intended migration.", object.kind),
        ),
        (ChangeKind::Added, _) => (
            Risk::Low,
            false,
            format!("An additive {} appeared; verify deploy ordering and ownership.", object.kind),
        ),
        (ChangeKind::Modified, _) => (
            Risk::Medium,
            false,
            format!("The {} metadata changed and needs human review.", object.kind),
        ),
    };
    let orm_invisible = object.kind == ObjectKind::View
        || (object.kind == ObjectKind::Index
            && object
                .details
                .get("definition")
                .and_then(|v| v.as_str())
                .is_some_and(|value| value.contains(" WHERE ")));
    let likely_owner = if object.kind == ObjectKind::View {
        "Database DDL; may be absent from ORM models"
    } else if object.kind == ObjectKind::Index
        && (object.name.ends_with("_pkey") || object.name == "PRIMARY")
    {
        "Database or ORM-generated primary-key migration"
    } else if object.kind == ObjectKind::ForeignKey
        && (object.name.ends_with("_fkey") || object.name.starts_with("fk_"))
    {
        "ORM or migration-tool constraint"
    } else {
        "Application migration or out-of-band DDL"
    }
    .to_owned();

    Change {
        change,
        object_kind: object.kind,
        object: object.display_name(),
        risk,
        destructive,
        orm_invisible,
        likely_owner,
        explanation,
        before,
        after,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::model::{Dialect, SCHEMA_VERSION};

    fn snapshot(objects: Vec<SchemaObject>) -> Snapshot {
        Snapshot {
            schema_version: SCHEMA_VERSION,
            dialect: Dialect::PostgreSql,
            captured_at: "2026-08-27T12:00:00Z".to_owned(),
            source: "test".to_owned(),
            redacted: false,
            redaction_key_id: None,
            objects,
        }
    }

    fn column(nullable: bool) -> SchemaObject {
        SchemaObject {
            kind: ObjectKind::Column,
            schema: "public".to_owned(),
            table: Some("users".to_owned()),
            name: "email".to_owned(),
            details: BTreeMap::from([
                ("nullable".to_owned(), json!(nullable)),
                ("data_type".to_owned(), json!("text")),
            ]),
        }
    }

    #[test]
    fn required_column_change_is_high_and_destructive() {
        let review = compare(
            &snapshot(vec![column(true)]),
            &snapshot(vec![column(false)]),
        )
        .unwrap();
        assert_eq!(review.summary.high, 1);
        assert!(review.changes[0].destructive);
    }

    #[test]
    fn identical_snapshots_are_empty() {
        let input = snapshot(vec![column(true)]);
        let review = compare(&input, &input).unwrap();
        assert_eq!(review.summary.total, 0);
    }
}
