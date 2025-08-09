use firebase_db::{FirebaseClient, CollectionManager, User, FirebaseError};
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), FirebaseError> {
    dotenv().ok();
    
    let project_id = env::var("FIREBASE_PROJECT_ID")
        .map_err(|_| FirebaseError::ConfigError("FIREBASE_PROJECT_ID not set".to_string()))?;
    
    let api_key = env::var("FIREBASE_API_KEY")
        .map_err(|_| FirebaseError::ConfigError("FIREBASE_API_KEY not set".to_string()))?;
    
    let client = FirebaseClient::new(project_id, api_key);
    let collection_manager = CollectionManager::new(client);
    
    println!("Firebase Collection Management Demo");
    println!("===================================\n");
    
    // 1. List all collections
    println!("1. LISTING COLLECTIONS");
    println!("----------------------");
    println!("Discovering collections in your Firebase project...\n");
    
    match collection_manager.list_collections().await {
        Ok(collections) => {
            if collections.is_empty() {
                println!("❌ No collections found or all collections are empty\n");
            } else {
                // Display in table format
                let table_output = collection_manager.format_collections_table(&collections, true);
                println!("📊 Collections (Table Format):");
                println!("{}\n", table_output);
                
                // Display in text format
                let text_output = collection_manager.format_collections_table(&collections, false);
                println!("📋 Collections (Text Format):");
                println!("{}", text_output);
                
                println!("📈 Total collections found: {}\n", collections.len());
            }
        }
        Err(e) => {
            println!("❌ Error listing collections: {}\n", e);
        }
    }
    
    // 2. Get detailed info for each collection
    println!("2. COLLECTION DETAILS");
    println!("---------------------");
    
    let test_collections = vec!["users", "posts", "comments"];
    
    for collection_name in &test_collections {
        println!("📊 Info for collection '{}':", collection_name);
        
        match collection_manager.get_collection_info(collection_name).await {
            Ok(info) => {
                println!("  ✅ Collection: {}", info.name);
                println!("     Documents: {}", info.document_count);
                println!("     Estimated Size: {}", info.estimated_size);
                if let Some(last_modified) = info.last_modified {
                    println!("     Last Modified: {}", last_modified);
                } else {
                    println!("     Last Modified: Unknown");
                }
            }
            Err(FirebaseError::NotFound(_)) => {
                println!("  ❌ Collection '{}' not found or is empty", collection_name);
            }
            Err(e) => {
                println!("  ❌ Error: {}", e);
            }
        }
        println!();
    }
    
    // 3. Describe schema for existing collections
    println!("3. SCHEMA ANALYSIS");
    println!("------------------");
    
    // Find a collection with data
    match collection_manager.list_collections().await {
        Ok(collections) => {
            if let Some(collection) = collections.first() {
                let collection_name = &collection.name;
                println!("🔍 Analyzing schema for collection '{}'...\n", collection_name);
                
                match collection_manager.describe_collection(collection_name, 10).await {
                    Ok(schema) => {
                        // Display in table format
                        println!("📊 Schema Analysis (Table Format):");
                        let table_output = collection_manager.format_schema_table(&schema, true);
                        println!("{}\n", table_output);
                        
                        // Display in text format  
                        println!("📋 Schema Analysis (Text Format):");
                        let text_output = collection_manager.format_schema_table(&schema, false);
                        println!("{}\n", text_output);
                        
                        // Show field statistics
                        println!("📈 Schema Statistics:");
                        println!("  - Total Documents Analyzed: {}", schema.total_documents);
                        println!("  - Total Fields: {}", schema.fields.len());
                        
                        let required_fields = schema.fields.iter().filter(|f| f.is_required).count();
                        let optional_fields = schema.fields.len() - required_fields;
                        
                        println!("  - Required Fields: {}", required_fields);
                        println!("  - Optional Fields: {}", optional_fields);
                        
                        // Field type breakdown
                        let mut type_counts = std::collections::HashMap::new();
                        for field in &schema.fields {
                            *type_counts.entry(&field.field_type).or_insert(0) += 1;
                        }
                        
                        println!("  - Field Type Breakdown:");
                        for (field_type, count) in type_counts {
                            println!("    * {}: {} fields", field_type, count);
                        }
                        println!();
                    }
                    Err(e) => {
                        println!("❌ Error analyzing schema: {}\n", e);
                    }
                }
            } else {
                println!("❌ No collections available for schema analysis\n");
            }
        }
        Err(e) => {
            println!("❌ Error getting collections: {}\n", e);
        }
    }
    
    // 4. Demonstrate CLI usage
    println!("4. CLI USAGE EXAMPLES");
    println!("---------------------");
    println!("You can use the CLI tool for collection management:");
    println!();
    println!("📋 List Collections:");
    println!("  cargo run --bin firebase-cli collections list");
    println!("  cargo run --bin firebase-cli collections list -f text");
    println!();
    println!("🔍 Describe Collection Schema:");
    println!("  cargo run --bin firebase-cli collections describe -c users");
    println!("  cargo run --bin firebase-cli collections describe -c users -s 100 -f text");
    println!();
    println!("📊 Get Collection Info:");
    println!("  cargo run --bin firebase-cli collections info -c users");
    println!();
    
    // 5. Feature summary
    println!("5. FEATURE SUMMARY");
    println!("------------------");
    println!("✅ Collection Discovery: Automatically finds collections with data");
    println!("✅ Schema Analysis: Analyzes field types, requirements, and sample values");
    println!("✅ Table Formatting: Beautiful table output with colors and formatting");
    println!("✅ Text Output: Simple text format for scripts and automation");
    println!("✅ Statistics: Document counts, size estimates, and field analysis");
    println!("✅ CLI Integration: Easy-to-use command-line interface");
    println!("✅ Error Handling: Graceful handling of missing collections and errors");
    println!();
    
    // 6. Use cases
    println!("6. COMMON USE CASES");
    println!("-------------------");
    println!("🔍 Database Exploration:");
    println!("  - Discover what collections exist in your Firebase project");
    println!("  - Understand the structure of your data");
    println!("  - Get quick statistics about document counts and sizes");
    println!();
    println!("📊 Schema Documentation:");
    println!("  - Generate documentation for your database schema");
    println!("  - Understand field types and requirements");
    println!("  - See sample values to understand data patterns");
    println!();
    println!("🛠️ Development & Debugging:");
    println!("  - Quickly check if collections have the expected structure");
    println!("  - Verify data types and field presence");
    println!("  - Monitor collection growth and changes");
    println!();
    println!("📋 Data Migration & Analysis:");
    println!("  - Assess data before migration");
    println!("  - Understand schema differences between environments");
    println!("  - Plan data transformations based on current structure");
    println!();
    
    println!("🎉 Collection Management Demo Completed!");
    println!("========================================");
    println!("Your Firebase Rust client now includes powerful collection");
    println!("management and schema analysis capabilities!");
    
    Ok(())
}