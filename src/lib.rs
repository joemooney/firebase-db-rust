pub mod firebase;
pub mod models;
pub mod error;
pub mod query;
pub mod schema;
pub mod security_rules;
pub mod json_manager;
pub mod collections;

pub use firebase::FirebaseClient;
pub use models::*;
pub use error::FirebaseError;
pub use query::{QueryBuilder, FieldOperator, create_filter};
pub use schema::{SchemaManager, Collection, Field, FieldType, Index, IndexField, IndexOrder, ValidationRule, ValidationRuleType, MigrationManager, Migration};
pub use security_rules::{SecurityRules, RuleBuilder, Expression, Permission};
pub use json_manager::{JsonSchemaManager, JsonSchema, JsonCollection, JsonField, DataExport};
pub use collections::{CollectionManager, CollectionInfo, CollectionSchema, FieldInfo};