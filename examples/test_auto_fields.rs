use firebase_db::collections::{CollectionSchema, FieldInfo, AutoFieldType};
use firebase_db::tui_form::TuiForm;

fn main() {
    println!("Testing Automatic Field Generation\n");
    
    // Create a mock schema with automatic fields
    let schema = CollectionSchema {
        collection_name: "users".to_string(),
        total_documents: 100,
        sample_document: None,
        fields: vec![
            // Regular user fields
            FieldInfo {
                name: "name".to_string(),
                field_type: "string".to_string(),
                is_required: true,
                sample_values: vec!["John Doe".to_string(), "Jane Smith".to_string()],
                frequency: 100,
                unique_values: 100,
                auto_field: None,
            },
            FieldInfo {
                name: "email".to_string(),
                field_type: "string".to_string(),
                is_required: true,
                sample_values: vec!["john@example.com".to_string()],
                frequency: 100,
                unique_values: 100,
                auto_field: None,
            },
            FieldInfo {
                name: "age".to_string(),
                field_type: "integer".to_string(),
                is_required: false,
                sample_values: vec!["25".to_string(), "30".to_string()],
                frequency: 80,
                unique_values: 50,
                auto_field: None,
            },
            // Automatic fields that should be detected and generated
            FieldInfo {
                name: "created_at".to_string(),
                field_type: "timestamp".to_string(), // Will be set to timestamp by detection
                is_required: true,
                sample_values: vec!["2024-01-01T12:00:00Z".to_string()],
                frequency: 100,
                unique_values: 100,
                auto_field: Some(AutoFieldType::CreatedAt),
            },
            FieldInfo {
                name: "updated_at".to_string(),
                field_type: "timestamp".to_string(),
                is_required: true,
                sample_values: vec!["2024-01-01T12:00:00Z".to_string()],
                frequency: 100,
                unique_values: 100,
                auto_field: Some(AutoFieldType::UpdatedAt),
            },
            FieldInfo {
                name: "id".to_string(),
                field_type: "string".to_string(),
                is_required: true,
                sample_values: vec!["550e8400-e29b-41d4-a716-446655440000".to_string()],
                frequency: 100,
                unique_values: 100,
                auto_field: Some(AutoFieldType::RandomUuid),
            },
            FieldInfo {
                name: "user_id".to_string(),
                field_type: "string".to_string(),
                is_required: false,
                sample_values: vec!["system-user".to_string()],
                frequency: 50,
                unique_values: 10,
                auto_field: Some(AutoFieldType::UserId),
            },
        ],
    };
    
    // Create form from schema
    let form = TuiForm::from_schema("users", &schema);
    
    println!("Schema Analysis:");
    println!("Total fields in schema: {}", schema.fields.len());
    
    let auto_fields: Vec<_> = schema.fields.iter().filter(|f| f.auto_field.is_some()).collect();
    let manual_fields: Vec<_> = schema.fields.iter().filter(|f| f.auto_field.is_none()).collect();
    
    println!("Automatic fields (will be generated): {}", auto_fields.len());
    for field in &auto_fields {
        if let Some(auto_type) = &field.auto_field {
            println!("  {} ({}) - {}", field.name, field.field_type, auto_type.description());
        }
    }
    
    println!("\nManual fields (user will input): {}", manual_fields.len());
    for field in &manual_fields {
        println!("  {} ({}){}", 
            field.name, 
            field.field_type,
            if field.is_required { " *required" } else { "" }
        );
    }
    
    println!("\nForm fields (only manual fields shown): {}", form.fields.len());
    for field in &form.fields {
        println!("  {} ({}){}", 
            field.name, 
            field.field_type,
            if field.required { " *required" } else { "" }
        );
    }
    
    // Test document generation
    println!("\n=== Testing Document Generation ===");
    
    // Simulate filling out the form
    let mut test_form = form;
    test_form.fields[0].value = "Alice Johnson".to_string(); // name
    test_form.fields[1].value = "alice@example.com".to_string(); // email
    test_form.fields[2].value = "28".to_string(); // age
    
    match test_form.to_json() {
        Ok(json) => {
            println!("✅ Generated document successfully!");
            println!("Document JSON:");
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
            
            // Check that automatic fields were added
            let obj = json.as_object().unwrap();
            println!("\nField Analysis:");
            println!("• Manual fields: {} values", test_form.fields.len());
            println!("• Automatic fields: {} values", obj.len() - test_form.fields.len());
            println!("• Total document fields: {} values", obj.len());
            
            // Check specific automatic fields
            if obj.contains_key("created_at") {
                println!("✅ created_at was auto-generated");
            }
            if obj.contains_key("updated_at") {
                println!("✅ updated_at was auto-generated");
            }
            if obj.contains_key("id") {
                println!("✅ id (UUID) was auto-generated");
            }
            if obj.contains_key("user_id") {
                println!("✅ user_id was auto-generated");
            }
        }
        Err(e) => {
            println!("❌ Failed to generate document: {}", e);
        }
    }
    
    println!("\n=== Automatic Field Type Demonstrations ===");
    for auto_type in [
        AutoFieldType::CreatedAt,
        AutoFieldType::UpdatedAt,
        AutoFieldType::RandomUuid,
        AutoFieldType::SequenceNumber,
        AutoFieldType::RandomNumber,
        AutoFieldType::UserId,
    ] {
        let value = auto_type.generate_value();
        println!("{:20} -> {}", auto_type.description(), value);
    }
}