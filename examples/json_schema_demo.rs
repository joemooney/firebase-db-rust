use firebase_db::{FirebaseClient, JsonSchemaManager, User, FirebaseError};
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
    let mut json_manager = JsonSchemaManager::new(client);
    
    println!("JSON Schema & Data Management Demo");
    println!("===================================\n");
    
    // 1. Create example schema file
    println!("1. CREATING EXAMPLE SCHEMA FILE");
    println!("--------------------------------");
    json_manager.create_example_schema_file("demo-schema.json")?;
    println!("✅ Created demo-schema.json with example collections");
    
    let schema_content = std::fs::read_to_string("demo-schema.json")?;
    println!("📄 Schema preview (first 500 chars):");
    println!("{}", &schema_content[..schema_content.len().min(500)]);
    if schema_content.len() > 500 {
        println!("... (truncated)");
    }
    println!();
    
    // 2. Import schema from JSON
    println!("2. IMPORTING SCHEMA FROM JSON");
    println!("------------------------------");
    json_manager.import_schema_from_file("demo-schema.json")?;
    println!("✅ Schema imported successfully");
    
    // Validate some test data
    let valid_user = User::new("Alice Johnson".to_string(), "alice@example.com".to_string(), 25);
    match json_manager.get_schema_manager().validate("users", &valid_user) {
        Ok(_) => println!("✅ Valid user passed schema validation"),
        Err(e) => println!("❌ User validation failed: {}", e),
    }
    
    let invalid_user = User::new("Bob".to_string(), "not-an-email".to_string(), 12);
    match json_manager.get_schema_manager().validate("users", &invalid_user) {
        Ok(_) => println!("❌ Invalid user should have failed validation"),
        Err(e) => println!("✅ Invalid user correctly rejected: {}", e),
    }
    println!();
    
    // 3. Create some test data
    println!("3. CREATING TEST DATA");
    println!("---------------------");
    
    let test_users = vec![
        User::new("Alice Johnson".to_string(), "alice@example.com".to_string(), 28),
        User::new("Bob Smith".to_string(), "bob@example.com".to_string(), 34),
        User::new("Carol Davis".to_string(), "carol@example.com".to_string(), 22),
        User::new("David Wilson".to_string(), "david@example.com".to_string(), 45),
    ];
    
    for user in &test_users {
        let doc_id = json_manager.get_client().create("users", user).await?;
        println!("Created user: {} (ID: {})", user.name, doc_id);
    }
    println!();
    
    // 4. Export data to JSON
    println!("4. EXPORTING DATA TO JSON");
    println!("--------------------------");
    let count = json_manager.export_collection_data::<User>("users", "users_export.json").await?;
    println!("✅ Exported {} users to users_export.json", count);
    
    let export_content = std::fs::read_to_string("users_export.json")?;
    println!("📄 Export file preview (first 300 chars):");
    println!("{}", &export_content[..export_content.len().min(300)]);
    if export_content.len() > 300 {
        println!("... (truncated)");
    }
    println!();
    
    // 5. Clean up original data
    println!("5. CLEANING UP ORIGINAL DATA");
    println!("-----------------------------");
    let users_to_delete: Vec<User> = json_manager.get_client().list("users", None).await?;
    for user in users_to_delete {
        if let Some(id) = &user.id {
            json_manager.get_client().delete("users", id).await?;
            println!("Deleted user: {}", user.name);
        }
    }
    println!("✅ All original users deleted");
    println!();
    
    // 6. Import data from JSON
    println!("6. IMPORTING DATA FROM JSON");
    println!("----------------------------");
    let imported_count = json_manager.import_collection_data::<User>("users_export.json", Some("users")).await?;
    println!("✅ Imported {} users from JSON file", imported_count);
    
    // Verify the import
    let restored_users: Vec<User> = json_manager.get_client().list("users", None).await?;
    println!("📋 Restored users:");
    for user in &restored_users {
        println!("  - {} ({}) - Age: {}", user.name, user.email, user.age);
    }
    println!();
    
    // 7. Full backup demonstration
    println!("7. FULL BACKUP DEMONSTRATION");
    println!("-----------------------------");
    let backup_results = json_manager.backup_all_data("demo_backup").await?;
    let total_backed_up: usize = backup_results.values().sum();
    println!("✅ Backup completed!");
    println!("📊 Backup summary:");
    for (collection, count) in backup_results {
        println!("  - {}: {} items", collection, count);
    }
    println!("  - Total: {} items", total_backed_up);
    println!();
    
    // 8. Export schema for version control
    println!("8. EXPORTING SCHEMA FOR VERSION CONTROL");
    println!("----------------------------------------");
    json_manager.export_schema_to_file("production-schema.json")?;
    println!("✅ Schema exported to production-schema.json");
    println!("💡 This file can be committed to version control");
    println!();
    
    // 9. JSON Schema file structure explanation
    println!("9. JSON SCHEMA STRUCTURE");
    println!("------------------------");
    println!("The JSON schema file contains:");
    println!("  ✓ Collections with field definitions");
    println!("  ✓ Data types (string, integer, boolean, etc.)");
    println!("  ✓ Validation rules (min/max, email format, etc.)");
    println!("  ✓ Index definitions for query optimization");
    println!("  ✓ Field descriptions for documentation");
    println!();
    
    println!("10. CLI USAGE EXAMPLES");
    println!("----------------------");
    println!("You can also use the CLI tool:");
    println!("  cargo run --bin firebase-cli schema example");
    println!("  cargo run --bin firebase-cli schema import -i schema.json");
    println!("  cargo run --bin firebase-cli data export -c users -o users.json");
    println!("  cargo run --bin firebase-cli data import -i users.json");
    println!("  cargo run --bin firebase-cli data backup -d backup_folder");
    println!("  cargo run --bin firebase-cli data list -c users");
    println!();
    
    // Cleanup demo files
    println!("🧹 CLEANING UP DEMO FILES");
    println!("-------------------------");
    let cleanup_files = vec![
        "demo-schema.json",
        "users_export.json", 
        "production-schema.json"
    ];
    
    for file in cleanup_files {
        if std::path::Path::new(file).exists() {
            std::fs::remove_file(file).ok();
            println!("Removed {}", file);
        }
    }
    
    // Remove backup directory
    if std::path::Path::new("demo_backup").exists() {
        std::fs::remove_dir_all("demo_backup").ok();
        println!("Removed demo_backup directory");
    }
    
    // Clean up test data
    let final_users: Vec<User> = json_manager.get_client().list("users", None).await?;
    for user in final_users {
        if let Some(id) = &user.id {
            json_manager.get_client().delete("users", id).await?;
        }
    }
    println!("Cleaned up test data");
    
    println!("\n🎉 JSON Schema & Data Management Demo Completed!");
    println!("================================================");
    println!("Key capabilities demonstrated:");
    println!("  ✅ JSON schema definition and validation");
    println!("  ✅ Data export to JSON files");
    println!("  ✅ Data import from JSON files");
    println!("  ✅ Full database backup");
    println!("  ✅ Schema version control");
    println!("  ✅ CLI tools for operations");
    
    Ok(())
}