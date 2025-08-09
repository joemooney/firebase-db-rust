# Firestore Schema Management in Rust

While Firestore is schemaless by nature, this client provides comprehensive schema management capabilities to enforce structure, validation, and consistency in your application.

## Key Concepts

### 1. **Firestore is Schemaless**
- Collections and documents are created automatically on first write
- No DDL (CREATE TABLE) statements needed
- Fields can be added/removed dynamically
- No migrations required for schema changes

### 2. **Application-Level Schema Management**
This client provides tools to manage schemas at the application level:

## Features

### Schema Definition
```rust
let users_collection = Collection {
    name: "users".to_string(),
    fields: vec![
        Field {
            name: "email".to_string(),
            field_type: FieldType::String,
            required: true,
            default_value: None,
            description: Some("User email".to_string()),
        },
        // ... more fields
    ],
    indexes: vec![/* ... */],
    validation_rules: vec![/* ... */],
};

schema_manager.define_collection(users_collection);
```

### Data Validation
```rust
// Validate before saving
schema_manager.validate("users", &user)?;

// Validation rules include:
- Required fields
- Type checking
- Min/Max values
- String length
- Email format
- Regular expressions
- Custom validation
```

### Index Management
```rust
// Define composite indexes
Index {
    fields: vec![
        IndexField { field_path: "age".to_string(), order: IndexOrder::Ascending },
        IndexField { field_path: "created_at".to_string(), order: IndexOrder::Descending },
    ],
    unique: false,
}
```

### Security Rules Generation
```rust
let mut rules = SecurityRules::new();

rules.add_rule(
    RuleBuilder::new("/users/{userId}")
        .allow_read_if(Expression::IsAuthenticated)
        .allow_write_if(Expression::IsOwner("userId".to_string()))
        .build()
);

// Generate firestore.rules file
rules.export_to_file("firestore.rules")?;
```

### Schema Export/Import
```rust
// Export schema to JSON
let schema_json = schema_manager.export_schema();
std::fs::write("schema.json", schema_json)?;

// Import schema from JSON
let json = std::fs::read_to_string("schema.json")?;
schema_manager.import_schema(&json)?;
```

## Migration Strategy

Since Firestore doesn't require traditional migrations, here's how to handle schema changes:

### 1. **Adding Fields**
- Just start writing the new field
- Old documents won't have it (handle with defaults in code)

### 2. **Removing Fields**
- Stop reading/writing the field
- Optionally run batch update to remove from existing docs

### 3. **Renaming Fields**
- Add new field with new name
- Copy data from old to new field
- Remove old field references

### 4. **Changing Field Types**
- Create new field with different name
- Migrate data with type conversion
- Remove old field

### Example Migration
```rust
// Version 1: age as string
// Version 2: age as integer

// Migration code
let users: Vec<User> = client.list("/users", None).await?;
for mut user in users {
    // Convert age from string to integer
    if let Some(age_str) = user.get_field_as_string("age") {
        user.age = age_str.parse().unwrap_or(0);
        client.update("/users", &user.id, &user).await?;
    }
}
```

## Best Practices

### 1. **Version Your Schema**
```rust
const SCHEMA_VERSION: &str = "1.0.0";
```

### 2. **Use Default Values**
```rust
Field {
    name: "status".to_string(),
    field_type: FieldType::String,
    required: false,
    default_value: Some(FirestoreValue::StringValue("active".to_string())),
    // ...
}
```

### 3. **Document Collections**
```rust
Field {
    name: "user_id".to_string(),
    field_type: FieldType::String,
    required: true,
    default_value: None,
    description: Some("Reference to users collection".to_string()),
}
```

### 4. **Validate Before Write**
```rust
// Always validate
if let Err(e) = schema_manager.validate("users", &user) {
    eprintln!("Validation failed: {}", e);
    return Err(e);
}
client.create("/users", &user).await?;
```

### 5. **Keep Schema in Version Control**
```bash
# Commit schema files
git add schema.json firestore.rules
git commit -m "Update user schema"
```

## Deployment Process

1. **Define Schema in Code**
2. **Export Security Rules**: `cargo run --example schema_management`
3. **Deploy Rules**: `firebase deploy --only firestore:rules`
4. **Create Indexes**: Via Firebase Console or CLI
5. **Test Validation**: Run integration tests

## Limitations

- **Indexes**: Must be created via Firebase Console or CLI (not via API)
- **Security Rules**: Must be deployed via Firebase CLI
- **Transactions**: Schema changes aren't transactional
- **Enforcement**: Schema is enforced only in your app (not at database level)

## When to Use SQL Instead

Consider Cloud SQL (PostgreSQL/MySQL) if you need:
- Strong schema enforcement at database level
- Complex JOINs and relationships
- ACID transactions across tables
- SQL-specific features (views, stored procedures)
- Existing SQL-based tools/ORMs