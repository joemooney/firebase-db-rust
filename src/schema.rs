use crate::error::{FirebaseError, Result};
use crate::firebase::FirebaseClient;
use crate::models::{FirestoreValue, ToFirestore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub fields: Vec<Field>,
    pub indexes: Vec<Index>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default_value: Option<FirestoreValue>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Double,
    Boolean,
    Timestamp,
    Map,
    Array,
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub fields: Vec<IndexField>,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexField {
    pub field_path: String,
    pub order: IndexOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rule: ValidationRuleType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    MinLength(usize),
    MaxLength(usize),
    Min(f64),
    Max(f64),
    Regex(String),
    Email,
    Url,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct SchemaManager {
    client: FirebaseClient,
    collections: HashMap<String, Collection>,
}

impl SchemaManager {
    pub fn new(client: FirebaseClient) -> Self {
        Self {
            client,
            collections: HashMap::new(),
        }
    }
    
    pub fn define_collection(&mut self, collection: Collection) {
        self.collections.insert(collection.name.clone(), collection);
    }
    
    pub fn validate<T: ToFirestore>(&self, collection_name: &str, item: &T) -> Result<()> {
        let collection = self.collections.get(collection_name)
            .ok_or_else(|| FirebaseError::ConfigError(format!("Collection {} not defined", collection_name)))?;
        
        let fields = item.to_firestore();
        
        for field_def in &collection.fields {
            if field_def.required && !fields.contains_key(&field_def.name) {
                return Err(FirebaseError::DatabaseError(
                    format!("Required field '{}' is missing", field_def.name)
                ));
            }
            
            if let Some(value) = fields.get(&field_def.name) {
                self.validate_field_type(&field_def.name, &field_def.field_type, value)?;
            }
        }
        
        for rule in &collection.validation_rules {
            if let Some(value) = fields.get(&rule.field) {
                self.validate_rule(&rule.field, &rule.rule, value)?;
            }
        }
        
        Ok(())
    }
    
    fn validate_field_type(&self, field_name: &str, expected_type: &FieldType, value: &FirestoreValue) -> Result<()> {
        let valid = match (expected_type, value) {
            (FieldType::String, FirestoreValue::StringValue(_)) => true,
            (FieldType::Integer, FirestoreValue::IntegerValue(_)) => true,
            (FieldType::Double, FirestoreValue::DoubleValue(_)) => true,
            (FieldType::Boolean, FirestoreValue::BooleanValue(_)) => true,
            (FieldType::Timestamp, FirestoreValue::TimestampValue(_)) => true,
            (FieldType::Map, FirestoreValue::MapValue { .. }) => true,
            (FieldType::Array, FirestoreValue::ArrayValue { .. }) => true,
            _ => false,
        };
        
        if !valid {
            return Err(FirebaseError::DatabaseError(
                format!("Field '{}' has incorrect type. Expected {:?}", field_name, expected_type)
            ));
        }
        
        Ok(())
    }
    
    fn validate_rule(&self, field_name: &str, rule: &ValidationRuleType, value: &FirestoreValue) -> Result<()> {
        match rule {
            ValidationRuleType::MinLength(min) => {
                if let FirestoreValue::StringValue(s) = value {
                    if s.len() < *min {
                        return Err(FirebaseError::DatabaseError(
                            format!("Field '{}' must be at least {} characters", field_name, min)
                        ));
                    }
                }
            }
            ValidationRuleType::MaxLength(max) => {
                if let FirestoreValue::StringValue(s) = value {
                    if s.len() > *max {
                        return Err(FirebaseError::DatabaseError(
                            format!("Field '{}' must be at most {} characters", field_name, max)
                        ));
                    }
                }
            }
            ValidationRuleType::Min(min) => {
                match value {
                    FirestoreValue::IntegerValue(i) => {
                        if i.parse::<f64>().unwrap_or(0.0) < *min {
                            return Err(FirebaseError::DatabaseError(
                                format!("Field '{}' must be at least {}", field_name, min)
                            ));
                        }
                    }
                    FirestoreValue::DoubleValue(d) => {
                        if d < min {
                            return Err(FirebaseError::DatabaseError(
                                format!("Field '{}' must be at least {}", field_name, min)
                            ));
                        }
                    }
                    _ => {}
                }
            }
            ValidationRuleType::Max(max) => {
                match value {
                    FirestoreValue::IntegerValue(i) => {
                        if i.parse::<f64>().unwrap_or(0.0) > *max {
                            return Err(FirebaseError::DatabaseError(
                                format!("Field '{}' must be at most {}", field_name, max)
                            ));
                        }
                    }
                    FirestoreValue::DoubleValue(d) => {
                        if d > max {
                            return Err(FirebaseError::DatabaseError(
                                format!("Field '{}' must be at most {}", field_name, max)
                            ));
                        }
                    }
                    _ => {}
                }
            }
            ValidationRuleType::Email => {
                if let FirestoreValue::StringValue(s) = value {
                    if !s.contains('@') || !s.contains('.') {
                        return Err(FirebaseError::DatabaseError(
                            format!("Field '{}' must be a valid email", field_name)
                        ));
                    }
                }
            }
            ValidationRuleType::Regex(pattern) => {
                if let FirestoreValue::StringValue(s) = value {
                    let re = regex::Regex::new(pattern)
                        .map_err(|e| FirebaseError::ConfigError(format!("Invalid regex: {}", e)))?;
                    if !re.is_match(s) {
                        return Err(FirebaseError::DatabaseError(
                            format!("Field '{}' does not match pattern {}", field_name, pattern)
                        ));
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    pub async fn initialize_collections(&self) -> Result<()> {
        for (name, collection) in &self.collections {
            println!("Initializing collection: {}", name);
            
            let metadata_doc = CollectionMetadata {
                name: collection.name.clone(),
                created_at: Utc::now(),
                fields: collection.fields.clone(),
                indexes: collection.indexes.clone(),
                validation_rules: collection.validation_rules.clone(),
            };
            
            self.client.create("_metadata_collections", &metadata_doc).await?;
        }
        
        Ok(())
    }
    
    pub async fn create_indexes(&self, collection_name: &str) -> Result<()> {
        let collection = self.collections.get(collection_name)
            .ok_or_else(|| FirebaseError::ConfigError(format!("Collection {} not defined", collection_name)))?;
        
        println!("Note: Firestore indexes must be created in the Firebase Console or via Firebase CLI");
        println!("Required indexes for collection '{}':", collection_name);
        
        for (i, index) in collection.indexes.iter().enumerate() {
            println!("\nIndex {}:", i + 1);
            for field in &index.fields {
                println!("  - Field: {}, Order: {:?}", field.field_path, field.order);
            }
            if index.unique {
                println!("  - Unique: true");
            }
        }
        
        Ok(())
    }
    
    pub fn export_schema(&self) -> String {
        serde_json::to_string_pretty(&self.collections).unwrap_or_default()
    }
    
    pub fn import_schema(&mut self, json: &str) -> Result<()> {
        let collections: HashMap<String, Collection> = serde_json::from_str(json)?;
        self.collections = collections;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectionMetadata {
    name: String,
    created_at: chrono::DateTime<Utc>,
    fields: Vec<Field>,
    indexes: Vec<Index>,
    validation_rules: Vec<ValidationRule>,
}

impl ToFirestore for CollectionMetadata {
    fn to_firestore(&self) -> HashMap<String, FirestoreValue> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), FirestoreValue::StringValue(self.name.clone()));
        map.insert("created_at".to_string(), FirestoreValue::TimestampValue(self.created_at.to_rfc3339()));
        map.insert("fields".to_string(), FirestoreValue::StringValue(serde_json::to_string(&self.fields).unwrap_or_default()));
        map.insert("indexes".to_string(), FirestoreValue::StringValue(serde_json::to_string(&self.indexes).unwrap_or_default()));
        map.insert("validation_rules".to_string(), FirestoreValue::StringValue(serde_json::to_string(&self.validation_rules).unwrap_or_default()));
        map
    }
}

pub struct Migration {
    pub version: String,
    pub description: String,
    pub up: Box<dyn Fn(&FirebaseClient) -> Result<()>>,
    pub down: Box<dyn Fn(&FirebaseClient) -> Result<()>>,
}

pub struct MigrationManager {
    client: FirebaseClient,
    migrations: Vec<Migration>,
}

impl MigrationManager {
    pub fn new(client: FirebaseClient) -> Self {
        Self {
            client,
            migrations: Vec::new(),
        }
    }
    
    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }
    
    pub async fn run_migrations(&self) -> Result<()> {
        println!("Running migrations...");
        
        for migration in &self.migrations {
            println!("Running migration: {} - {}", migration.version, migration.description);
            
            // Note: In a real implementation, you'd track which migrations have been run
            // by storing them in a _migrations collection
        }
        
        Ok(())
    }
}