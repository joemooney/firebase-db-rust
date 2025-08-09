# Firebase CRUD Operations CLI

Complete Create, Read, Update, Delete operations with interactive TUI forms for beautiful data management.

## Overview

The Firebase CLI now includes comprehensive CRUD operations with both command-line JSON input and interactive TUI (Terminal User Interface) forms. The system intelligently discovers your collection schemas and creates dynamic forms for intuitive data entry.

## Key Features

### 🖥️ Interactive TUI Forms
- **Schema Discovery**: Automatically analyzes existing collections to build smart forms
- **Field Type Detection**: Recognizes strings, integers, booleans, arrays, objects, and timestamps  
- **Visual Form Interface**: Terminal-based forms with navigation, validation, and help text
- **Pre-filled Updates**: When updating, forms are pre-populated with current values

### 📊 Multiple Output Formats
- **JSON**: Full structured data output
- **Table**: Beautiful formatted tables with color coding
- **YAML**: Human-readable key-value display

### ✅ Smart Validation
- **Type Checking**: Validates data types during form input
- **Required Fields**: Identifies and enforces required fields based on frequency analysis
- **Error Handling**: Clear error messages with helpful suggestions

## Commands

### CREATE - Adding New Documents

#### Command Line JSON
```bash
# Basic creation with auto-generated ID
cargo run --bin firebase-cli data create -c users -j '{"name":"John Doe","age":30,"active":true}'

# Creation with specific document ID  
cargo run --bin firebase-cli data create -c users -i user_123 -j '{"name":"Jane Smith","email":"jane@example.com"}'

# Complex nested data
cargo run --bin firebase-cli data create -c users -j '{
  "name": "Alice Johnson",
  "preferences": {"theme": "dark", "notifications": true},
  "skills": ["rust", "firebase", "cli"]
}'
```

#### Interactive TUI Form
```bash
# Opens intelligent form based on collection schema
cargo run --bin firebase-cli data create -c users

# Force interactive mode even with JSON provided
cargo run --bin firebase-cli data create -c users -j '{"name":"Bob"}' --interactive
```

**TUI Form Features:**
- ↑/↓ - Navigate between fields
- Enter - Edit field value  
- Tab - Move to next field
- Ctrl+S - Save and submit
- Ctrl+C/Esc - Cancel
- Auto-discovers field types from existing data
- Shows field descriptions and sample values
- Validates input as you type

### READ - Retrieving Documents

#### Basic Reading
```bash
# Default JSON format
cargo run --bin firebase-cli data read -c users -i user_123

# Pretty table format
cargo run --bin firebase-cli data read -c users -i user_123 --format table

# YAML-like format  
cargo run --bin firebase-cli data read -c users -i user_123 --format yaml
```

**Output Examples:**

JSON Format:
```json
{
  "name": "John Doe",
  "age": 30,
  "active": true,
  "created_at": "2024-01-01T00:00:00Z"
}
```

Table Format:
```
📄 Document: user_123
+------------+-------------------------+---------+
| Field      | Value                   | Type    |
+============================================+
| name       | "John Doe"             | string  |
|------------|-------------------------+---------|
| age        | 30                     | integer |
|------------|-------------------------+---------|
| active     | true                   | boolean |
|------------|-------------------------+---------|
| created_at | "2024-01-01T00:00:00Z" | string  |
+------------+-------------------------+---------+
```

### UPDATE - Modifying Documents

#### Command Line JSON
```bash
# Merge update (default) - only updates specified fields
cargo run --bin firebase-cli data update -c users -i user_123 -j '{"age":31,"last_updated":"2024-01-15T10:00:00Z"}'

# Replace entire document (overwrites all fields)
cargo run --bin firebase-cli data update -c users -i user_123 --replace -j '{"name":"New Name","age":25}'
```

#### Interactive TUI Form
```bash
# Opens form pre-filled with current document values
cargo run --bin firebase-cli data update -c users -i user_123

# Force interactive mode
cargo run --bin firebase-cli data update -c users -i user_123 -j '{"age":32}' --interactive
```

**Update Modes:**
- **Merge (default)**: Only updates fields you specify, preserves others
- **Replace**: Overwrites entire document with new data

### DELETE - Removing Documents

```bash
# With confirmation prompt
cargo run --bin firebase-cli data delete -c users -i user_123

# Skip confirmation (dangerous!)
cargo run --bin firebase-cli data delete -c users -i user_123 --yes
```

**Safety Features:**
- Interactive confirmation by default
- Clear warning messages
- `--yes` flag to skip confirmation for automation

### LIST - Browsing Collections

#### Table Format (Default)
```bash
# Beautiful table with all fields
cargo run --bin firebase-cli data list -c users

# Limit results
cargo run --bin firebase-cli data list -c users --limit 10
```

**Table Output Example:**
```
📊 Collection: users
+----------+----------+-----+-------------+--------+
| ID       | name     | age | email       | active |
+=================================================+
| user_123 | John Doe | 30  | john@ex.com | true   |
|----------+----------+-----+-------------+--------|
| user_456 | Jane S.  | 28  | jane@ex.com | false  |
+----------+----------+-----+-------------+--------+
Total: 2 documents
```

#### Other Formats
```bash
# JSON format - full document data
cargo run --bin firebase-cli data list -c users --format json

# Text format - simple summary
cargo run --bin firebase-cli data list -c users --format text
```

## Advanced Features

### Schema-Based TUI Forms

The CLI analyzes your existing collection data to create intelligent forms:

```bash
# First, it discovers your collection schema
cargo run --bin firebase-cli collections describe -c users

# Then creates forms based on discovered patterns
cargo run --bin firebase-cli data create -c users
```

**Form Intelligence:**
- **Field Types**: Detects string, integer, boolean, array, object, timestamp fields
- **Required Fields**: Marks fields as required based on frequency (80%+ = required)
- **Sample Values**: Shows examples from existing data
- **Validation**: Real-time type checking and format validation

### Field Type Support

The TUI forms support all Firebase/Firestore data types:

| Type | Input Format | Example |
|------|-------------|---------|
| String | Text input | `"John Doe"` |
| Integer | Number input | `30` |  
| Number | Decimal input | `29.99` |
| Boolean | true/false/yes/no/1/0 | `true` |
| Array | JSON array | `["tag1", "tag2"]` |
| Object | JSON object | `{"key": "value"}` |
| Timestamp | ISO 8601 or "now" | `2024-01-01T00:00:00Z` |

### Error Handling

Comprehensive error handling with helpful messages:

```
❌ Validation Error: Invalid integer: "abc" 
💡 Tip: Enter a whole number like 42

❌ Document Not Found: User 'missing_user' not found in collection 'users'
💡 Tip: Check the document ID and try again

❌ Invalid JSON: Unexpected character '}' at position 15
💡 Tip: Use proper JSON format: {"key": "value"}
```

## Integration Examples

### With Shell Scripts
```bash
#!/bin/bash
# Batch create users
for user in alice bob charlie; do
    cargo run --bin firebase-cli data create -c users -j "{\"name\":\"$user\",\"status\":\"new\"}"
done
```

### With Data Processing
```bash
# Export data for processing
cargo run --bin firebase-cli data export -c users -o users.json

# Process data...

# Import processed data
cargo run --bin firebase-cli data import -i processed_users.json -c users
```

## Best Practices

### 1. Use Interactive Forms for Complex Data
- For documents with many fields
- When you need to see field types and validation
- For exploratory data entry

### 2. Use JSON for Automation
- In shell scripts and CI/CD pipelines
- For programmatic data creation
- When you know the exact data structure

### 3. Choose the Right Output Format
- **Table**: Great for browsing and comparing documents
- **JSON**: Perfect for processing and automation  
- **YAML**: Ideal for human reading and debugging

### 4. Schema Discovery Workflow
```bash
# 1. Analyze collection to understand structure
cargo run --bin firebase-cli collections describe -c users

# 2. Create schema-based documents  
cargo run --bin firebase-cli data create -c users

# 3. Export schema for version control
cargo run --bin firebase-cli schema export -o users-schema.json
```

## Troubleshooting

### Common Issues

**1. TUI Form Not Opening**
```bash
# Check if collection exists and has data for schema discovery
cargo run --bin firebase-cli collections list
```

**2. Validation Errors**
```bash
# Check field types in existing documents
cargo run --bin firebase-cli collections describe -c collection_name
```

**3. Permission Errors**
```bash
# Verify Firebase credentials
echo $FIREBASE_PROJECT_ID
echo $FIREBASE_API_KEY
```

**4. Terminal Display Issues**
- Ensure terminal supports color output
- Try resizing terminal window
- Check terminal compatibility with ratatui

## Security Considerations

1. **Confirmation Prompts**: Delete operations require confirmation by default
2. **Data Validation**: All inputs are validated before submission
3. **Error Messages**: Don't expose sensitive internal information
4. **Credentials**: Environment variables keep API keys secure

## Performance Tips

1. **Limit Large Collections**: Use `--limit` flag for large datasets
2. **Schema Caching**: Schema discovery results are cached within sessions
3. **Batch Operations**: Use export/import for bulk data management
4. **Efficient Queries**: Consider using the query commands for complex filtering

---

The Firebase CRUD CLI provides a powerful, user-friendly interface for managing your Firebase data with both command-line efficiency and interactive ease-of-use.