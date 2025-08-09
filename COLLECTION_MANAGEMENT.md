# Collection Management & Schema Analysis

Your Firebase Rust client now includes powerful collection management and schema analysis capabilities with beautiful table formatting.

## Features

### 📋 Collection Discovery
- **List Collections**: Automatically discover collections in your Firebase project
- **Collection Info**: Get detailed statistics about document counts and sizes
- **Smart Detection**: Only shows collections that contain data

### 🔍 Schema Analysis  
- **Field Discovery**: Analyze field types, requirements, and patterns
- **Sample Values**: See example values for each field
- **Type Analysis**: Understand data types and mixed-type fields
- **Frequency Stats**: See how often fields appear across documents

### 🎨 Beautiful Output
- **Table Format**: Colored, formatted tables using `comfy-table`
- **Text Format**: Simple text output for scripting and automation
- **Both CLI and Programmatic**: Use via command line or in your code

## CLI Usage

### List Collections
```bash
# Table format (default) - colored, formatted tables
cargo run --bin firebase-cli collections list

# Text format - simple output for scripts
cargo run --bin firebase-cli collections list -f text
```

**Output:**
```
📊 Firebase Collections

+-----------------------+-----------+-----------+---------------------+
| Collection            | Documents | Est. Size | Last Modified       |
+=====================================================================+
| users                 | 22        | 44.0KB    | 2025-08-09T20:41:03 |
|-----------------------+-----------+-----------+---------------------|
| _metadata_collections | 2         | 4.0KB     | 2025-08-09T21:46:41 |
+-----------------------+-----------+-----------+---------------------+

📈 Summary: Found 2 collection(s)
```

### Describe Collection Schema
```bash
# Analyze collection schema (default: 50 documents sample)
cargo run --bin firebase-cli collections describe -c users

# Analyze with custom sample size and text format
cargo run --bin firebase-cli collections describe -c users -s 100 -f text
```

**Output:**
```
📊 Collection: users (22 documents)

+------------+-----------+----------+-----------+--------------------------------------------------------------------------------+
| Field Name | Type      | Required | Frequency | Sample Values                                                                  |
+================================================================================================================================+
| name       | string    | ✓        | 22/22     | "Diana Prince", "Charlie Brown", "David Wilson", ... (5 total)                 |
|------------+-----------+----------+-----------+--------------------------------------------------------------------------------|
| email      | string    | ✓        | 22/22     | "alice@example.com", "diana@example.com", "charlie@example.com", ... (5 total) |
|------------+-----------+----------+-----------+--------------------------------------------------------------------------------|
| age        | integer   | ✓        | 22/22     | 25, 34, 28, ... (5 total)                                                      |
|------------+-----------+----------+-----------+--------------------------------------------------------------------------------|
| created_at | timestamp | ✓        | 22/22     | 2024-01-01T00:00:00Z                                                           |
|------------+-----------+----------+-----------+--------------------------------------------------------------------------------|
| updated_at | timestamp | ✓        | 22/22     | 2024-01-01T00:00:00Z                                                           |
+------------+-----------+----------+-----------+--------------------------------------------------------------------------------+

📄 Sample Document:
{
  "age": 25,
  "created_at": "2025-08-09T20:41:03.227390Z",
  "email": "alice@example.com",
  "name": "Alice Smith", 
  "updated_at": "2025-08-09T20:41:03.227390Z"
}
```

### Get Collection Info
```bash
# Quick stats for a specific collection
cargo run --bin firebase-cli collections info -c users
```

**Output:**
```
📊 Getting info for collection 'users'...
Collection: users
Documents: 22
Estimated Size: 44.0KB
Last Modified: 2025-08-09T20:41:03.594991Z
```

## Programmatic Usage

### Collection Discovery
```rust
use firebase_db::{FirebaseClient, CollectionManager};

let client = FirebaseClient::new(project_id, api_key);
let collection_manager = CollectionManager::new(client);

// List all collections
let collections = collection_manager.list_collections().await?;
for collection in collections {
    println!("{}: {} documents", collection.name, collection.document_count);
}

// Get specific collection info
let info = collection_manager.get_collection_info("users").await?;
println!("Collection: {}", info.name);
println!("Documents: {}", info.document_count);
println!("Size: {}", info.estimated_size);
```

### Schema Analysis
```rust
// Analyze collection schema
let schema = collection_manager.describe_collection("users", 50).await?;

println!("Collection: {}", schema.collection_name);
println!("Documents analyzed: {}", schema.total_documents);

for field in schema.fields {
    println!("Field: {} ({})", field.name, field.field_type);
    println!("  Required: {}", field.is_required);
    println!("  Frequency: {}/{}", field.frequency, schema.total_documents);
    println!("  Samples: {:?}", field.sample_values);
}

// Show sample document
if let Some(sample) = schema.sample_document {
    println!("Sample: {}", serde_json::to_string_pretty(&sample)?);
}
```

### Table Formatting
```rust
// Format collections as table
let table_output = collection_manager.format_collections_table(&collections, true);
println!("{}", table_output);

// Format as simple text
let text_output = collection_manager.format_collections_table(&collections, false);
println!("{}", text_output);

// Format schema analysis
let schema_table = collection_manager.format_schema_table(&schema, true);
println!("{}", schema_table);
```

## Field Types Detected

| Firestore Type | Rust Display | Description |
|----------------|--------------|-------------|
| `stringValue` | `string` | Text fields |
| `integerValue` | `integer` | Whole numbers |
| `doubleValue` | `double` | Decimal numbers |
| `booleanValue` | `boolean` | True/false values |
| `timestampValue` | `timestamp` | Date/time fields |
| `arrayValue` | `array` | Lists of values |
| `mapValue` | `map` | Nested objects |
| `nullValue` | `null` | Empty/null values |

### Mixed Types
When a field contains multiple types across documents:
```
field_type: "Mixed(string, integer)"
```

## Schema Analysis Details

### Field Information
- **Name**: Field identifier
- **Type**: Data type (or "Mixed" for multiple types)
- **Required**: ✓ if present in all documents, ✗ if optional
- **Frequency**: How many documents contain this field (e.g., "18/20")
- **Sample Values**: Up to 5 example values from the collection

### Statistics
- **Total Documents**: Number of documents analyzed
- **Field Count**: Total unique fields found
- **Required vs Optional**: Breakdown of field requirements
- **Type Distribution**: Count of each data type

## Use Cases

### 🔍 Database Exploration
```bash
# Discover what's in your database
cargo run --bin firebase-cli collections list

# Understand data structure
cargo run --bin firebase-cli collections describe -c users
```

### 📊 Schema Documentation
```bash
# Generate schema documentation
cargo run --bin firebase-cli collections describe -c users > users_schema.txt

# Export for multiple collections
for collection in users posts comments; do
    cargo run --bin firebase-cli collections describe -c $collection > ${collection}_schema.txt
done
```

### 🛠️ Development & Debugging
```bash
# Quick health check
cargo run --bin firebase-cli collections list

# Verify field types
cargo run --bin firebase-cli collections describe -c users -f text

# Check collection sizes
cargo run --bin firebase-cli collections info -c users
```

### 📋 Data Migration Planning
```rust
// Analyze source schema
let source_schema = collection_manager.describe_collection("users", 100).await?;

// Plan migration based on field analysis
for field in source_schema.fields {
    if field.field_type.contains("Mixed") {
        println!("⚠️ Field '{}' has mixed types - needs conversion", field.name);
    }
    
    if !field.is_required && field.frequency < source_schema.total_documents {
        println!("ℹ️ Field '{}' is optional ({} frequency)", field.name, field.frequency);
    }
}
```

## Performance & Limitations

### Collection Discovery
- **Method**: Tries common collection names and checks metadata
- **Performance**: Fast for small numbers of collections
- **Limitation**: May not find all collections if they have unusual names

### Schema Analysis
- **Sample Size**: Analyzes up to 100 documents (Firebase API limit)
- **Accuracy**: More documents = more accurate schema understanding
- **Performance**: Fast analysis, results cached per request

### Table Formatting
- **Coloring**: Uses terminal colors for better readability
- **Responsive**: Tables adjust to content size
- **Export**: Text format available for scripting

## Error Handling

### Collection Not Found
```
❌ Collection 'posts' not found or is empty
```

### Access Issues
```
❌ Error analyzing collection: Missing or insufficient permissions
```

### Network Problems
```
❌ Error listing collections: HTTP request failed: connection timeout
```

## Best Practices

### Schema Analysis
- **Use appropriate sample sizes**: 50 for quick analysis, 100 for thorough analysis
- **Check mixed types**: Look for fields with inconsistent types
- **Document findings**: Export schema analysis to files for reference

### Collection Management
- **Regular monitoring**: Check collection growth over time
- **Performance awareness**: Large collections may take longer to analyze
- **Access control**: Ensure proper Firebase security rules

### Integration
- **Combine with exports**: Use with data export/import features
- **Automate documentation**: Script schema analysis for CI/CD
- **Monitor changes**: Track schema evolution over time