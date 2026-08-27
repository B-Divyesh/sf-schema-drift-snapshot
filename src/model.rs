use std::collections::BTreeMap;
use std::collections::BTreeSet;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    PostgreSql,
    MySql,
}

impl std::fmt::Display for Dialect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PostgreSql => write!(f, "postgresql"),
            Self::MySql => write!(f, "mysql"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Table,
    View,
    Column,
    Index,
    ForeignKey,
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Table => "table",
            Self::View => "view",
            Self::Column => "column",
            Self::Index => "index",
            Self::ForeignKey => "foreign key",
        };
        write!(f, "{value}")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaObject {
    pub kind: ObjectKind,
    pub schema: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, Value>,
}

impl SchemaObject {
    pub fn key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.kind,
            self.schema,
            self.table.as_deref().unwrap_or(""),
            self.name
        )
    }

    pub fn display_name(&self) -> String {
        match &self.table {
            Some(table) => format!("{}.{}.{}", self.schema, table, self.name),
            None => format!("{}.{}", self.schema, self.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema_version: u8,
    pub dialect: Dialect,
    pub captured_at: String,
    pub source: String,
    pub redacted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redaction_key_id: Option<String>,
    pub objects: Vec<SchemaObject>,
}

impl Snapshot {
    pub fn read(path: &std::path::Path) -> Result<Self> {
        let input = std::fs::read_to_string(path)
            .with_context(|| format!("could not read snapshot {}", path.display()))?;
        let snapshot: Self = serde_json::from_str(&input)
            .with_context(|| format!("{} is not a valid SDS snapshot", path.display()))?;
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SCHEMA_VERSION {
            bail!(
                "unsupported snapshot schema version {}; this build reads version {}",
                self.schema_version,
                SCHEMA_VERSION
            );
        }
        if self.redacted && self.redaction_key_id.is_none() {
            bail!("redacted snapshot is missing redaction_key_id");
        }
        let mut keys = BTreeSet::new();
        for object in &self.objects {
            if object.schema.trim().is_empty() || object.name.trim().is_empty() {
                bail!("snapshot contains an object with an empty schema or name");
            }
            if !keys.insert(object.key()) {
                bail!("snapshot contains duplicate object key {}", object.key());
            }
        }
        Ok(())
    }

    pub fn sort(&mut self) {
        self.objects.sort_by_key(SchemaObject::key);
    }
}
