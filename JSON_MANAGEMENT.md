# JSON Schema & Data Management

This Firebase Rust client provides comprehensive JSON-based schema management and data import/export capabilities.

## Features

### 📋 Schema Management
- **JSON Schema Definition**: Define collections, fields, and validation rules in JSON
- **Schema Import/Export**: Load schemas from files and export to JSON
- **Schema Validation**: Validate data against defined schemas
- **Version Control**: Keep schema files in git for change tracking

### 📊 Data Management
- **Data Export**: Export collection data to JSON files
- **Data Import**: Import data from JSON files into collections
- **Full Backup**: Backup all collections to JSON files
- **Data Migration**: Move data between environments

## CLI Usage

The project includes a CLI tool for easy data operations:

```bash
# Schema operations
cargo run --bin firebase-cli schema example -o my-schema.json
cargo run --bin firebase-cli schema import -i my-schema.json
cargo run --bin firebase-cli schema export -o current-schema.json
cargo run --bin firebase-cli schema validate -f my-schema.json

# Data operations  
cargo run --bin firebase-cli data export -c users -o users.json
cargo run --bin firebase-cli data import -i users.json -c users
cargo run --bin firebase-cli data backup -d backup_folder
cargo run --bin firebase-cli data list -c users -l 10
```

## JSON Schema Format

### Example Schema File
```json
{
  "version": "1.0.0",
  "collections": {
    "users": {
      "name": "users",
      "description": "User accounts",
      "fields": [
        {
          "name": "name",
          "field_type": "string",
          "required": true,
          "description": "User's full name"
        },
        {
          "name": "email", 
          "field_type": "string",
          "required": true,
          "description": "User's email address"
        },
        {
          "name": "age",
          "field_type": "integer", 
          "required": true,
          "default_value": 18,
          "description": "User's age"
        },
        {
          "name": "active",
          "field_type": "boolean",
          "required": false,
          "default_value": true,
          "description": "Account status"
        }
      ],
      "indexes": [
        {
          "fields": [
            {"field_path": "email", "order": "asc"}
          ],
          "unique": true,
          "description": "Unique email index"
        }
      ],
      "validation_rules": [
        {
          "field": "email",
          "rule_type": "email",
          "description": "Must be valid email format"
        },
        {
          "field": "age", 
          "rule_type": "min",
          "value": 13,
          "description": "Minimum age 13"
        },
        {
          "field": "name",
          "rule_type": "min_length", 
          "value": 2,
          "description": "Name must be at least 2 characters"
        }
      ]
    }
  }
}
```

### Field Types
- `string` - Text data
- `integer` - Whole numbers  
- `double` - Decimal numbers
- `boolean` - True/false values
- `timestamp` - Date/time values
- `array` - Lists of values
- `map` - Nested objects
- `reference` - References to other documents

### Validation Rules
- `email` - Email format validation
- `url` - URL format validation  
- `min` / `max` - Numeric range validation
- `min_length` / `max_length` - String length validation
- `regex` - Regular expression matching
- `custom` - Custom validation expressions

## Programmatic Usage

### Schema Management
```rust
use firebase_db::{FirebaseClient, JsonSchemaManager};

let client = FirebaseClient::new(project_id, api_key);
let mut json_manager = JsonSchemaManager::new(client);

// Create example schema file
json_manager.create_example_schema_file("schema.json")?;

// Import schema from file
json_manager.import_schema_from_file("schema.json")?;

// Export current schema 
json_manager.export_schema_to_file("current-schema.json")?;

// Validate data against schema
let user = User::new("John".to_string(), "john@example.com".to_string(), 25);
json_manager.get_schema_manager().validate("users", &user)?;
```

### Data Export/Import
```rust
// Export collection data
let count = json_manager.export_collection_data::<User>("users", "users.json").await?;
println!("Exported {} users", count);

// Import collection data
let count = json_manager.import_collection_data::<User>("users.json", Some("users")).await?;
println!("Imported {} users", count);

// Full backup
let results = json_manager.backup_all_data("backup_dir").await?;
for (collection, count) in results {
    println!("Backed up {} items from {}", count, collection);
}
```

## Data Export Format

### Example Export File
```json
{
  "collection": "users",
  "exported_at": "2024-01-15T10:30:00Z",
  "count": 3,
  "data": [
    {
      "name": "John Doe",
      "email": "john@example.com", 
      "age": 30,
      "active": true,
      "created_at": "2024-01-15T09:00:00Z"
    },
    {
      "name": "Jane Smith",
      "email": "jane@example.com",
      "age": 28, 
      "active": true,
      "created_at": "2024-01-15T09:15:00Z"
    }
  ]
}
```

## Workflow Examples

### 1. Development to Production Migration
```bash
# Export schema from development
cargo run --bin firebase-cli schema export -o dev-schema.json

# Review and modify schema
# ... edit dev-schema.json ...

# Import schema in production
cargo run --bin firebase-cli schema import -i dev-schema.json

# Export test data
cargo run --bin firebase-cli data export -c users -o test-users.json

# Import to production (after review)
cargo run --bin firebase-cli data import -i test-users.json
```

### 2. Regular Backups
```bash
# Create timestamped backup
BACKUP_DIR="backup_$(date +%Y%m%d_%H%M%S)"
cargo run --bin firebase-cli data backup -d "$BACKUP_DIR"

# Archive backup
tar -czf "${BACKUP_DIR}.tar.gz" "$BACKUP_DIR"
```

### 3. Schema Version Control
```bash
# Export current schema
cargo run --bin firebase-cli schema export -o schema_v2.json

# Commit to git
git add schema_v2.json
git commit -m "Update schema to v2 - added user preferences"

# Deploy to production
cargo run --bin firebase-cli schema import -i schema_v2.json
```

## Best Practices

### Schema Management
- **Version your schemas**: Include version numbers in filenames
- **Document changes**: Use descriptive commit messages
- **Test schemas**: Validate before deploying to production
- **Backup before changes**: Always backup data before schema updates

### Data Operations
- **Validate imports**: Check data format before importing
- **Use transactions**: For large imports, consider batch operations
- **Monitor size**: Large exports may take time and use bandwidth
- **Regular backups**: Automate backups for production systems

### Security
- **Restrict access**: Limit who can import/export data
- **Sanitize data**: Review exported data for sensitive information
- **Environment separation**: Use different credentials for dev/prod
- **Audit trails**: Log all import/export operations

## Limitations

- **Large datasets**: Very large collections may require streaming
- **Schema enforcement**: Validation is client-side only
- **Concurrent access**: No locking during import operations
- **Data types**: Limited to JSON-compatible types
- **Relationships**: No foreign key constraints

## Error Handling

Common errors and solutions:

- **File not found**: Check file paths are correct
- **Permission denied**: Verify Firebase security rules
- **Invalid schema**: Use `schema validate` to check syntax
- **Import failures**: Check data format matches schema
- **Network issues**: Implement retry logic for large operations