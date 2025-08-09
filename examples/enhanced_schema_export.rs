use firebase_db::{FirebaseClient, JsonSchemaManager, FirebaseError};
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
    let json_manager = JsonSchemaManager::new(client);
    
    println!("Enhanced Schema Export Demo");
    println!("===========================\n");
    
    // 1. Export discovered schemas from database
    println!("1. DISCOVERED SCHEMA EXPORT");
    println!("----------------------------");
    println!("🔍 Analyzing your actual database collections...\n");
    
    match json_manager.export_discovered_schemas("discovered_schema.json").await {
        Ok(_) => {
            println!("✅ Discovered schema exported to 'discovered_schema.json'\n");
            
            // Read and display a snippet
            if let Ok(content) = std::fs::read_to_string("discovered_schema.json") {
                let preview = if content.len() > 500 {
                    format!("{}...\n(truncated - {} total characters)", &content[..500], content.len())
                } else {
                    content
                };
                println!("📄 Schema preview:");
                println!("{}\n", preview);
            }
        }
        Err(e) => {
            println!("❌ Error exporting discovered schema: {}\n", e);
        }
    }
    
    // 2. Export manually defined schemas  
    println!("2. MANUAL SCHEMA EXPORT");
    println!("-----------------------");
    println!("📝 Exporting manually defined schemas (if any)...\n");
    
    match json_manager.export_schema_to_file("manual_schema.json") {
        Ok(_) => {
            println!("✅ Manual schema exported to 'manual_schema.json'\n");
            
            if let Ok(content) = std::fs::read_to_string("manual_schema.json") {
                println!("📄 Manual schema content:");
                println!("{}\n", content);
            }
        }
        Err(e) => {
            println!("❌ Error exporting manual schema: {}\n", e);
        }
    }
    
    // 3. Comparison and Analysis
    println!("3. SCHEMA ANALYSIS");
    println!("------------------");
    
    // Load and analyze the discovered schema
    if let Ok(content) = std::fs::read_to_string("discovered_schema.json") {
        if let Ok(schema) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(collections) = schema.get("collections").and_then(|c| c.as_object()) {
                println!("📊 Discovered {} collection(s):", collections.len());
                
                for (name, collection) in collections {
                    println!("\n🗃️ Collection: {}", name);
                    
                    if let Some(description) = collection.get("description").and_then(|d| d.as_str()) {
                        println!("   Description: {}", description);
                    }
                    
                    if let Some(fields) = collection.get("fields").and_then(|f| f.as_array()) {
                        println!("   Fields: {} total", fields.len());
                        
                        let mut required_count = 0;
                        let mut type_counts = std::collections::HashMap::new();
                        
                        for field in fields {
                            if let Some(field_obj) = field.as_object() {
                                if let Some(required) = field_obj.get("required").and_then(|r| r.as_bool()) {
                                    if required {
                                        required_count += 1;
                                    }
                                }
                                
                                if let Some(field_type) = field_obj.get("field_type").and_then(|t| t.as_str()) {
                                    *type_counts.entry(field_type).or_insert(0) += 1;
                                }
                            }
                        }
                        
                        println!("   Required fields: {}/{}", required_count, fields.len());
                        println!("   Field types:");
                        for (field_type, count) in type_counts {
                            println!("     - {}: {} fields", field_type, count);
                        }
                    }
                }
            }
        }
    }
    
    println!("\n4. CLI USAGE EXAMPLES");
    println!("---------------------");
    println!("The enhanced schema export provides two modes:\n");
    
    println!("🔍 Discovered Schema Export (DEFAULT):");
    println!("  - Analyzes your actual database collections");
    println!("  - Extracts field types, requirements, and sample values");
    println!("  - Perfect for documentation and understanding existing data");
    println!("  - Commands:");
    println!("    cargo run --bin firebase-cli schema export");
    println!("    cargo run --bin firebase-cli schema export -o my-db-schema.json");
    println!();
    
    println!("📝 Manual Schema Export:");
    println!("  - Exports schemas you've manually defined in code");
    println!("  - Useful for planned schemas before data exists");
    println!("  - Commands:");
    println!("    cargo run --bin firebase-cli schema export --manual -o manual-only.json");
    println!();
    
    println!("5. USE CASES");
    println!("------------");
    println!("✅ **Database Documentation**: Generate comprehensive docs of your current schema");
    println!("✅ **Migration Planning**: Understand structure before moving data");
    println!("✅ **Team Onboarding**: Share schema files with new developers");
    println!("✅ **Schema Evolution**: Track how your schema changes over time");
    println!("✅ **Validation Setup**: Use discovered schemas to set up validation rules");
    println!("✅ **Cross-Environment Sync**: Compare schemas between dev/staging/prod");
    println!();
    
    println!("6. DISCOVERED SCHEMA FEATURES");
    println!("-----------------------------");
    println!("The discovered schema export provides rich information:");
    println!("  📋 Field Types: Actual types found in your data");
    println!("  ✅ Required/Optional: Based on field frequency across documents");
    println!("  💡 Sample Values: Real examples from your database");
    println!("  📊 Frequency Stats: How often fields appear (e.g., '18/20 documents')");
    println!("  📝 Descriptions: Auto-generated based on analysis");
    println!("  🔍 Mixed Types: Detection of fields with inconsistent types");
    println!();
    
    println!("7. WORKFLOW INTEGRATION");
    println!("-----------------------");
    println!("📋 **Development Workflow**:");
    println!("  1. Export discovered schema: cargo run --bin firebase-cli schema export");
    println!("  2. Review and understand your data structure");
    println!("  3. Commit schema.json to version control");
    println!("  4. Use for documentation and team coordination");
    println!();
    
    println!("🔄 **Schema Evolution**:");
    println!("  1. Regularly export schemas to track changes");
    println!("  2. Compare schema files to see evolution");
    println!("  3. Plan migrations based on discovered structures");
    println!("  4. Validate new data against discovered patterns");
    println!();
    
    // Cleanup
    println!("🧹 CLEANUP");
    println!("----------");
    std::fs::remove_file("discovered_schema.json").ok();
    std::fs::remove_file("manual_schema.json").ok();
    println!("✅ Cleaned up demo files\n");
    
    println!("🎉 Enhanced Schema Export Demo Completed!");
    println!("==========================================");
    println!("Your schema export now intelligently discovers and documents");
    println!("your actual database structure - perfect for understanding,");
    println!("documenting, and working with real Firebase data!");
    
    Ok(())
}