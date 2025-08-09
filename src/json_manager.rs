use crate::error::{FirebaseError, Result};
use crate::firebase::FirebaseClient;
use crate::models::{FromFirestore, ToFirestore, FirestoreValue};
use crate::schema::{SchemaManager, Collection, Field, FieldType, Index, IndexField, IndexOrder, ValidationRule, ValidationRuleType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    pub version: String,
    pub collections: HashMap<String, JsonCollection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonCollection {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<JsonField>,
    pub indexes: Vec<JsonIndex>,
    pub validation_rules: Vec<JsonValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonIndex {
    pub fields: Vec<JsonIndexField>,
    pub unique: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonIndexField {
    pub field_path: String,
    pub order: String, // "asc" or "desc"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonValidationRule {
    pub field: String,
    pub rule_type: String,
    pub value: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExport {
    pub collection: String,
    pub exported_at: String,
    pub count: usize,
    pub data: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct JsonSchemaManager {
    schema_manager: SchemaManager,
    client: FirebaseClient,
}

impl JsonSchemaManager {
    pub fn new(client: FirebaseClient) -> Self {
        Self {
            schema_manager: SchemaManager::new(client.clone()),
            client,
        }
    }

    // Schema Import/Export
    pub fn export_schema_to_file(&self, file_path: &str) -> Result<()> {
        let json_schema = self.convert_to_json_schema();
        let json_content = serde_json::to_string_pretty(&json_schema)?;
        fs::write(file_path, json_content)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to write schema file: {}", e)))?;
        Ok(())
    }

    pub fn import_schema_from_file(&mut self, file_path: &str) -> Result<()> {
        let json_content = fs::read_to_string(file_path)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to read schema file: {}", e)))?;
        let json_schema: JsonSchema = serde_json::from_str(&json_content)?;
        self.apply_json_schema(json_schema)?;
        Ok(())
    }

    pub fn create_example_schema_file(&self, file_path: &str) -> Result<()> {
        let example_schema = JsonSchema {
            version: "1.0.0".to_string(),
            collections: {
                let mut collections = HashMap::new();
                
                collections.insert("users".to_string(), JsonCollection {
                    name: "users".to_string(),
                    description: Some("User accounts".to_string()),
                    fields: vec![
                        JsonField {
                            name: "id".to_string(),
                            field_type: "string".to_string(),
                            required: false,
                            default_value: None,
                            description: Some("Auto-generated user ID".to_string()),
                        },
                        JsonField {
                            name: "name".to_string(),
                            field_type: "string".to_string(),
                            required: true,
                            default_value: None,
                            description: Some("User's full name".to_string()),
                        },
                        JsonField {
                            name: "email".to_string(),
                            field_type: "string".to_string(),
                            required: true,
                            default_value: None,
                            description: Some("User's email address".to_string()),
                        },
                        JsonField {
                            name: "age".to_string(),
                            field_type: "integer".to_string(),
                            required: true,
                            default_value: Some(serde_json::Value::Number(serde_json::Number::from(18))),
                            description: Some("User's age".to_string()),
                        },
                        JsonField {
                            name: "active".to_string(),
                            field_type: "boolean".to_string(),
                            required: false,
                            default_value: Some(serde_json::Value::Bool(true)),
                            description: Some("Account status".to_string()),
                        },
                        JsonField {
                            name: "created_at".to_string(),
                            field_type: "timestamp".to_string(),
                            required: true,
                            default_value: None,
                            description: Some("Account creation timestamp".to_string()),
                        },
                        JsonField {
                            name: "tags".to_string(),
                            field_type: "array".to_string(),
                            required: false,
                            default_value: Some(serde_json::Value::Array(vec![])),
                            description: Some("User tags".to_string()),
                        },
                        JsonField {
                            name: "profile".to_string(),
                            field_type: "map".to_string(),
                            required: false,
                            default_value: None,
                            description: Some("User profile data".to_string()),
                        },
                    ],
                    indexes: vec![
                        JsonIndex {
                            fields: vec![JsonIndexField {
                                field_path: "email".to_string(),
                                order: "asc".to_string(),
                            }],
                            unique: true,
                            description: Some("Unique email index".to_string()),
                        },
                        JsonIndex {
                            fields: vec![
                                JsonIndexField {
                                    field_path: "active".to_string(),
                                    order: "asc".to_string(),
                                },
                                JsonIndexField {
                                    field_path: "created_at".to_string(),
                                    order: "desc".to_string(),
                                },
                            ],
                            unique: false,
                            description: Some("Active users by creation date".to_string()),
                        },
                    ],
                    validation_rules: vec![
                        JsonValidationRule {
                            field: "email".to_string(),
                            rule_type: "email".to_string(),
                            value: None,
                            description: Some("Must be valid email format".to_string()),
                        },
                        JsonValidationRule {
                            field: "age".to_string(),
                            rule_type: "min".to_string(),
                            value: Some(serde_json::Value::Number(serde_json::Number::from(13))),
                            description: Some("Minimum age 13".to_string()),
                        },
                        JsonValidationRule {
                            field: "age".to_string(),
                            rule_type: "max".to_string(),
                            value: Some(serde_json::Value::Number(serde_json::Number::from(120))),
                            description: Some("Maximum age 120".to_string()),
                        },
                        JsonValidationRule {
                            field: "name".to_string(),
                            rule_type: "min_length".to_string(),
                            value: Some(serde_json::Value::Number(serde_json::Number::from(2))),
                            description: Some("Name must be at least 2 characters".to_string()),
                        },
                        JsonValidationRule {
                            field: "name".to_string(),
                            rule_type: "max_length".to_string(),
                            value: Some(serde_json::Value::Number(serde_json::Number::from(100))),
                            description: Some("Name must be at most 100 characters".to_string()),
                        },
                    ],
                });

                collections.insert("posts".to_string(), JsonCollection {
                    name: "posts".to_string(),
                    description: Some("User posts".to_string()),
                    fields: vec![
                        JsonField {
                            name: "title".to_string(),
                            field_type: "string".to_string(),
                            required: true,
                            default_value: None,
                            description: Some("Post title".to_string()),
                        },
                        JsonField {
                            name: "content".to_string(),
                            field_type: "string".to_string(),
                            required: true,
                            default_value: None,
                            description: Some("Post content".to_string()),
                        },
                        JsonField {
                            name: "author_id".to_string(),
                            field_type: "string".to_string(),
                            required: true,
                            default_value: None,
                            description: Some("Reference to user ID".to_string()),
                        },
                        JsonField {
                            name: "published".to_string(),
                            field_type: "boolean".to_string(),
                            required: false,
                            default_value: Some(serde_json::Value::Bool(false)),
                            description: Some("Publication status".to_string()),
                        },
                        JsonField {
                            name: "created_at".to_string(),
                            field_type: "timestamp".to_string(),
                            required: true,
                            default_value: None,
                            description: Some("Creation timestamp".to_string()),
                        },
                    ],
                    indexes: vec![
                        JsonIndex {
                            fields: vec![
                                JsonIndexField {
                                    field_path: "author_id".to_string(),
                                    order: "asc".to_string(),
                                },
                                JsonIndexField {
                                    field_path: "published".to_string(),
                                    order: "asc".to_string(),
                                },
                            ],
                            unique: false,
                            description: Some("Posts by author and publication status".to_string()),
                        },
                    ],
                    validation_rules: vec![
                        JsonValidationRule {
                            field: "title".to_string(),
                            rule_type: "min_length".to_string(),
                            value: Some(serde_json::Value::Number(serde_json::Number::from(5))),
                            description: Some("Title must be at least 5 characters".to_string()),
                        },
                        JsonValidationRule {
                            field: "title".to_string(),
                            rule_type: "max_length".to_string(),
                            value: Some(serde_json::Value::Number(serde_json::Number::from(200))),
                            description: Some("Title must be at most 200 characters".to_string()),
                        },
                        JsonValidationRule {
                            field: "content".to_string(),
                            rule_type: "min_length".to_string(),
                            value: Some(serde_json::Value::Number(serde_json::Number::from(10))),
                            description: Some("Content must be at least 10 characters".to_string()),
                        },
                    ],
                });

                collections
            },
        };

        let json_content = serde_json::to_string_pretty(&example_schema)?;
        fs::write(file_path, json_content)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to write example schema: {}", e)))?;
        Ok(())
    }

    // Data Import/Export
    pub async fn export_collection_data<T>(&self, collection_name: &str, output_file: &str) -> Result<usize>
    where
        T: FromFirestore + Serialize,
    {
        let data: Vec<T> = self.client.list(collection_name, None).await?;
        
        let json_data: Vec<serde_json::Value> = data.iter()
            .map(|item| serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
            .collect();

        let export = DataExport {
            collection: collection_name.to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            count: json_data.len(),
            data: json_data,
        };

        let json_content = serde_json::to_string_pretty(&export)?;
        fs::write(output_file, json_content)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to write export file: {}", e)))?;

        Ok(export.count)
    }

    pub async fn export_collection_raw(&self, collection_name: &str, output_file: &str) -> Result<usize> {
        // Get raw documents from Firebase
        let url = format!("{}/{}?key={}", 
            self.client.base_url,
            collection_name.trim_start_matches('/'),
            self.client.api_key
        );
        
        let response = self.client.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to list documents: {}", error_text)));
        }
        
        let response_text = response.text().await?;
        let list_response: serde_json::Value = serde_json::from_str(&response_text)?;
        
        let empty_vec = vec![];
        let documents = list_response.get("documents")
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_vec);
        
        let json_data: Vec<serde_json::Value> = documents.iter()
            .filter_map(|doc| {
                if let Some(fields) = doc.get("fields") {
                    Some(fields.clone())
                } else {
                    None
                }
            })
            .collect();

        let export = DataExport {
            collection: collection_name.to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            count: json_data.len(),
            data: json_data,
        };

        let json_content = serde_json::to_string_pretty(&export)?;
        fs::write(output_file, json_content)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to write export file: {}", e)))?;

        Ok(export.count)
    }

    pub async fn import_collection_data<T>(&self, input_file: &str, collection_name: Option<&str>) -> Result<usize>
    where
        T: for<'de> Deserialize<'de> + ToFirestore,
    {
        let json_content = fs::read_to_string(input_file)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to read import file: {}", e)))?;

        let export: DataExport = serde_json::from_str(&json_content)?;
        let target_collection = collection_name.unwrap_or(&export.collection);

        let mut imported_count = 0;
        for item_value in export.data {
            match serde_json::from_value::<T>(item_value) {
                Ok(item) => {
                    if let Err(e) = self.client.create(target_collection, &item).await {
                        eprintln!("Failed to import item: {}", e);
                    } else {
                        imported_count += 1;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to deserialize item: {}", e);
                }
            }
        }

        Ok(imported_count)
    }

    pub async fn backup_all_data(&self, backup_dir: &str) -> Result<HashMap<String, usize>> {
        fs::create_dir_all(backup_dir)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to create backup directory: {}", e)))?;

        let mut results = HashMap::new();

        // For now, we'll backup known collections
        // In a real implementation, you might want to discover collections
        let collections = vec!["users"];

        for collection in collections {
            let output_file = format!("{}/{}_backup.json", backup_dir, collection);
            match self.export_collection_raw(collection, &output_file).await {
                Ok(count) => {
                    results.insert(collection.to_string(), count);
                    println!("Backed up {} items from {}", count, collection);
                },
                Err(e) => {
                    eprintln!("Failed to backup {}: {}", collection, e);
                    results.insert(collection.to_string(), 0);
                }
            }
        }

        Ok(results)
    }

    // Enhanced schema export that includes discovered schemas
    pub async fn export_discovered_schemas(&self, output_file: &str) -> Result<()> {
        use crate::collections::CollectionManager;
        
        let collection_manager = CollectionManager::new(self.client.clone());
        
        // Get all collections from database
        let collections = collection_manager.list_collections().await?;
        
        let mut json_schema = JsonSchema {
            version: "1.0.0".to_string(),
            collections: HashMap::new(),
        };
        
        // Convert discovered collections to JSON schema format
        for collection_info in collections {
            println!("📊 Analyzing schema for collection '{}'...", collection_info.name);
            
            match collection_manager.describe_collection(&collection_info.name, 50).await {
                Ok(schema) => {
                    let json_collection = self.convert_collection_schema_to_json(schema);
                    json_schema.collections.insert(collection_info.name.clone(), json_collection);
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to analyze collection '{}': {}", collection_info.name, e);
                }
            }
        }
        
        // Also include any manually defined schemas from schema_manager
        let existing_schema = self.convert_to_json_schema();
        for (name, collection) in existing_schema.collections {
            json_schema.collections.insert(name, collection);
        }
        
        // Write to file
        let json_content = serde_json::to_string_pretty(&json_schema)?;
        fs::write(output_file, json_content)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to write schema file: {}", e)))?;
            
        Ok(())
    }

    fn convert_collection_schema_to_json(&self, schema: crate::collections::CollectionSchema) -> JsonCollection {
        let fields: Vec<JsonField> = schema.fields.into_iter().map(|field| {
            // Convert field type
            let field_type = match field.field_type.as_str() {
                "string" => "string",
                "integer" => "integer", 
                "double" => "double",
                "boolean" => "boolean",
                "timestamp" => "timestamp",
                "array" => "array",
                "map" => "map",
                _ if field.field_type.starts_with("Mixed(") => "mixed",
                _ => "unknown",
            }.to_string();
            
            // Use first sample value as default if available
            let default_value = if !field.sample_values.is_empty() && field.is_required {
                match field_type.as_str() {
                    "string" => {
                        let sample = field.sample_values[0].trim_matches('"');
                        Some(serde_json::Value::String(sample.to_string()))
                    },
                    "integer" => {
                        if let Ok(num) = field.sample_values[0].parse::<i64>() {
                            Some(serde_json::Value::Number(serde_json::Number::from(num)))
                        } else {
                            None
                        }
                    },
                    "double" => {
                        if let Ok(num) = field.sample_values[0].parse::<f64>() {
                            serde_json::Number::from_f64(num).map(serde_json::Value::Number)
                        } else {
                            None
                        }
                    },
                    "boolean" => {
                        if let Ok(b) = field.sample_values[0].parse::<bool>() {
                            Some(serde_json::Value::Bool(b))
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            } else {
                None
            };
            
            JsonField {
                name: field.name.clone(),
                field_type,
                required: field.is_required,
                default_value,
                description: Some(format!("Found in {}/{} documents. Samples: {}", 
                    field.frequency, 
                    schema.total_documents,
                    field.sample_values.join(", ")
                )),
            }
        }).collect();
        
        JsonCollection {
            name: schema.collection_name.clone(),
            description: Some(format!("Discovered collection with {} documents", schema.total_documents)),
            fields,
            indexes: vec![], // Would need additional analysis to discover indexes
            validation_rules: vec![], // Could infer some rules from data patterns
        }
    }

    // Helper methods
    fn convert_to_json_schema(&self) -> JsonSchema {
        // This would convert the current schema_manager state to JsonSchema
        // For now, we'll return an empty schema with just manually defined collections
        JsonSchema {
            version: "1.0.0".to_string(),
            collections: HashMap::new(), // Could populate from schema_manager if needed
        }
    }

    fn apply_json_schema(&mut self, json_schema: JsonSchema) -> Result<()> {
        for (_, json_collection) in json_schema.collections {
            let collection = self.convert_from_json_collection(json_collection)?;
            self.schema_manager.define_collection(collection);
        }
        Ok(())
    }

    fn convert_from_json_collection(&self, json_collection: JsonCollection) -> Result<Collection> {
        let fields: Result<Vec<Field>> = json_collection.fields.into_iter()
            .map(|f| self.convert_from_json_field(f))
            .collect();

        let indexes: Result<Vec<Index>> = json_collection.indexes.into_iter()
            .map(|i| self.convert_from_json_index(i))
            .collect();

        let validation_rules: Result<Vec<ValidationRule>> = json_collection.validation_rules.into_iter()
            .map(|r| self.convert_from_json_validation_rule(r))
            .collect();

        Ok(Collection {
            name: json_collection.name,
            fields: fields?,
            indexes: indexes?,
            validation_rules: validation_rules?,
        })
    }

    fn convert_from_json_field(&self, json_field: JsonField) -> Result<Field> {
        let field_type = match json_field.field_type.as_str() {
            "string" => FieldType::String,
            "integer" => FieldType::Integer,
            "double" => FieldType::Double,
            "boolean" => FieldType::Boolean,
            "timestamp" => FieldType::Timestamp,
            "map" => FieldType::Map,
            "array" => FieldType::Array,
            "reference" => FieldType::Reference,
            _ => return Err(FirebaseError::ConfigError(format!("Unknown field type: {}", json_field.field_type))),
        };

        let default_value = json_field.default_value.map(|v| self.json_value_to_firestore_value(v)).transpose()?;

        Ok(Field {
            name: json_field.name,
            field_type,
            required: json_field.required,
            default_value,
            description: json_field.description,
        })
    }

    fn convert_from_json_index(&self, json_index: JsonIndex) -> Result<Index> {
        let fields: Result<Vec<IndexField>> = json_index.fields.into_iter()
            .map(|f| {
                let order = match f.order.as_str() {
                    "asc" => IndexOrder::Ascending,
                    "desc" => IndexOrder::Descending,
                    _ => return Err(FirebaseError::ConfigError(format!("Unknown index order: {}", f.order))),
                };
                Ok(IndexField {
                    field_path: f.field_path,
                    order,
                })
            })
            .collect();

        Ok(Index {
            fields: fields?,
            unique: json_index.unique,
        })
    }

    fn convert_from_json_validation_rule(&self, json_rule: JsonValidationRule) -> Result<ValidationRule> {
        let rule_type = match json_rule.rule_type.as_str() {
            "min_length" => {
                let value = json_rule.value.and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                ValidationRuleType::MinLength(value)
            },
            "max_length" => {
                let value = json_rule.value.and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                ValidationRuleType::MaxLength(value)
            },
            "min" => {
                let value = json_rule.value.and_then(|v| v.as_f64()).unwrap_or(0.0);
                ValidationRuleType::Min(value)
            },
            "max" => {
                let value = json_rule.value.and_then(|v| v.as_f64()).unwrap_or(0.0);
                ValidationRuleType::Max(value)
            },
            "regex" => {
                let pattern = json_rule.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
                ValidationRuleType::Regex(pattern)
            },
            "email" => ValidationRuleType::Email,
            "url" => ValidationRuleType::Url,
            "custom" => {
                let expr = json_rule.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
                ValidationRuleType::Custom(expr)
            },
            _ => return Err(FirebaseError::ConfigError(format!("Unknown validation rule type: {}", json_rule.rule_type))),
        };

        Ok(ValidationRule {
            field: json_rule.field,
            rule: rule_type,
        })
    }

    fn json_value_to_firestore_value(&self, value: serde_json::Value) -> Result<FirestoreValue> {
        match value {
            serde_json::Value::String(s) => Ok(FirestoreValue::StringValue(s)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(FirestoreValue::IntegerValue(i.to_string()))
                } else if let Some(f) = n.as_f64() {
                    Ok(FirestoreValue::DoubleValue(f))
                } else {
                    Err(FirebaseError::ConfigError("Invalid number format".to_string()))
                }
            },
            serde_json::Value::Bool(b) => Ok(FirestoreValue::BooleanValue(b)),
            serde_json::Value::Array(arr) => {
                let values: Result<Vec<FirestoreValue>> = arr.into_iter()
                    .map(|v| self.json_value_to_firestore_value(v))
                    .collect();
                Ok(FirestoreValue::ArrayValue { values: values? })
            },
            serde_json::Value::Object(obj) => {
                let mut fields = HashMap::new();
                for (k, v) in obj {
                    fields.insert(k, self.json_value_to_firestore_value(v)?);
                }
                Ok(FirestoreValue::MapValue { fields })
            },
            serde_json::Value::Null => Ok(FirestoreValue::NullValue),
        }
    }

    pub fn get_schema_manager(&self) -> &SchemaManager {
        &self.schema_manager
    }

    pub fn get_schema_manager_mut(&mut self) -> &mut SchemaManager {
        &mut self.schema_manager
    }

    pub fn get_client(&self) -> &FirebaseClient {
        &self.client
    }
}