use firebase_db::{FirebaseClient, User, FirebaseError};
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
    
    println!("Firebase Rust CRUD Example");
    println!("==========================\n");
    
    let user = User::new(
        "John Doe".to_string(),
        "john@example.com".to_string(),
        30
    );
    println!("Creating user: {:?}", user);
    
    let doc_id = client.create("/users", &user).await?;
    println!("Created user with ID: {}\n", doc_id);
    
    println!("Fetching user with ID: {}", doc_id);
    let fetched_user: User = client.get("/users", &doc_id).await?;
    println!("Fetched user: {:?}\n", fetched_user);
    
    let mut updated_user = fetched_user.clone();
    updated_user.age = 31;
    updated_user.name = "John Smith".to_string();
    
    println!("Updating user...");
    client.update("/users", &doc_id, &updated_user).await?;
    println!("User updated successfully\n");
    
    println!("Fetching updated user...");
    let updated_fetched: User = client.get("/users", &doc_id).await?;
    println!("Updated user: {:?}\n", updated_fetched);
    
    println!("Listing all users...");
    let all_users: Vec<User> = client.list("/users", Some(10)).await?;
    for user in &all_users {
        println!("  - {:?}", user);
    }
    println!();
    
    println!("Deleting user with ID: {}", doc_id);
    client.delete("/users", &doc_id).await?;
    println!("User deleted successfully\n");
    
    match client.get::<User>("/users", &doc_id).await {
        Err(FirebaseError::NotFound(_)) => println!("User not found (as expected after deletion)"),
        Ok(_) => println!("ERROR: User still exists!"),
        Err(e) => println!("Error fetching deleted user: {:?}", e),
    }
    
    Ok(())
}
