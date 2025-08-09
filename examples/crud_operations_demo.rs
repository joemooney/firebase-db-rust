use firebase_db::{FirebaseClient, TuiForm, CollectionManager, FirebaseError};
use dotenv::dotenv;
use std::env;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), FirebaseError> {
    dotenv().ok();
    
    let project_id = env::var("FIREBASE_PROJECT_ID")
        .map_err(|_| FirebaseError::ConfigError("FIREBASE_PROJECT_ID not set".to_string()))?;
    
    let api_key = env::var("FIREBASE_API_KEY")
        .map_err(|_| FirebaseError::ConfigError("FIREBASE_API_KEY not set".to_string()))?;
    
    let client = FirebaseClient::new(project_id, api_key);
    let collection_manager = CollectionManager::new(client.clone());
    
    println!("🚀 Firebase CRUD Operations Demo");
    println!("================================\n");
    
    let collection_name = "demo_users";
    
    // 1. CREATE: Demonstrate creating documents with different data types
    println!("1. 📝 CREATE Operations");
    println!("------------------------");
    
    // Create with JSON directly
    let user_data = json!({
        "name": "Alice Johnson",
        "email": "alice@example.com",
        "age": 28,
        "active": true,
        "preferences": {
            "theme": "dark",
            "notifications": true
        },
        "tags": ["developer", "remote", "full-stack"]
    });
    
    println!("📄 Creating user with complex data structure...");
    let user_id = client.create_document(collection_name, None, user_data).await?;
    println!("✅ Created user with ID: {}\n", user_id);
    
    // Create with specific ID
    let admin_data = json!({
        "name": "Bob Smith",
        "email": "bob@example.com",
        "age": 35,
        "role": "admin",
        "active": true,
        "last_login": "2024-01-01T00:00:00Z"
    });
    
    println!("📄 Creating admin user with specific ID...");
    let admin_id = "admin_bob";
    client.create_document(collection_name, Some(admin_id.to_string()), admin_data).await?;
    println!("✅ Created admin with ID: {}\n", admin_id);
    
    // 2. READ: Demonstrate reading documents
    println!("2. 📖 READ Operations");
    println!("----------------------");
    
    println!("🔍 Reading user document...");
    let user_doc = client.get_document(collection_name, &user_id).await?;
    println!("📄 User data:");
    println!("{}\n", serde_json::to_string_pretty(&user_doc)?);
    
    println!("🔍 Reading admin document...");
    let admin_doc = client.get_document(collection_name, admin_id).await?;
    println!("📄 Admin data:");
    println!("{}\n", serde_json::to_string_pretty(&admin_doc)?);
    
    // 3. UPDATE: Demonstrate updating documents
    println!("3. 🔄 UPDATE Operations");
    println!("-----------------------");
    
    // Partial update (merge mode)
    let update_data = json!({
        "age": 29,
        "last_updated": "2024-01-15T10:00:00Z",
        "preferences": {
            "theme": "light"
        }
    });
    
    println!("🔄 Updating user with partial data (merge mode)...");
    client.update_document(collection_name, &user_id, update_data, true).await?;
    println!("✅ User updated successfully\n");
    
    // Read updated document
    println!("📖 Reading updated user document...");
    let updated_user = client.get_document(collection_name, &user_id).await?;
    println!("📄 Updated user data:");
    println!("{}\n", serde_json::to_string_pretty(&updated_user)?);
    
    // 4. COLLECTION ANALYSIS: Demonstrate schema discovery
    println!("4. 🔍 COLLECTION ANALYSIS");
    println!("--------------------------");
    
    println!("📊 Analyzing collection schema...");
    match collection_manager.describe_collection(collection_name, 10).await {
        Ok(schema) => {
            println!("📋 Collection: {}", schema.collection_name);
            println!("📈 Total documents: {}", schema.total_documents);
            println!("🏷️ Fields discovered:");
            
            for field in &schema.fields {
                println!("  • {} ({}): {} values, required: {}", 
                    field.name, 
                    field.field_type, 
                    field.sample_values.len(),
                    field.is_required
                );
                
                if !field.sample_values.is_empty() {
                    let sample = if field.sample_values.len() > 3 {
                        format!("{}, ...", field.sample_values[..3].join(", "))
                    } else {
                        field.sample_values.join(", ")
                    };
                    println!("    Sample values: {}", sample);
                }
            }
            println!();
        }
        Err(e) => {
            println!("❌ Error analyzing collection: {}\n", e);
        }
    }
    
    // 5. TUI FORM DEMO: Show how forms would work
    println!("5. 🖥️ TUI FORM DEMONSTRATION");
    println!("-----------------------------");
    
    // Simulate what a TUI form would create
    let form_data = json!({
        "name": "Charlie Wilson",
        "email": "charlie@example.com",
        "age": 42,
        "department": "Engineering",
        "skills": ["rust", "firebase", "cli-tools"]
    });
    
    println!("🖥️ Simulating TUI form submission...");
    println!("📝 Form data that would be created:");
    println!("{}", serde_json::to_string_pretty(&form_data)?);
    
    let form_user_id = client.create_document(collection_name, None, form_data).await?;
    println!("✅ TUI form simulation completed. Created user: {}\n", form_user_id);
    
    // 6. LIST AND QUERY: Show listing capabilities
    println!("6. 📋 LIST AND QUERY Operations");
    println!("--------------------------------");
    
    println!("📄 All documents in collection (simplified view):");
    // Note: In a real application, you'd implement a generic list method
    // For this demo, we'll show the concept
    
    let sample_doc_ids = vec![&user_id, admin_id, &form_user_id];
    for doc_id in sample_doc_ids {
        match client.get_document(collection_name, doc_id).await {
            Ok(doc) => {
                if let Some(name) = doc.get("name").and_then(|n| n.as_str()) {
                    println!("  📄 {}: {}", doc_id, name);
                }
            }
            Err(_) => println!("  ❌ Failed to read {}", doc_id),
        }
    }
    println!();
    
    // 7. DELETE: Clean up demo data
    println!("7. 🗑️ DELETE Operations (Cleanup)");
    println!("----------------------------------");
    
    let cleanup_ids = vec![&user_id, admin_id.to_string(), &form_user_id];
    
    for doc_id in cleanup_ids {
        println!("🗑️ Deleting document: {}", doc_id);
        match client.delete_document(collection_name, &doc_id).await {
            Ok(_) => println!("✅ Deleted successfully"),
            Err(e) => println!("❌ Delete failed: {}", e),
        }
    }
    println!();
    
    // 8. CLI COMMANDS SUMMARY
    println!("8. 🛠️ CLI COMMANDS SUMMARY");
    println!("---------------------------");
    println!("The Firebase CLI now supports full CRUD operations:");
    println!();
    
    println!("📝 CREATE:");
    println!("  # With JSON data");
    println!("  cargo run --bin firebase-cli data create -c users -j '{{\"name\":\"John\",\"age\":30}}'");
    println!("  ");
    println!("  # With interactive TUI form (discovers schema automatically)");
    println!("  cargo run --bin firebase-cli data create -c users");
    println!("  ");
    println!("  # With specific document ID");
    println!("  cargo run --bin firebase-cli data create -c users -i user_123 -j '{{\"name\":\"Jane\"}}'");
    println!();
    
    println!("📖 READ:");
    println!("  # JSON format (default)");
    println!("  cargo run --bin firebase-cli data read -c users -i user_123");
    println!("  ");
    println!("  # Table format (pretty display)");
    println!("  cargo run --bin firebase-cli data read -c users -i user_123 --format table");
    println!("  ");
    println!("  # YAML-like format");
    println!("  cargo run --bin firebase-cli data read -c users -i user_123 --format yaml");
    println!();
    
    println!("🔄 UPDATE:");
    println!("  # With JSON data (merge mode by default)");
    println!("  cargo run --bin firebase-cli data update -c users -i user_123 -j '{{\"age\":31}}'");
    println!("  ");
    println!("  # With interactive TUI form (pre-filled with current values)");
    println!("  cargo run --bin firebase-cli data update -c users -i user_123");
    println!("  ");
    println!("  # Replace entire document (not merge)");
    println!("  cargo run --bin firebase-cli data update -c users -i user_123 --replace -j '{{\"name\":\"New Name\"}}'");
    println!();
    
    println!("🗑️ DELETE:");
    println!("  # With confirmation prompt");
    println!("  cargo run --bin firebase-cli data delete -c users -i user_123");
    println!("  ");
    println!("  # Skip confirmation");
    println!("  cargo run --bin firebase-cli data delete -c users -i user_123 --yes");
    println!();
    
    println!("📋 LIST:");
    println!("  # Table format (default)");
    println!("  cargo run --bin firebase-cli data list -c users");
    println!("  ");
    println!("  # JSON format");
    println!("  cargo run --bin firebase-cli data list -c users --format json");
    println!("  ");
    println!("  # Limit results");
    println!("  cargo run --bin firebase-cli data list -c users --limit 10");
    println!();
    
    println!("9. 🎯 KEY FEATURES");
    println!("------------------");
    println!("✅ **Interactive TUI Forms**: Auto-discovering schemas for intuitive data entry");
    println!("✅ **Multiple Output Formats**: JSON, Table, and YAML display options");
    println!("✅ **Smart Schema Detection**: Analyzes existing data to build forms");
    println!("✅ **Type Validation**: Validates field types during form input");
    println!("✅ **Merge vs Replace**: Choose between updating specific fields or replacing documents");
    println!("✅ **Flexible Input**: Support both command-line JSON and interactive forms");
    println!("✅ **Beautiful Tables**: Rich, formatted table output with color coding");
    println!("✅ **Safety Features**: Confirmation prompts for destructive operations");
    println!();
    
    println!("🎉 CRUD Operations Demo Completed!");
    println!("===================================");
    println!("Your Firebase CLI now has full CRUD capabilities with an intuitive");
    println!("TUI interface for interactive data entry and beautiful table outputs!");
    
    Ok(())
}