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
    /// Import schema from JSON or YAML file
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
    /// Discover and save schema to Firestore schemas collection
    Sync {
        /// Collection name to discover and sync
        #[arg(short, long)]
        collection: String,
        /// Number of sample documents to analyze (default: 50)
        #[arg(short, long, default_value = "50")]
        samples: usize,
    },
    /// List all schemas stored in Firestore
    List,
    /// Show detailed information about a schema stored in Firestore
    Show {
        /// Collection name
        #[arg(short, long)]
        collection: String,
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
        /// JSON or YAML data for the document (if not provided, opens TUI form)
        #[arg(short, long)]
        json: Option<String>,
        /// Use interactive TUI form even if JSON is provided
        #[arg(long)]
        interactive: bool,
        /// Field values as key=value or key:value pairs (e.g., name="John Doe" age=30 active=true)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        fields: Vec<String>,
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
        /// JSON or YAML data for updates (if not provided, opens TUI form)
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
    /// Import data from JSON or YAML file
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
        SchemaActions::Sync { collection, samples } => {
            println!("🔍 Discovering schema for collection '{}'...", collection);
            match json_manager.discover_and_save_schema(&collection, samples).await {
                Ok(schema) => {
                    println!("✅ Schema discovered and saved to Firestore");
                    println!("📊 Collection: {}", schema.collection_name);
                    println!("📝 Fields found: {}", schema.fields.len());
                    println!("📄 Documents analyzed: {}", schema.total_documents);
                    println!("📅 Last updated: {}", schema.last_updated);
                },
                Err(e) => {
                    println!("❌ Failed to discover and save schema: {}", e);
                    return Err(e);
                }
            }
        }
        SchemaActions::List => {
            println!("📋 Listing all schemas stored in Firestore...");
            match json_manager.list_firestore_schemas().await {
                Ok(schemas) => {
                    if schemas.is_empty() {
                        println!("📭 No schemas found in Firestore");
                        println!("💡 Use 'schema sync -c <collection>' to discover and save schemas");
                    } else {
                        println!("Found {} schema(s):\n", schemas.len());
                        for schema in schemas {
                            println!("🗂️  Collection: {}", schema.collection_name);
                            println!("   📊 Fields: {}", schema.fields.len());
                            println!("   📄 Documents: {}", schema.total_documents);
                            println!("   📅 Updated: {}", schema.last_updated);
                            println!("   🔧 Source: {}", schema.discovery_source);
                            println!();
                        }
                    }
                },
                Err(e) => {
                    println!("❌ Failed to list schemas: {}", e);
                    return Err(e);
                }
            }
        }
        SchemaActions::Show { collection } => {
            println!("📖 Loading schema for collection '{}'...", collection);
            match json_manager.load_schema_from_firestore(&collection).await {
                Ok(Some(schema)) => {
                    println!("✅ Schema found for collection '{}'", collection);
                    println!("📊 Collection: {}", schema.collection_name);
                    println!("📝 Version: {}", schema.version);
                    println!("📅 Last updated: {}", schema.last_updated);
                    println!("📄 Total documents: {}", schema.total_documents);
                    println!("🔧 Discovery source: {}", schema.discovery_source);
                    if let Some(desc) = &schema.description {
                        println!("📋 Description: {}", desc);
                    }
                    println!("\n🏷️  Fields ({}):", schema.fields.len());
                    println!("{}", "─".repeat(60));
                    
                    use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};
                    let mut table = Table::new();
                    table.set_content_arrangement(ContentArrangement::Dynamic);
                    table.set_header(vec![
                        Cell::new("Field").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Required").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Description").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Sample Values").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    ]);
                    
                    for field in &schema.fields {
                        let required_str = if field.required { "Yes" } else { "No" };
                        let desc = field.description.as_deref().unwrap_or("-");
                        let samples = if field.sample_values.is_empty() {
                            "-".to_string()
                        } else {
                            field.sample_values.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                        };
                        
                        table.add_row(vec![
                            Cell::new(&field.name).fg(Color::Yellow),
                            Cell::new(&field.field_type).fg(Color::Green),
                            Cell::new(required_str).fg(if field.required { Color::Red } else { Color::Grey }),
                            Cell::new(desc),
                            Cell::new(samples).fg(Color::Blue),
                        ]);
                    }
                    
                    println!("{}", table);
                    
                    if !schema.validation_rules.is_empty() {
                        println!("\n✅ Validation Rules:");
                        for rule in &schema.validation_rules {
                            println!("  • {}: {}", rule.field, rule.rule_type);
                        }
                    }
                    
                    if !schema.indexes.is_empty() {
                        println!("\n🔍 Indexes:");
                        for index in &schema.indexes {
                            let fields: Vec<String> = index.fields.iter()
                                .map(|f| format!("{} ({})", f.field_path, f.order))
                                .collect();
                            println!("  • {}{}", fields.join(", "), if index.unique { " (unique)" } else { "" });
                        }
                    }
                },
                Ok(None) => {
                    println!("❌ No schema found for collection '{}'", collection);
                    println!("💡 Use 'schema sync -c {}' to discover and save the schema", collection);
                },
                Err(e) => {
                    println!("❌ Failed to load schema: {}", e);
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
        DataActions::Create { collection, id, json, interactive, fields } => {
            // Check if user is asking for help
            if fields.len() == 1 && fields[0].to_lowercase() == "help" {
                // Show collection-specific help
                show_collection_help(&collection, json_manager, collection_manager).await?;
                return Ok(());
            }
            
            let data = if !fields.is_empty() {
                // Parse field arguments
                println!("📝 Creating document from field arguments...");
                let parsed_data = parse_field_arguments(&fields)?;
                add_timestamps_to_document(parsed_data)
            } else if interactive || json.is_none() {
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
                    Some(data) => add_timestamps_to_document(data),
                    None => {
                        println!("❌ Document creation cancelled");
                        return Ok(());
                    }
                }
            } else {
                let data = parse_json_or_yaml(&json.unwrap())?;
                add_timestamps_to_document(data)
            };
            
            // Validate against stored schema if available
            println!("🔍 Validating against stored schema...");
            match json_manager.validate_against_schema(&collection, &data).await {
                Ok(errors) => {
                    if !errors.is_empty() {
                        println!("⚠️ Validation warnings:");
                        for error in &errors {
                            println!("  • {}", error);
                        }
                        println!("📝 Proceeding with creation despite warnings...");
                    } else {
                        println!("✅ Data validates against schema");
                    }
                },
                Err(_) => {
                    // Schema validation failed (e.g., no schema found) - continue anyway
                    println!("💡 No schema found for validation, proceeding...");
                }
            }
            
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
                    println!("{}", serde_yaml::to_string(&data).unwrap_or_else(|e| format!("Error serializing to YAML: {}", e)));
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
                    Some(data) => add_updated_timestamp(data),
                    None => {
                        println!("❌ Document update cancelled");
                        return Ok(());
                    }
                }
            } else {
                let data = parse_json_or_yaml(&json.unwrap())?;
                add_updated_timestamp(data)
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
                    "yaml" => {
                        for (i, (doc_id, data)) in documents.iter().enumerate() {
                            println!("# Document {}: {}", i + 1, doc_id);
                            println!("{}", serde_yaml::to_string(data).unwrap_or_else(|e| format!("Error serializing to YAML: {}", e)));
                            println!();
                        }
                    }
                    "text" => {
                        for (i, (doc_id, data)) in documents.iter().enumerate() {
                            println!("{}. {} - {} fields", i + 1, doc_id, count_fields(data));
                        }
                    }
                    _ => {
                        println!("❌ Unsupported format '{}'. Use: table, json, yaml, or text", format);
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
    
    // Get the raw text first to debug JSON structure
    let response_text = response.text().await?;
    
    // Try to parse the JSON and provide better error messages
    let list_response: ListResponse = match serde_json::from_str(&response_text) {
        Ok(response) => response,
        Err(e) => {
            eprintln!("JSON parsing error: {}", e);
            eprintln!("Response JSON (first 1000 chars):");
            eprintln!("{}", &response_text[..response_text.len().min(1000)]);
            return Err(FirebaseError::DatabaseError(format!("JSON parsing failed: {}", e)));
        }
    };
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
        FirestoreValue::NullValue(_) => Ok(serde_json::Value::Null),
        FirestoreValue::TimestampValue(ts) => Ok(serde_json::Value::String(ts.clone())),
        FirestoreValue::ArrayValue { values } => {
            let mut json_array = Vec::new();
            for val in values {
                json_array.push(firestore_value_to_json_value(val)?);
            }
            Ok(serde_json::Value::Array(json_array))
        }
        FirestoreValue::MapValue { fields } => {
            let mut json_map = serde_json::Map::new();
            for (key, val) in fields {
                json_map.insert(key.clone(), firestore_value_to_json_value(val)?);
            }
            Ok(serde_json::Value::Object(json_map))
        }
        FirestoreValue::Unknown => Ok(serde_json::Value::String("(unknown value type)".to_string())),
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

// Parse field arguments like "name=John Doe" "age=30" "active=true"
fn parse_field_arguments(fields: &[String]) -> Result<serde_json::Value, FirebaseError> {
    let mut map = serde_json::Map::new();
    
    for field_arg in fields {
        // Support both = and : separators
        let (key, value_str) = if let Some((k, v)) = field_arg.split_once('=') {
            (k.trim(), v.trim())
        } else if let Some((k, v)) = field_arg.split_once(':') {
            (k.trim(), v.trim())
        } else {
            return Err(FirebaseError::ValidationError(
                format!("Invalid field format '{}'. Use 'key=value' or 'key:value'", field_arg)
            ));
        };
        
        if key.is_empty() {
            return Err(FirebaseError::ValidationError(
                format!("Empty field name in '{}'", field_arg)
            ));
        }
        
        // Parse value with type inference
        let value = parse_field_value_with_inference(value_str)?;
        map.insert(key.to_string(), value);
    }
    
    if map.is_empty() {
        return Err(FirebaseError::ValidationError(
            "No valid fields provided".to_string()
        ));
    }
    
    Ok(serde_json::Value::Object(map))
}

// Add automatic timestamps for created_at and updated_at if not provided
fn add_automatic_timestamps(map: &mut serde_json::Map<String, serde_json::Value>) {
    let current_time = chrono::Utc::now().to_rfc3339();
    
    // Add created_at if not provided
    if !map.contains_key("created_at") {
        map.insert("created_at".to_string(), serde_json::Value::String(current_time.clone()));
    }
    
    // Add updated_at if not provided (or update existing one)
    map.insert("updated_at".to_string(), serde_json::Value::String(current_time));
}

// Add automatic timestamps to any JSON value (for all input methods)
fn add_timestamps_to_document(mut data: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(ref mut map) = data {
        add_automatic_timestamps(map);
    }
    data
}

// Add only updated_at timestamp for update operations
fn add_updated_timestamp(mut data: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(ref mut map) = data {
        let current_time = chrono::Utc::now().to_rfc3339();
        map.insert("updated_at".to_string(), serde_json::Value::String(current_time));
    }
    data
}

// Parse JSON or YAML content into serde_json::Value
fn parse_json_or_yaml(content: &str) -> Result<serde_json::Value, FirebaseError> {
    // First try JSON
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
        return Ok(json_value);
    }
    
    // If JSON fails, try YAML
    match serde_yaml::from_str::<serde_json::Value>(content) {
        Ok(yaml_value) => Ok(yaml_value),
        Err(yaml_err) => {
            // If both fail, return more helpful error
            let json_err = serde_json::from_str::<serde_json::Value>(content).unwrap_err();
            Err(FirebaseError::ValidationError(format!(
                "Failed to parse as JSON: {}. Failed to parse as YAML: {}", 
                json_err, yaml_err
            )))
        }
    }
}

// Parse field value with automatic type inference
fn parse_field_value_with_inference(value_str: &str) -> Result<serde_json::Value, FirebaseError> {
    let trimmed = value_str.trim();
    
    // Handle empty values
    if trimmed.is_empty() {
        return Ok(serde_json::Value::String(String::new()));
    }
    
    // Handle null values
    if trimmed.eq_ignore_ascii_case("null") || trimmed.eq_ignore_ascii_case("none") {
        return Ok(serde_json::Value::Null);
    }
    
    // Handle boolean values
    if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("yes") {
        return Ok(serde_json::Value::Bool(true));
    }
    if trimmed.eq_ignore_ascii_case("false") || trimmed.eq_ignore_ascii_case("no") {
        return Ok(serde_json::Value::Bool(false));
    }
    
    // Handle JSON arrays and objects
    if (trimmed.starts_with('[') && trimmed.ends_with(']')) || 
       (trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return serde_json::from_str(trimmed)
            .map_err(|e| FirebaseError::ValidationError(
                format!("Invalid JSON in '{}': {}", trimmed, e)
            ));
    }
    
    // Handle quoted strings (remove quotes)
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) ||
       (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2) {
        let unquoted = &trimmed[1..trimmed.len()-1];
        return Ok(serde_json::Value::String(unquoted.to_string()));
    }
    
    // Try to parse as number
    if let Ok(int_val) = trimmed.parse::<i64>() {
        return Ok(serde_json::Value::Number(serde_json::Number::from(int_val)));
    }
    
    if let Ok(float_val) = trimmed.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return Ok(serde_json::Value::Number(num));
        }
    }
    
    // Handle timestamp patterns (ISO 8601-like)
    if trimmed.len() >= 19 && 
       (trimmed.contains('T') || trimmed.contains(' ')) &&
       (trimmed.contains('-') || trimmed.contains(':')) {
        // Try parsing as timestamp
        if let Ok(_) = chrono::DateTime::parse_from_rfc3339(trimmed) {
            return Ok(serde_json::Value::String(trimmed.to_string()));
        }
        // Try other common timestamp formats
        if let Ok(_) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
            return Ok(serde_json::Value::String(trimmed.to_string()));
        }
    }
    
    // Handle "now" as current timestamp
    if trimmed.eq_ignore_ascii_case("now") {
        return Ok(serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
    }
    
    // Default to string
    Ok(serde_json::Value::String(trimmed.to_string()))
}

// Show collection-specific help for create command
async fn show_collection_help(
    collection_name: &str,
    json_manager: &JsonSchemaManager,
    collection_manager: &CollectionManager,
) -> Result<(), FirebaseError> {
    use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};
    
    println!("📚 Collection Help: '{}'", collection_name);
    println!("{}", "=".repeat(60));
    println!();
    
    // Try to get schema from both sources: defined schemas and discovered schemas
    let mut has_schema = false;
    
    // First, check if there's a manually defined schema
    let defined_schemas = json_manager.get_schemas();
    let defined_schema = defined_schemas.iter()
        .find(|s| s.name == collection_name);
    
    if let Some(schema) = defined_schema {
        has_schema = true;
        println!("📋 Defined Schema Fields:");
        println!("{}", "-".repeat(40));
        
        let mut table = Table::new();
        table.set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec![
            Cell::new("Field").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Required").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Default").add_attribute(Attribute::Bold).fg(Color::Cyan),
            Cell::new("Description").add_attribute(Attribute::Bold).fg(Color::Cyan),
        ]);
        
        for field in &schema.fields {
            let required_str = if field.required { "Yes ✓" } else { "No" };
            let default_str = field.default_value
                .as_ref()
                .map(|v| format_json_value_compact(v))
                .unwrap_or_else(|| "-".to_string());
            let description = field.description
                .as_ref()
                .map(|d| d.clone())
                .unwrap_or_else(|| "-".to_string());
            
            table.add_row(vec![
                Cell::new(&field.name).fg(Color::Yellow),
                Cell::new(&field.field_type).fg(Color::Green),
                Cell::new(required_str).fg(if field.required { Color::Red } else { Color::Grey }),
                Cell::new(default_str),
                Cell::new(description),
            ]);
        }
        
        println!("{}", table);
        println!();
        
        // Show validation rules if any
        if !schema.validation_rules.is_empty() {
            println!("✅ Validation Rules:");
            for rule in &schema.validation_rules {
                println!("  • {}: {:?}", rule.field, rule.rule_type);
            }
            println!();
        }
    }
    
    // Also try to discover from actual data
    match collection_manager.describe_collection(collection_name, 10).await {
        Ok(discovered_schema) => {
            if !has_schema || discovered_schema.total_documents > 0 {
                println!("🔍 Discovered Fields (from {} documents):", discovered_schema.total_documents);
                println!("{}", "-".repeat(40));
                
                if discovered_schema.fields.is_empty() {
                    println!("  No fields discovered yet. Collection may be empty.");
                } else {
                    let mut table = Table::new();
                    table.set_content_arrangement(ContentArrangement::Dynamic);
                    table.set_header(vec![
                        Cell::new("Field").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Found In").add_attribute(Attribute::Bold).fg(Color::Cyan),
                        Cell::new("Sample Values").add_attribute(Attribute::Bold).fg(Color::Cyan),
                    ]);
                    
                    for field in &discovered_schema.fields {
                        let frequency_str = format!("{}/{} docs", 
                            field.frequency, 
                            discovered_schema.total_documents
                        );
                        
                        let samples = if field.sample_values.len() > 3 {
                            format!("{}, ...", field.sample_values[..3].join(", "))
                        } else {
                            field.sample_values.join(", ")
                        };
                        
                        table.add_row(vec![
                            Cell::new(&field.name).fg(Color::Yellow),
                            Cell::new(&field.field_type).fg(Color::Green),
                            Cell::new(&frequency_str).fg(
                                if field.is_required { Color::Red } else { Color::Grey }
                            ),
                            Cell::new(&samples),
                        ]);
                    }
                    
                    println!("{}", table);
                    println!();
                }
                has_schema = true;
            }
        }
        Err(_) => {
            if !has_schema {
                println!("⚠️  No schema found for collection '{}'", collection_name);
                println!("   The collection may not exist or may be empty.");
                println!();
            }
        }
    }
    
    // Show usage examples
    println!("📝 Usage Examples:");
    println!("{}", "-".repeat(40));
    println!();
    
    println!("Create with field arguments:");
    println!("  cargo run --bin firebase-cli data create -c {} \\", collection_name);
    
    if has_schema {
        // Show example based on discovered or defined fields
        if let Some(schema) = defined_schema {
            let mut example_fields = Vec::new();
            for (i, field) in schema.fields.iter().enumerate() {
                if i >= 3 { break; } // Show max 3 fields in example
                
                let example_value = match field.field_type.as_str() {
                    "string" | "String" => format!("{}=\"example\"", field.name),
                    "integer" | "Integer" => format!("{}=123", field.name),
                    "number" | "Number" | "float" | "Float" => format!("{}=45.67", field.name),
                    "boolean" | "Boolean" => format!("{}=true", field.name),
                    "timestamp" | "Timestamp" => format!("{}=now", field.name),
                    "array" | "Array" => format!("{}='[\"item1\",\"item2\"]'", field.name),
                    "object" | "Object" | "map" | "Map" => format!("{}='{{\"key\":\"value\"}}'", field.name),
                    _ => format!("{}=\"value\"", field.name),
                };
                example_fields.push(format!("    {}", example_value));
            }
            
            if !example_fields.is_empty() {
                println!("{}", example_fields.join(" \\\n"));
            } else {
                println!("    field1=\"value1\" field2=123 field3=true");
            }
        } else {
            println!("    field1=\"value1\" field2=123 field3=true");
        }
    } else {
        println!("    name=\"Example\" value=123 active=true");
    }
    
    println!();
    println!("Create with JSON:");
    println!("  cargo run --bin firebase-cli data create -c {} \\", collection_name);
    println!("    -j '{{\"field1\":\"value\",\"field2\":123}}'");
    println!();
    
    println!("Create with YAML:");
    println!("  cargo run --bin firebase-cli data create -c {} \\", collection_name);
    println!("    -j 'field1: value");
    println!("         field2: 123'");
    println!();
    
    println!("Create with interactive TUI form:");
    println!("  cargo run --bin firebase-cli data create -c {}", collection_name);
    println!();
    
    // Show field format help
    println!("💡 Field Format Tips:");
    println!("{}", "-".repeat(40));
    println!("  • Strings: name=\"John Doe\" or name='Jane Smith'");
    println!("  • Numbers: age=30 or price=29.99");
    println!("  • Booleans: active=true or enabled=false");
    println!("  • Arrays: tags='[\"tag1\",\"tag2\"]'");
    println!("  • Objects: data='{{\"key\":\"value\"}}'");
    println!("  • Timestamps: created_at=now or date=\"2024-01-01T00:00:00Z\"");
    println!("  • Null: optional=null");
    println!();
    
    println!("🤖 Auto-Generated Fields:");
    println!("  • created_at: Automatically added with current timestamp if not provided");
    println!("  • updated_at: Always set to current timestamp on create/update");
    println!("  💡 You don't need to supply these fields - they're handled automatically!");
    println!();
    
    println!("📖 For more help:");
    println!("  cargo run --bin firebase-cli data create --help");
    println!();
    
    Ok(())
}

fn format_json_value_compact(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => {
            if s.len() > 20 {
                format!("\"{}...\"", &s[..17])
            } else {
                format!("\"{}\"", s)
            }
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
        serde_json::Value::Object(obj) => format!("{{{} fields}}", obj.len()),
        serde_json::Value::Null => "null".to_string(),
    }
}