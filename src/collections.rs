use crate::error::{FirebaseError, Result};
use crate::firebase::FirebaseClient;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
    pub document_count: usize,
    pub estimated_size: String,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub is_required: bool,
    pub sample_values: Vec<String>,
    pub frequency: usize,
    pub unique_values: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema {
    pub collection_name: String,
    pub total_documents: usize,
    pub fields: Vec<FieldInfo>,
    pub sample_document: Option<serde_json::Value>,
}

pub struct CollectionManager {
    client: FirebaseClient,
}

impl CollectionManager {
    pub fn new(client: FirebaseClient) -> Self {
        Self { client }
    }

    pub async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        // Since Firebase REST API doesn't have a direct "list collections" endpoint,
        // we'll discover collections by trying known collection names and checking metadata
        let mut collections = Vec::new();
        
        // Try common collection names and any we find in metadata
        let potential_collections = vec![
            "users", "posts", "comments", "products", "orders", "customers",
            "articles", "messages", "notifications", "settings", "logs",
            "events", "analytics", "feedback", "reviews", "categories",
        ];

        for collection_name in potential_collections {
            match self.get_collection_info(collection_name).await {
                Ok(info) => {
                    if info.document_count > 0 {
                        collections.push(info);
                    }
                }
                Err(_) => {
                    // Collection doesn't exist or is empty, skip
                }
            }
        }

        // Also check metadata collections
        match self.get_collection_info("_metadata_collections").await {
            Ok(info) => {
                if info.document_count > 0 {
                    collections.push(info);
                }
            }
            Err(_) => {} // Ignore if metadata collection doesn't exist
        }

        // Sort by document count (largest first)
        collections.sort_by(|a, b| b.document_count.cmp(&a.document_count));

        Ok(collections)
    }

    pub async fn get_collection_info(&self, collection_name: &str) -> Result<CollectionInfo> {
        // Get collection document count by listing documents
        let url = format!("{}/{}?key={}&pageSize=1", 
            self.client.base_url,
            collection_name.trim_start_matches('/'),
            self.client.api_key
        );
        
        let response = self.client.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(FirebaseError::NotFound(collection_name.to_string()));
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to get collection info: {}", error_text)));
        }

        let response_text = response.text().await?;
        let list_response: serde_json::Value = serde_json::from_str(&response_text)?;

        // Get document count (this is an approximation)
        let document_count = self.estimate_document_count(collection_name).await.unwrap_or(0);
        
        // Get last modified time from first document
        let last_modified = list_response
            .get("documents")
            .and_then(|docs| docs.as_array())
            .and_then(|arr| arr.first())
            .and_then(|doc| doc.get("updateTime"))
            .and_then(|time| time.as_str())
            .map(|s| s.to_string());

        Ok(CollectionInfo {
            name: collection_name.to_string(),
            document_count,
            estimated_size: self.format_size_estimate(document_count),
            last_modified,
        })
    }

    async fn estimate_document_count(&self, collection_name: &str) -> Result<usize> {
        // Get a larger sample to estimate count
        let url = format!("{}/{}?key={}&pageSize=100", 
            self.client.base_url,
            collection_name.trim_start_matches('/'),
            self.client.api_key
        );
        
        let response = self.client.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Ok(0);
        }

        let response_text = response.text().await?;
        let list_response: serde_json::Value = serde_json::from_str(&response_text)?;

        let count = list_response
            .get("documents")
            .and_then(|docs| docs.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        // If we got 100 documents, there are likely more
        if count == 100 {
            // For a more accurate count, we'd need to paginate through all results
            // For now, we'll estimate based on the fact we hit the limit
            Ok(count) // This is a minimum count
        } else {
            Ok(count)
        }
    }

    fn format_size_estimate(&self, doc_count: usize) -> String {
        if doc_count == 0 {
            "Empty".to_string()
        } else {
            // Rough estimation: average document ~2KB
            let bytes = doc_count * 2048;
            if bytes < 1024 {
                format!("{}B", bytes)
            } else if bytes < 1024 * 1024 {
                format!("{:.1}KB", bytes as f64 / 1024.0)
            } else if bytes < 1024 * 1024 * 1024 {
                format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        }
    }

    pub async fn describe_collection(&self, collection_name: &str, sample_size: usize) -> Result<CollectionSchema> {
        // Get sample documents to analyze schema
        let url = format!("{}/{}?key={}&pageSize={}", 
            self.client.base_url,
            collection_name.trim_start_matches('/'),
            self.client.api_key,
            sample_size.min(100) // Firebase limits page size
        );
        
        let response = self.client.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(FirebaseError::NotFound(collection_name.to_string()));
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to describe collection: {}", error_text)));
        }

        let response_text = response.text().await?;
        let list_response: serde_json::Value = serde_json::from_str(&response_text)?;

        let empty_vec = vec![];
        let documents = list_response
            .get("documents")
            .and_then(|docs| docs.as_array())
            .unwrap_or(&empty_vec);

        if documents.is_empty() {
            return Err(FirebaseError::NotFound(format!("No documents found in collection {}", collection_name)));
        }

        // Analyze field patterns
        let mut field_stats: HashMap<String, FieldStats> = HashMap::new();
        let mut sample_document = None;

        for (i, doc) in documents.iter().enumerate() {
            if i == 0 {
                // Keep first document as sample
                sample_document = Some(self.convert_firestore_doc_to_json(doc));
            }

            if let Some(fields) = doc.get("fields").and_then(|f| f.as_object()) {
                for (field_name, field_value) in fields {
                    let stats = field_stats.entry(field_name.clone()).or_insert(FieldStats::new());
                    stats.frequency += 1;
                    
                    let (field_type, sample_value) = self.analyze_firestore_field(field_value);
                    stats.field_types.insert(field_type);
                    
                    if stats.sample_values.len() < 5 {
                        stats.sample_values.insert(sample_value);
                    }
                }
            }
        }

        // Convert field stats to field info
        let total_docs = documents.len();
        let fields: Vec<FieldInfo> = field_stats.into_iter().map(|(name, stats)| {
            let field_type = if stats.field_types.len() == 1 {
                stats.field_types.iter().next().unwrap().clone()
            } else {
                format!("Mixed({})", stats.field_types.iter().cloned().collect::<Vec<_>>().join(", "))
            };

            let is_required = stats.frequency == total_docs;
            let sample_values: Vec<String> = stats.sample_values.into_iter().collect();

            FieldInfo {
                name,
                field_type,
                is_required,
                frequency: stats.frequency,
                unique_values: sample_values.len(),
                sample_values,
            }
        }).collect();

        Ok(CollectionSchema {
            collection_name: collection_name.to_string(),
            total_documents: total_docs,
            fields,
            sample_document,
        })
    }

    fn convert_firestore_doc_to_json(&self, doc: &serde_json::Value) -> serde_json::Value {
        if let Some(fields) = doc.get("fields").and_then(|f| f.as_object()) {
            let mut json_doc = serde_json::Map::new();
            
            for (key, value) in fields {
                json_doc.insert(key.clone(), self.convert_firestore_value_to_json(value));
            }
            
            serde_json::Value::Object(json_doc)
        } else {
            serde_json::Value::Null
        }
    }

    fn convert_firestore_value_to_json(&self, value: &serde_json::Value) -> serde_json::Value {
        if let Some(string_val) = value.get("stringValue").and_then(|v| v.as_str()) {
            serde_json::Value::String(string_val.to_string())
        } else if let Some(int_val) = value.get("integerValue").and_then(|v| v.as_str()) {
            if let Ok(num) = int_val.parse::<i64>() {
                serde_json::Value::Number(serde_json::Number::from(num))
            } else {
                serde_json::Value::String(int_val.to_string())
            }
        } else if let Some(double_val) = value.get("doubleValue").and_then(|v| v.as_f64()) {
            serde_json::Value::Number(serde_json::Number::from_f64(double_val).unwrap_or(serde_json::Number::from(0)))
        } else if let Some(bool_val) = value.get("booleanValue").and_then(|v| v.as_bool()) {
            serde_json::Value::Bool(bool_val)
        } else if let Some(timestamp) = value.get("timestampValue").and_then(|v| v.as_str()) {
            serde_json::Value::String(timestamp.to_string())
        } else if let Some(array) = value.get("arrayValue").and_then(|v| v.get("values")).and_then(|v| v.as_array()) {
            let json_array: Vec<serde_json::Value> = array.iter()
                .map(|item| self.convert_firestore_value_to_json(item))
                .collect();
            serde_json::Value::Array(json_array)
        } else if let Some(map) = value.get("mapValue").and_then(|v| v.get("fields")).and_then(|v| v.as_object()) {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                json_map.insert(k.clone(), self.convert_firestore_value_to_json(v));
            }
            serde_json::Value::Object(json_map)
        } else {
            serde_json::Value::Null
        }
    }

    fn analyze_firestore_field(&self, field_value: &serde_json::Value) -> (String, String) {
        if let Some(string_val) = field_value.get("stringValue").and_then(|v| v.as_str()) {
            ("string".to_string(), format!("\"{}\"", string_val))
        } else if let Some(int_val) = field_value.get("integerValue").and_then(|v| v.as_str()) {
            ("integer".to_string(), int_val.to_string())
        } else if let Some(double_val) = field_value.get("doubleValue") {
            ("double".to_string(), double_val.to_string())
        } else if let Some(bool_val) = field_value.get("booleanValue") {
            ("boolean".to_string(), bool_val.to_string())
        } else if field_value.get("timestampValue").is_some() {
            ("timestamp".to_string(), "2024-01-01T00:00:00Z".to_string())
        } else if field_value.get("arrayValue").is_some() {
            ("array".to_string(), "[...]".to_string())
        } else if field_value.get("mapValue").is_some() {
            ("map".to_string(), "{...}".to_string())
        } else if field_value.get("nullValue").is_some() {
            ("null".to_string(), "null".to_string())
        } else {
            ("unknown".to_string(), "?".to_string())
        }
    }

    pub fn format_collections_table(&self, collections: &[CollectionInfo], use_table: bool) -> String {
        if !use_table {
            let mut output = String::new();
            output.push_str("Collections:\n");
            for collection in collections {
                output.push_str(&format!("  {} - {} documents ({})\n", 
                    collection.name, 
                    collection.document_count, 
                    collection.estimated_size
                ));
            }
            return output;
        }

        let mut table = Table::new();
        table
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Collection").add_attribute(Attribute::Bold).fg(Color::Blue),
                Cell::new("Documents").add_attribute(Attribute::Bold).fg(Color::Blue),
                Cell::new("Est. Size").add_attribute(Attribute::Bold).fg(Color::Blue),
                Cell::new("Last Modified").add_attribute(Attribute::Bold).fg(Color::Blue),
            ]);

        for collection in collections {
            let last_modified = collection.last_modified.as_deref()
                .map(|s| {
                    if s.len() > 19 {
                        &s[..19]
                    } else {
                        s
                    }
                })
                .unwrap_or("Unknown");

            table.add_row(vec![
                Cell::new(&collection.name).fg(Color::Green),
                Cell::new(collection.document_count.to_string()).fg(Color::Yellow),
                Cell::new(&collection.estimated_size),
                Cell::new(last_modified),
            ]);
        }

        table.to_string()
    }

    pub fn format_schema_table(&self, schema: &CollectionSchema, use_table: bool) -> String {
        if !use_table {
            let mut output = String::new();
            output.push_str(&format!("Collection: {} ({} documents)\n\n", schema.collection_name, schema.total_documents));
            output.push_str("Fields:\n");
            for field in &schema.fields {
                output.push_str(&format!("  {} ({}){} - {} occurrences\n", 
                    field.name,
                    field.field_type,
                    if field.is_required { " *required" } else { "" },
                    field.frequency
                ));
                if !field.sample_values.is_empty() {
                    output.push_str(&format!("    Samples: {}\n", field.sample_values.join(", ")));
                }
            }
            return output;
        }

        let mut table = Table::new();
        table
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("Field Name").add_attribute(Attribute::Bold).fg(Color::Blue),
                Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Blue),
                Cell::new("Required").add_attribute(Attribute::Bold).fg(Color::Blue),
                Cell::new("Frequency").add_attribute(Attribute::Bold).fg(Color::Blue),
                Cell::new("Sample Values").add_attribute(Attribute::Bold).fg(Color::Blue),
            ]);

        for field in &schema.fields {
            let required_indicator = if field.is_required {
                Cell::new("✓").fg(Color::Green)
            } else {
                Cell::new("✗").fg(Color::Red)
            };

            let sample_display = if field.sample_values.len() > 3 {
                format!("{}, ... ({} total)", field.sample_values[..3].join(", "), field.sample_values.len())
            } else {
                field.sample_values.join(", ")
            };

            table.add_row(vec![
                Cell::new(&field.name).fg(Color::Green),
                Cell::new(&field.field_type).fg(Color::Yellow),
                required_indicator,
                Cell::new(format!("{}/{}", field.frequency, schema.total_documents)),
                Cell::new(sample_display),
            ]);
        }

        let mut output = String::new();
        output.push_str(&format!("📊 Collection: {} ({} documents)\n\n", schema.collection_name, schema.total_documents));
        output.push_str(&table.to_string());
        
        if let Some(sample) = &schema.sample_document {
            output.push_str("\n\n📄 Sample Document:\n");
            output.push_str(&serde_json::to_string_pretty(sample).unwrap_or_else(|_| "Unable to display".to_string()));
        }

        output
    }
}

#[derive(Debug)]
struct FieldStats {
    frequency: usize,
    field_types: HashSet<String>,
    sample_values: HashSet<String>,
}

impl FieldStats {
    fn new() -> Self {
        Self {
            frequency: 0,
            field_types: HashSet::new(),
            sample_values: HashSet::new(),
        }
    }
}