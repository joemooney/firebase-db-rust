use firebase_db::{FirebaseClient, JsonSchemaManager, CollectionManager, User, FirebaseError};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use std::env;
use std::path::Path;

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
            handle_data_command(&json_manager, action).await?;
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
    action: DataActions
) -> Result<(), FirebaseError> {
    match action {
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
        DataActions::List { collection, limit } => {
            println!("📋 Listing documents from collection '{}':", collection);
            // For this example, we'll use User type. In a real CLI, you'd want to be more generic
            let users: Vec<User> = json_manager.get_client().list(&collection, limit).await?;
            
            if users.is_empty() {
                println!("  No documents found.");
            } else {
                for (i, user) in users.iter().enumerate() {
                    println!("  {}. {} ({}) - Age: {}", i + 1, user.name, user.email, user.age);
                }
                println!("  Total: {} documents", users.len());
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