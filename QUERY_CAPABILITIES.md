# Firestore Query Capabilities (SQL Equivalents)

Firestore provides rich querying capabilities similar to SQL WHERE clauses. Here's what's available:

## Basic Comparison Operators

| SQL | Firestore Method | Description |
|-----|-----------------|-------------|
| `WHERE age = 30` | `.where_eq("age", value)` | Equal to |
| `WHERE age < 30` | `.where_lt("age", value)` | Less than |
| `WHERE age <= 30` | `.where_lte("age", value)` | Less than or equal |
| `WHERE age > 30` | `.where_gt("age", value)` | Greater than |
| `WHERE age >= 30` | `.where_gte("age", value)` | Greater than or equal |
| `WHERE age != 30` | `.where_ne("age", value)` | Not equal to |

## Advanced Operators

| SQL | Firestore Method | Description |
|-----|-----------------|-------------|
| `WHERE age IN (25, 30, 35)` | `.where_in("age", vec![...])` | Value in list |
| `WHERE age NOT IN (25, 30)` | `.where_not_in("age", vec![...])` | Value not in list |
| `WHERE tags CONTAINS 'rust'` | `.where_array_contains("tags", value)` | Array contains value |
| `WHERE tags CONTAINS ANY ('rust', 'go')` | `.where_array_contains_any("tags", vec![...])` | Array contains any |
| `WHERE email IS NULL` | `.where_is_null("email")` | Field is null |
| `WHERE email IS NOT NULL` | `.where_is_not_null("email")` | Field is not null |

## Compound Queries

| SQL | Firestore Method | Description |
|-----|-----------------|-------------|
| `WHERE age >= 25 AND age <= 30` | `.and(vec![filter1, filter2])` | Multiple AND conditions |
| `WHERE age < 25 OR age > 65` | `.or(vec![filter1, filter2])` | Multiple OR conditions |

## Ordering and Pagination

| SQL | Firestore Method | Description |
|-----|-----------------|-------------|
| `ORDER BY age ASC` | `.order_by("age", false)` | Sort ascending |
| `ORDER BY age DESC` | `.order_by("age", true)` | Sort descending |
| `LIMIT 10` | `.limit(10)` | Limit results |
| `OFFSET 20` | `.offset(20)` | Skip results |

## Usage Examples

### Simple Query
```rust
// SQL: SELECT * FROM users WHERE age > 25
let query = QueryBuilder::new("users")
    .where_gt("age", FirestoreValue::IntegerValue("25".to_string()))
    .build();
let users: Vec<User> = client.query(query).await?;
```

### Range Query
```rust
// SQL: SELECT * FROM users WHERE age >= 25 AND age <= 30
let filters = vec![
    create_filter("age", FieldOperator::GreaterThanOrEqual, 
                  FirestoreValue::IntegerValue("25".to_string())),
    create_filter("age", FieldOperator::LessThanOrEqual, 
                  FirestoreValue::IntegerValue("30".to_string())),
];
let query = QueryBuilder::new("users")
    .and(filters)
    .build();
```

### Complex Query with Ordering
```rust
// SQL: SELECT * FROM users WHERE age > 20 ORDER BY age DESC LIMIT 10
let query = QueryBuilder::new("users")
    .where_gt("age", FirestoreValue::IntegerValue("20".to_string()))
    .order_by("age", true)  // true = descending
    .limit(10)
    .build();
```

## Limitations

1. **OR queries**: Limited support - can't combine OR with range queries
2. **NOT queries**: Limited to NOT IN and != operators
3. **Joins**: No JOIN support - Firestore is NoSQL
4. **Aggregations**: Limited (COUNT, SUM, AVG available through separate API)
5. **Full-text search**: Not natively supported (use external service)
6. **Case-insensitive**: No native support (store lowercase copies)
7. **Regex/LIKE**: Not supported (use external search service)

## Best Practices

1. **Indexes**: Complex queries require composite indexes (configure in Firebase Console)
2. **Performance**: Use `.limit()` to avoid fetching large datasets
3. **Costs**: Each query counts as document reads (billing consideration)
4. **Denormalization**: Store redundant data to avoid joins
5. **Array queries**: Limited to one array-contains per query