use firebase_db::{FirebaseClient, JsonSchemaManager, CollectionManager, User, FirebaseError, TuiForm, CollectionSchema, FirestoreValue};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use std::env;
use std::path::Path;
use std::io;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Schema management commands
    Schema {
        #[command(subcommand)]
        action: SchemaActions,
    },
    /// Data management commands
    Data {
        #[command(subcommand)]
        action: DataActions,
    },
    /// Collection management commands
    Collections {
        #[command(subcommand)]
        action: CollectionActions,
    },
}

#[derive(Subcommand)]
enum SchemaActions {
    /// Export schema to JSON file
    Export {
        /// Output file path
        #[arg(short, long, default_value = "schema.json")]
        output: String,
        /// Export manually defined schemas instead of discovered ones (default: discover from database)
        #[arg(long)]
        manual: bool,
    },
    /// Import schema from JSON file
    Import {
        /// Input file path
        #[arg(short, long)]
        input: String,
    },
    /// Create example schema file
    Example {
        /// Output file path
        #[arg(short, long, default_value = "example-schema.json")]
        output: String,
    },
    /// Validate a schema file
    Validate {
        /// Schema file path
        #[arg(short, long)]
        file: String,
    },
}

#[derive(Subcommand)]
enum DataActions {
    /// Create a new document
    Create {
        /// Collection name
        #[arg(short, long)]
        collection: String,
        /// Document ID (optional, auto-generated if not provided)
        #[arg(short, long)]
        id: Option<String>,
        /// JSON data for the document (if not provided, opens TUI form)
        #[arg(short, long)]
        json: Option<String>,
        /// Use interactive TUI form even if JSON is provided
        #[arg(long)]
        interactive: bool,
    },
    /// Read/get a document by ID
    Read {
        /// Collection name
        #[arg(short, long)]
        collection: String,
        /// Document ID
        #[arg(short, long)]
        id: String,
        /// Output format (json, table, or yaml)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Update an existing document
    Update {
        /// Collection name
        #[arg(short, long)]
        collection: String,
        /// Document ID
        #[arg(short, long)]
        id: String,
        /// JSON data for updates (if not provided, opens TUI form)
        #[arg(short, long)]
        json: Option<String>,
        /// Use interactive TUI form even if JSON is provided
        #[arg(long)]
        interactive: bool,
        /// Merge with existing document (default: true)
        #[arg(long)]
        replace: bool,
    },
    /// Delete a document by ID
    Delete {
        /// Collection name
        #[arg(short, long)]
        collection: String,
        /// Document ID
        #[arg(short, long)]
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Export collection data to JSON file
    Export {
        /// Collection name
        #[arg(short, long)]
        collection: String,
        /// Output file path
        #[arg(short, long)]
        output: String,
    },
    /// Import data from JSON file
    Import {
        /// Input file path
        #[arg(short, long)]
        input: String,
        /// Target collection (optional, uses collection from file if not specified)
        #[arg(short, long)]
        collection: Option<String>,
    },
    /// Backup all collections
    Backup {
        /// Backup directory
        #[arg(short, long, default_value = "backup")]
        directory: String,
    },
    /// List all documents in a collection
    List {
        /// Collection name
        #[arg(short, long)]
        collection: String,
        /// Maximum number of documents to list
        #[arg(short, long)]
        limit: Option<usize>,
        /// Output format (table, json, or text)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[derive(Subcommand)]
enum CollectionActions {
    /// List all collections
    List {
        /// Output format (table or text)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Describe a collection's schema
    Describe {
        /// Collection name
        #[arg(short, long)]
        collection: String,
        /// Number of documents to sample for schema analysis
        #[arg(short, long, default_value = "50")]
        sample: usize,
        /// Output format (table or text)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Get collection statistics
    Info {
        /// Collection name
        #[arg(short, long)]
        collection: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), FirebaseError> {
    dotenv().ok();
    
    let cli = Cli::parse();
    
    // Initialize Firebase client
    let project_id = env::var("FIREBASE_PROJECT_ID")
        .map_err(|_| FirebaseError::ConfigError("FIREBASE_PROJECT_ID not set".to_string()))?;
    
    let api_key = env::var("FIREBASE_API_KEY")
        .map_err(|_| FirebaseError::ConfigError("FIREBASE_API_KEY not set".to_string()))?;
    
    let client = FirebaseClient::new(project_id, api_key);
    let mut json_manager = JsonSchemaManager::new(client.clone());
    let collection_manager = CollectionManager::new(client);
    
    match cli.command {
        Commands::Schema { action } => {
            handle_schema_command(&mut json_manager, action).await?;
        }
        Commands::Data { action } => {
            handle_data_command(&json_manager, &collection_manager, action).await?;
        }
        Commands::Collections { action } => {
            handle_collections_command(&collection_manager, action).await?;
        }
    }
    
    Ok(())
}

async fn handle_schema_command(
    json_manager: &mut JsonSchemaManager, 
    action: SchemaActions
) -> Result<(), FirebaseError> {
    match action {
        SchemaActions::Export { output, manual } => {
            if manual {
                println!("📝 Exporting manually defined schemas...");
                json_manager.export_schema_to_file(&output)?;
                println!("✅ Manually defined schemas exported to {}", output);
            } else {
                println!("🔍 Discovering and exporting schemas from database...");
                json_manager.export_discovered_schemas(&output).await?;
                println!("✅ Discovered schemas exported to {}", output);
            }
        }
        SchemaActions::Import { input } => {
            if !Path::new(&input).exists() {
                return Err(FirebaseError::ConfigError(format!("File not found: {}", input)));
            }
            json_manager.import_schema_from_file(&input)?;
            println!("✅ Schema imported from {}", input);
        }
        SchemaActions::Example { output } => {
            json_manager.create_example_schema_file(&output)?;
            println!("✅ Example schema created at {}", output);
            println!("📝 Edit this file to define your collections, then use 'schema import' to load it");
        }
        SchemaActions::Validate { file } => {
            if !Path::new(&file).exists() {
                return Err(FirebaseError::ConfigError(format!("File not found: {}", file)));
            }
            match json_manager.import_schema_from_file(&file) {
                Ok(_) => println!("✅ Schema file {} is valid", file),
                Err(e) => {
                    println!("❌ Schema file {} is invalid: {}", file, e);
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

async fn handle_data_command(
    json_manager: &JsonSchemaManager,
    collection_manager: &CollectionManager,
    action: DataActions
) -> Result<(), FirebaseError> {
    let client = json_manager.get_client();
    
    match action {
        DataActions::Create { collection, id, json, interactive } => {
            let data = if interactive || json.is_none() {
                // Use TUI form
                let schema = match collection_manager.describe_collection(&collection, 10).await {
                    Ok(schema) => schema,
                    Err(_) => {
                        // Create a basic schema if collection doesn't exist
                        CollectionSchema {
                            collection_name: collection.clone(),
                            total_documents: 0,
                            fields: vec![],
                            sample_document: None,
                        }
                    }
                };
                
                let mut form = TuiForm::from_schema(&collection, &schema);
                println!("🖥️ Opening interactive form for document creation...");
                
                match form.run()? {
                    Some(data) => data,
                    None => {
                        println!("❌ Document creation cancelled");
                        return Ok(());
                    }
                }
            } else {
                serde_json::from_str(&json.unwrap())
                    .map_err(|e| FirebaseError::ValidationError(format!("Invalid JSON: {}", e)))?
            };
            
            println!("🔄 Creating document in collection '{}'...", collection);
            let doc_id = client.create_document(&collection, id, data).await?;
            println!("✅ Document created with ID: {}", doc_id);
        }
        
        DataActions::Read { collection, id, format } => {
            println!("🔍 Reading document '{}' from collection '{}'...", id, collection);
            let data = client.get_document(&collection, &id).await?;
            
            match format.to_lowercase().as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&data)?);
                }
                "table" => {
                    display_document_table(&id, &data);
                }
                "yaml" => {
                    // Simple YAML-like output
                    display_document_yaml(&id, &data);
                }
                _ => {
                    println!("❌ Unsupported format '{}'. Use: json, table, or yaml", format);
                    return Err(FirebaseError::ValidationError(format!("Unsupported format: {}", format)));
                }
            }
        }
        
        DataActions::Update { collection, id, json, interactive, replace } => {
            let data = if interactive || json.is_none() {
                // Get existing document for the form
                let existing_data = client.get_document(&collection, &id).await?;
                
                let mut form = TuiForm::from_existing_data(&collection, &id, &existing_data);
                println!("🖥️ Opening interactive form for document update...");
                
                match form.run()? {
                    Some(data) => data,
                    None => {
                        println!("❌ Document update cancelled");
                        return Ok(());
                    }
                }
            } else {
                serde_json::from_str(&json.unwrap())
                    .map_err(|e| FirebaseError::ValidationError(format!("Invalid JSON: {}", e)))?
            };
            
            let merge_mode = !replace;
            println!("🔄 Updating document '{}' in collection '{}' (merge: {})...", id, collection, merge_mode);
            client.update_document(&collection, &id, data, merge_mode).await?;
            println!("✅ Document updated successfully");
        }
        
        DataActions::Delete { collection, id, yes } => {
            if !yes {
                println!("⚠️ Are you sure you want to delete document '{}' from collection '{}'? [y/N]", id, collection);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("❌ Deletion cancelled");
                    return Ok(());
                }
            }
            
            println!("🗑️ Deleting document '{}' from collection '{}'...", id, collection);
            client.delete_document(&collection, &id).await?;
            println!("✅ Document deleted successfully");
        }
        
        DataActions::Export { collection, output } => {
            println!("🔄 Exporting data from collection '{}'...", collection);
            let count = json_manager.export_collection_raw(&collection, &output).await?;
            println!("✅ Exported {} items from '{}' to {}", count, collection, output);
        }
        DataActions::Import { input, collection } => {
            if !Path::new(&input).exists() {
                return Err(FirebaseError::ConfigError(format!("File not found: {}", input)));
            }
            println!("🔄 Importing data from {}...", input);
            let count = json_manager.import_collection_data::<User>(&input, collection.as_deref()).await?;
            println!("✅ Imported {} items", count);
        }
        DataActions::Backup { directory } => {
            println!("🔄 Creating backup in directory '{}'...", directory);
            let results = json_manager.backup_all_data(&directory).await?;
            let total: usize = results.values().sum();
            println!("✅ Backup completed! Total items backed up: {}", total);
            for (collection, count) in results {
                println!("  - {}: {} items", collection, count);
            }
        }
        DataActions::List { collection, limit, format } => {
            println!("📋 Listing documents from collection '{}':", collection);
            
            // Use generic document listing instead of User-specific
            let documents = list_collection_documents(&client, &collection, limit).await?;
            
            if documents.is_empty() {
                println!("  No documents found.");
            } else {
                match format.to_lowercase().as_str() {
                    "table" => {
                        display_documents_table(&collection, &documents);
                    }
                    "json" => {
                        for (i, (doc_id, data)) in documents.iter().enumerate() {
                            println!("{}. Document ID: {}", i + 1, doc_id);
                            println!("{}", serde_json::to_string_pretty(data)?);
                            println!();
                        }
                    }
                    "text" => {
                        for (i, (doc_id, data)) in documents.iter().enumerate() {
                            println!("{}. {} - {} fields", i + 1, doc_id, count_fields(data));
                        }
                    }
                    _ => {
                        println!("❌ Unsupported format '{}'. Use: table, json, or text", format);
                        return Err(FirebaseError::ValidationError(format!("Unsupported format: {}", format)));
                    }
                }
                println!("  Total: {} documents", documents.len());
            }
        }
    }
    Ok(())
}

async fn handle_collections_command(
    collection_manager: &CollectionManager,
    action: CollectionActions
) -> Result<(), FirebaseError> {
    match action {
        CollectionActions::List { format } => {
            println!("🔍 Discovering collections...");
            let collections = collection_manager.list_collections().await?;
            
            if collections.is_empty() {
                println!("❌ No collections found or all collections are empty");
                return Ok(());
            }

            let use_table = format.to_lowercase() == "table";
            let output = collection_manager.format_collections_table(&collections, use_table);
            
            if use_table {
                println!("📊 Firebase Collections\n");
            }
            println!("{}", output);
            
            println!("\n📈 Summary: Found {} collection(s)", collections.len());
        }
        CollectionActions::Describe { collection, sample, format } => {
            println!("🔍 Analyzing collection '{}'...", collection);
            
            match collection_manager.describe_collection(&collection, sample).await {
                Ok(schema) => {
                    let use_table = format.to_lowercase() == "table";
                    let output = collection_manager.format_schema_table(&schema, use_table);
                    println!("{}", output);
                }
                Err(FirebaseError::NotFound(_)) => {
                    println!("❌ Collection '{}' not found or is empty", collection);
                }
                Err(e) => {
                    println!("❌ Error analyzing collection: {}", e);
                    return Err(e);
                }
            }
        }
        CollectionActions::Info { collection } => {
            println!("📊 Getting info for collection '{}'...", collection);
            
            match collection_manager.get_collection_info(&collection).await {
                Ok(info) => {
                    println!("Collection: {}", info.name);
                    println!("Documents: {}", info.document_count);
                    println!("Estimated Size: {}", info.estimated_size);
                    if let Some(last_modified) = info.last_modified {
                        println!("Last Modified: {}", last_modified);
                    } else {
                        println!("Last Modified: Unknown");
                    }
                }
                Err(FirebaseError::NotFound(_)) => {
                    println!("❌ Collection '{}' not found", collection);
                }
                Err(e) => {
                    println!("❌ Error getting collection info: {}", e);
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

// Helper functions for document display and listing
async fn list_collection_documents(
    client: &FirebaseClient, 
    collection: &str, 
    limit: Option<usize>
) -> Result<Vec<(String, serde_json::Value)>, FirebaseError> {
    // This is a simplified version - ideally we'd implement a generic list method
    // For now, we'll make a request to list documents directly
    let url = format!("{}?key={}", 
        format!("{}/{}", client.base_url, collection.trim_start_matches('/')), 
        client.api_key);
    
    let response = client.client
        .get(&url)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(FirebaseError::DatabaseError(format!("List failed: {}", error_text)));
    }
    
    #[derive(serde::Deserialize)]
    struct ListResponse {
        documents: Option<Vec<DocumentResponse>>,
    }
    
    #[derive(serde::Deserialize)]
    struct DocumentResponse {
        name: String,
        fields: std::collections::HashMap<String, FirestoreValue>,
    }
    
    let list_response: ListResponse = response.json().await?;
    let mut results = Vec::new();
    
    if let Some(documents) = list_response.documents {
        for (i, doc) in documents.iter().enumerate() {
            if let Some(limit_val) = limit {
                if i >= limit_val {
                    break;
                }
            }
            
            let doc_id = doc.name.split('/').last().unwrap_or("unknown").to_string();
            
            // Convert Firestore fields to JSON
            let mut json_fields = serde_json::Map::new();
            for (key, value) in &doc.fields {
                if let Ok(json_value) = firestore_value_to_json_value(value) {
                    json_fields.insert(key.clone(), json_value);
                }
            }
            
            results.push((doc_id, serde_json::Value::Object(json_fields)));
        }
    }
    
    Ok(results)
}

fn firestore_value_to_json_value(value: &FirestoreValue) -> Result<serde_json::Value, FirebaseError> {
    match value {
        FirestoreValue::StringValue(s) => Ok(serde_json::Value::String(s.clone())),
        FirestoreValue::IntegerValue(i) => {
            Ok(serde_json::Value::Number(serde_json::Number::from(i.parse::<i64>().unwrap_or(0))))
        }
        FirestoreValue::DoubleValue(f) => {
            Ok(serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0))))
        }
        FirestoreValue::BooleanValue(b) => Ok(serde_json::Value::Bool(*b)),
        FirestoreValue::NullValue => Ok(serde_json::Value::Null),
        FirestoreValue::TimestampValue(ts) => Ok(serde_json::Value::String(ts.clone())),
        _ => Ok(serde_json::Value::String("(complex value)".to_string())),
    }
}

fn display_document_table(doc_id: &str, data: &serde_json::Value) {
    use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};
    
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("Field").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Value").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Cyan),
    ]);
    
    if let serde_json::Value::Object(map) = data {
        for (key, value) in map {
            let value_str = match value {
                serde_json::Value::String(s) => format!("\"{}\"", s),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
                serde_json::Value::Object(obj) => format!("{{{} fields}}", obj.len()),
                serde_json::Value::Null => "null".to_string(),
            };
            
            let type_str = match value {
                serde_json::Value::String(_) => "string",
                serde_json::Value::Number(n) if n.is_i64() => "integer",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
                serde_json::Value::Null => "null",
            };
            
            table.add_row(vec![
                Cell::new(key).fg(Color::Yellow),
                Cell::new(value_str),
                Cell::new(type_str).fg(Color::Grey),
            ]);
        }
    }
    
    println!("\n📄 Document: {}", doc_id);
    println!("{}", table);
}

fn display_document_yaml(doc_id: &str, data: &serde_json::Value) {
    println!("\n📄 Document: {}", doc_id);
    println!("---");
    
    if let serde_json::Value::Object(map) = data {
        for (key, value) in map {
            let value_str = match value {
                serde_json::Value::String(s) => format!("\"{}\"", s),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Array(_) => "[array]".to_string(),
                serde_json::Value::Object(_) => "{object}".to_string(),
                serde_json::Value::Null => "null".to_string(),
            };
            println!("{}: {}", key, value_str);
        }
    }
    println!("---");
}

fn display_documents_table(collection_name: &str, documents: &[(String, serde_json::Value)]) {
    use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};
    
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    
    // Determine common fields across all documents
    let mut all_fields = std::collections::BTreeSet::new();
    for (_, data) in documents {
        if let serde_json::Value::Object(map) = data {
            for key in map.keys() {
                all_fields.insert(key.clone());
            }
        }
    }
    
    // Set up table headers
    let mut headers = vec![Cell::new("ID").add_attribute(Attribute::Bold).fg(Color::Cyan)];
    for field in &all_fields {
        headers.push(Cell::new(field).add_attribute(Attribute::Bold).fg(Color::Cyan));
    }
    table.set_header(headers);
    
    // Add rows
    for (doc_id, data) in documents {
        let mut row = vec![Cell::new(doc_id).fg(Color::Yellow)];
        
        if let serde_json::Value::Object(map) = data {
            for field in &all_fields {
                let cell_value = if let Some(value) = map.get(field) {
                    match value {
                        serde_json::Value::String(s) => {
                            if s.len() > 30 {
                                format!("{}...", &s[..27])
                            } else {
                                s.clone()
                            }
                        }
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Array(arr) => format!("[{}]", arr.len()),
                        serde_json::Value::Object(obj) => format!("{{{}}}", obj.len()),
                        serde_json::Value::Null => "-".to_string(),
                    }
                } else {
                    "-".to_string()
                };
                row.push(Cell::new(cell_value));
            }
        }
        
        table.add_row(row);
    }
    
    println!("\n📊 Collection: {}", collection_name);
    println!("{}", table);
}

fn count_fields(data: &serde_json::Value) -> usize {
    match data {
        serde_json::Value::Object(map) => map.len(),
        _ => 0,
    }
}