use firebase_db::{FirebaseClient, User, FirebaseError, QueryBuilder, FirestoreValue, FieldOperator, create_filter};
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
    
    println!("Firestore Query Examples (SQL-like WHERE clauses)");
    println!("=================================================\n");
    
    println!("First, let's create some test data...");
    let users = vec![
        User::new("Alice Smith".to_string(), "alice@example.com".to_string(), 25),
        User::new("Bob Johnson".to_string(), "bob@example.com".to_string(), 30),
        User::new("Charlie Brown".to_string(), "charlie@example.com".to_string(), 35),
        User::new("Diana Prince".to_string(), "diana@example.com".to_string(), 28),
        User::new("Eve Adams".to_string(), "eve@example.com".to_string(), 22),
    ];
    
    for user in &users {
        client.create("/users", user).await?;
        println!("Created: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("1. Simple WHERE clause (age == 30):");
    println!("   SQL: SELECT * FROM users WHERE age = 30");
    let query = QueryBuilder::new("users")
        .where_eq("age", FirestoreValue::IntegerValue("30".to_string()))
        .build();
    let results: Vec<User> = client.query(query).await?;
    for user in results {
        println!("   Found: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("2. Greater than query (age > 25):");
    println!("   SQL: SELECT * FROM users WHERE age > 25");
    let query = QueryBuilder::new("users")
        .where_gt("age", FirestoreValue::IntegerValue("25".to_string()))
        .build();
    let results: Vec<User> = client.query(query).await?;
    for user in results {
        println!("   Found: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("3. Less than or equal query (age <= 28):");
    println!("   SQL: SELECT * FROM users WHERE age <= 28");
    let query = QueryBuilder::new("users")
        .where_lte("age", FirestoreValue::IntegerValue("28".to_string()))
        .build();
    let results: Vec<User> = client.query(query).await?;
    for user in results {
        println!("   Found: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("4. IN query (age IN [22, 28, 35]):");
    println!("   SQL: SELECT * FROM users WHERE age IN (22, 28, 35)");
    let query = QueryBuilder::new("users")
        .where_in("age", vec![
            FirestoreValue::IntegerValue("22".to_string()),
            FirestoreValue::IntegerValue("28".to_string()),
            FirestoreValue::IntegerValue("35".to_string()),
        ])
        .build();
    let results: Vec<User> = client.query(query).await?;
    for user in results {
        println!("   Found: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("5. Compound query with AND (age >= 25 AND age <= 30):");
    println!("   SQL: SELECT * FROM users WHERE age >= 25 AND age <= 30");
    let filters = vec![
        create_filter("age", FieldOperator::GreaterThanOrEqual, FirestoreValue::IntegerValue("25".to_string())),
        create_filter("age", FieldOperator::LessThanOrEqual, FirestoreValue::IntegerValue("30".to_string())),
    ];
    let query = QueryBuilder::new("users")
        .and(filters)
        .build();
    let results: Vec<User> = client.query(query).await?;
    for user in results {
        println!("   Found: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("6. ORDER BY with LIMIT:");
    println!("   SQL: SELECT * FROM users ORDER BY age DESC LIMIT 3");
    let query = QueryBuilder::new("users")
        .order_by("age", true)
        .limit(3)
        .build();
    let results: Vec<User> = client.query(query).await?;
    for user in results {
        println!("   Found: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("7. Not equal query (age != 30):");
    println!("   SQL: SELECT * FROM users WHERE age != 30");
    let query = QueryBuilder::new("users")
        .where_ne("age", FirestoreValue::IntegerValue("30".to_string()))
        .build();
    let results: Vec<User> = client.query(query).await?;
    for user in results {
        println!("   Found: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("8. Complex query with ordering and pagination:");
    println!("   SQL: SELECT * FROM users WHERE age > 20 ORDER BY age ASC LIMIT 2 OFFSET 1");
    let query = QueryBuilder::new("users")
        .where_gt("age", FirestoreValue::IntegerValue("20".to_string()))
        .order_by("age", false)
        .limit(2)
        .offset(1)
        .build();
    let results: Vec<User> = client.query(query).await?;
    for user in results {
        println!("   Found: {} (age: {})", user.name, user.age);
    }
    println!();
    
    println!("Cleaning up test data...");
    let all_users: Vec<User> = client.list("/users", None).await?;
    for user in all_users {
        if let Some(id) = &user.id {
            client.delete("/users", id).await?;
        }
    }
    println!("Test data cleaned up successfully!");
    
    Ok(())
}
