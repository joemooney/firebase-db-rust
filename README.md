# Firebase Rust Database Client

A Rust client for Firebase Firestore with CRUD operations using REST API.

## Setup

1. Create a Firebase project at https://console.firebase.google.com/
2. Enable Firestore Database in your project
3. Get your API key from Project Settings > General
4. Copy `.env.example` to `.env` and fill in your credentials:
   ```
   FIREBASE_PROJECT_ID=your-project-id
   FIREBASE_API_KEY=your-api-key
   ```

## Usage

Run the example:
```bash
cargo run
```

## API

### Create
```rust
let user = User::new("John Doe".to_string(), "john@example.com".to_string(), 30);
let doc_id = client.create("/users", &user).await?;
```

### Read
```rust
let user: User = client.get("/users", &doc_id).await?;
```

### Update
```rust
client.update("/users", &doc_id, &updated_user).await?;
```

### Delete
```rust
client.delete("/users", &doc_id).await?;
```

### List
```rust
let users: Vec<User> = client.list("/users", Some(10)).await?;
```

## Note

This implementation uses Firebase REST API with API key authentication. For production use, consider:
- Using service account authentication for better security
- Implementing connection pooling
- Adding retry logic for network failures
- Using Firebase Admin SDK when available for Rust