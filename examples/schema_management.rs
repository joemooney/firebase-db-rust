use firebase_db::{
    FirebaseClient, FirebaseError, User, FirestoreValue,
    SchemaManager, Collection, Field, FieldType, Index, IndexField, IndexOrder,
    ValidationRule, ValidationRuleType,
    SecurityRules, RuleBuilder, Expression
};
use dotenv::dotenv;
use std::env;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), FirebaseError> {
    dotenv().ok();
    
    let project_id = env::var("FIREBASE_PROJECT_ID")
        .map_err(|_| FirebaseError::ConfigError("FIREBASE_PROJECT_ID not set".to_string()))?;
    
    let api_key = env::var("FIREBASE_API_KEY")
        .map_err(|_| FirebaseError::ConfigError("FIREBASE_API_KEY not set".to_string()))?;
    
    let client = FirebaseClient::new(project_id.clone(), api_key);
    
    println!("Firestore Schema Management Example");
    println!("===================================\n");
    
    // 1. Define Schema
    println!("1. DEFINING SCHEMA");
    println!("------------------");
    
    let mut schema_manager = SchemaManager::new(client.clone());
    
    // Define Users collection schema
    let users_collection = Collection {
        name: "users".to_string(),
        fields: vec![
            Field {
                name: "id".to_string(),
                field_type: FieldType::String,
                required: false,
                default_value: None,
                description: Some("User ID".to_string()),
            },
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                required: true,
                default_value: None,
                description: Some("User's full name".to_string()),
            },
            Field {
                name: "email".to_string(),
                field_type: FieldType::String,
                required: true,
                default_value: None,
                description: Some("User's email address".to_string()),
            },
            Field {
                name: "age".to_string(),
                field_type: FieldType::Integer,
                required: true,
                default_value: Some(FirestoreValue::IntegerValue("18".to_string())),
                description: Some("User's age".to_string()),
            },
            Field {
                name: "created_at".to_string(),
                field_type: FieldType::Timestamp,
                required: true,
                default_value: None,
                description: Some("Account creation timestamp".to_string()),
            },
        ],
        indexes: vec![
            Index {
                fields: vec![
                    IndexField {
                        field_path: "email".to_string(),
                        order: IndexOrder::Ascending,
                    }
                ],
                unique: true,
            },
            Index {
                fields: vec![
                    IndexField {
                        field_path: "age".to_string(),
                        order: IndexOrder::Ascending,
                    },
                    IndexField {
                        field_path: "created_at".to_string(),
                        order: IndexOrder::Descending,
                    }
                ],
                unique: false,
            },
        ],
        validation_rules: vec![
            ValidationRule {
                field: "email".to_string(),
                rule: ValidationRuleType::Email,
            },
            ValidationRule {
                field: "age".to_string(),
                rule: ValidationRuleType::Min(18.0),
            },
            ValidationRule {
                field: "age".to_string(),
                rule: ValidationRuleType::Max(120.0),
            },
            ValidationRule {
                field: "name".to_string(),
                rule: ValidationRuleType::MinLength(2),
            },
            ValidationRule {
                field: "name".to_string(),
                rule: ValidationRuleType::MaxLength(100),
            },
        ],
    };
    
    schema_manager.define_collection(users_collection);
    println!("✓ Users collection schema defined");
    
    // Define Posts collection schema
    let posts_collection = Collection {
        name: "posts".to_string(),
        fields: vec![
            Field {
                name: "title".to_string(),
                field_type: FieldType::String,
                required: true,
                default_value: None,
                description: Some("Post title".to_string()),
            },
            Field {
                name: "content".to_string(),
                field_type: FieldType::String,
                required: true,
                default_value: None,
                description: Some("Post content".to_string()),
            },
            Field {
                name: "author_id".to_string(),
                field_type: FieldType::String,
                required: true,
                default_value: None,
                description: Some("Reference to user ID".to_string()),
            },
            Field {
                name: "tags".to_string(),
                field_type: FieldType::Array,
                required: false,
                default_value: Some(FirestoreValue::ArrayValue { values: vec![] }),
                description: Some("Post tags".to_string()),
            },
            Field {
                name: "published".to_string(),
                field_type: FieldType::Boolean,
                required: true,
                default_value: Some(FirestoreValue::BooleanValue(false)),
                description: Some("Publication status".to_string()),
            },
        ],
        indexes: vec![
            Index {
                fields: vec![
                    IndexField {
                        field_path: "author_id".to_string(),
                        order: IndexOrder::Ascending,
                    },
                    IndexField {
                        field_path: "published".to_string(),
                        order: IndexOrder::Ascending,
                    }
                ],
                unique: false,
            },
        ],
        validation_rules: vec![
            ValidationRule {
                field: "title".to_string(),
                rule: ValidationRuleType::MinLength(5),
            },
            ValidationRule {
                field: "title".to_string(),
                rule: ValidationRuleType::MaxLength(200),
            },
            ValidationRule {
                field: "content".to_string(),
                rule: ValidationRuleType::MinLength(10),
            },
        ],
    };
    
    schema_manager.define_collection(posts_collection);
    println!("✓ Posts collection schema defined\n");
    
    // 2. Validate Data Against Schema
    println!("2. SCHEMA VALIDATION");
    println!("--------------------");
    
    // Valid user
    let valid_user = User::new(
        "John Doe".to_string(),
        "john@example.com".to_string(),
        25
    );
    
    match schema_manager.validate("users", &valid_user) {
        Ok(_) => println!("✓ Valid user passed validation"),
        Err(e) => println!("✗ Valid user failed: {}", e),
    }
    
    // Invalid user (age too young)
    let invalid_user = User::new(
        "Jane Doe".to_string(),
        "jane@example.com".to_string(),
        15  // Below minimum age of 18
    );
    
    match schema_manager.validate("users", &invalid_user) {
        Ok(_) => println!("✗ Invalid user should have failed"),
        Err(e) => println!("✓ Invalid user correctly rejected: {}", e),
    }
    
    // Invalid email
    let invalid_email_user = User::new(
        "Bob Smith".to_string(),
        "not-an-email".to_string(),  // Invalid email format
        30
    );
    
    match schema_manager.validate("users", &invalid_email_user) {
        Ok(_) => println!("✗ Invalid email should have failed"),
        Err(e) => println!("✓ Invalid email correctly rejected: {}", e),
    }
    println!();
    
    // 3. Export/Import Schema
    println!("3. SCHEMA EXPORT/IMPORT");
    println!("-----------------------");
    
    let schema_json = schema_manager.export_schema();
    println!("Schema exported ({} characters)", schema_json.len());
    
    // Save to file (in production)
    std::fs::write("schema.json", &schema_json).ok();
    println!("✓ Schema saved to schema.json\n");
    
    // 4. Generate Index Requirements
    println!("4. INDEX REQUIREMENTS");
    println!("---------------------");
    
    schema_manager.create_indexes("users").await?;
    println!();
    
    // 5. Security Rules Generation
    println!("5. SECURITY RULES");
    println!("-----------------");
    
    let mut rules = SecurityRules::new();
    
    // Users collection rules
    rules.add_rule(
        RuleBuilder::new("/users/{userId}")
            .allow_read_if(Expression::IsAuthenticated)
            .allow_write_if(Expression::And(
                Box::new(Expression::IsAuthenticated),
                Box::new(Expression::IsOwner("userId".to_string()))
            ))
            .build()
    );
    
    // Posts collection rules
    rules.add_rule(
        RuleBuilder::new("/posts/{postId}")
            .public_read()  // Anyone can read posts
            .allow_create_if(Expression::IsAuthenticated)
            .allow_update_if(Expression::IsOwner("author_id".to_string()))
            .allow_delete_if(Expression::Or(
                Box::new(Expression::IsOwner("author_id".to_string())),
                Box::new(Expression::HasRole("admin".to_string()))
            ))
            .build()
    );
    
    // Admin collection rules
    rules.add_rule(
        RuleBuilder::new("/admin/{document=**}")
            .allow_read_if(Expression::HasRole("admin".to_string()))
            .allow_write_if(Expression::HasRole("admin".to_string()))
            .build()
    );
    
    // Public collection rules
    rules.add_rule(
        RuleBuilder::new("/public/{document=**}")
            .public_read()
            .allow_write_if(Expression::HasRole("moderator".to_string()))
            .build()
    );
    
    let rules_content = rules.generate();
    println!("Generated Firestore Security Rules:");
    println!("```");
    println!("{}", rules_content);
    println!("```");
    
    // Save rules to file
    rules.export_to_file("firestore.rules")?;
    println!("✓ Security rules saved to firestore.rules\n");
    
    // 6. Initialize Collections with Metadata
    println!("6. INITIALIZE COLLECTIONS");
    println!("-------------------------");
    
    match schema_manager.initialize_collections().await {
        Ok(_) => println!("✓ Collection metadata initialized"),
        Err(e) => println!("Note: {}", e),
    }
    
    println!("\n7. SCHEMA VERSIONING");
    println!("--------------------");
    println!("Schema version: 1.0.0");
    println!("Collections defined: 2 (users, posts)");
    println!("Total fields: 10");
    println!("Total indexes: 3");
    println!("Total validation rules: 8");
    
    println!("\n✅ Schema management example completed!");
    println!("\nNOTE: To deploy these changes to Firebase:");
    println!("1. Copy firestore.rules to your Firebase project");
    println!("2. Run: firebase deploy --only firestore:rules");
    println!("3. Create indexes in Firebase Console or via CLI");
    println!("4. Use schema validation in your application code");
    
    // Clean up
    std::fs::remove_file("schema.json").ok();
    std::fs::remove_file("firestore.rules").ok();
    
    Ok(())
}