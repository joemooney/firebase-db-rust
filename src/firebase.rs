use crate::error::{FirebaseError, Result};
use crate::models::{FirestoreValue, ToFirestore, FromFirestore};
use crate::query::{StructuredQuery, QueryBuilder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct FirebaseClient {
    pub client: Client,
    project_id: String,
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateDocumentRequest {
    fields: HashMap<String, FirestoreValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateDocumentRequest {
    fields: HashMap<String, FirestoreValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListDocumentsResponse {
    documents: Option<Vec<Document>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    name: String,
    fields: HashMap<String, FirestoreValue>,
    #[serde(rename = "createTime")]
    create_time: Option<String>,
    #[serde(rename = "updateTime")]
    update_time: Option<String>,
}

impl FirebaseClient {
    pub fn new(project_id: String, api_key: String) -> Self {
        let base_url = format!(
            "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents",
            project_id
        );
        
        Self {
            client: Client::new(),
            project_id,
            api_key,
            base_url,
        }
    }
    
    pub async fn create<T: ToFirestore>(&self, collection: &str, item: &T) -> Result<String> {
        let collection_path = collection.trim_start_matches('/');
        let url = format!("{}/{}?key={}", 
            self.base_url, 
            collection_path,
            self.api_key
        );
        
        let request_body = CreateDocumentRequest {
            fields: item.to_firestore(),
        };
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to create document: {}", error_text)));
        }
        
        let doc: Document = response.json().await?;
        
        let doc_id = doc.name
            .split('/')
            .last()
            .ok_or_else(|| FirebaseError::DatabaseError("Invalid document name".to_string()))?
            .to_string();
        
        Ok(doc_id)
    }
    
    pub async fn get<T: FromFirestore>(&self, collection: &str, doc_id: &str) -> Result<T> {
        let collection_path = collection.trim_start_matches('/');
        let url = format!("{}/{}/{}?key={}", 
            self.base_url,
            collection_path,
            doc_id,
            self.api_key
        );
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status() == 404 {
            return Err(FirebaseError::NotFound(doc_id.to_string()));
        }
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to get document: {}", error_text)));
        }
        
        let doc: Document = response.json().await?;
        T::from_firestore(&doc.fields)
    }
    
    pub async fn update<T: ToFirestore>(&self, collection: &str, doc_id: &str, item: &T) -> Result<()> {
        let collection_path = collection.trim_start_matches('/');
        let fields = item.to_firestore();
        let field_paths: Vec<String> = fields.keys().cloned().collect();
        let update_mask = field_paths.join("&updateMask.fieldPaths=");
        
        let url = format!("{}/{}/{}?key={}&updateMask.fieldPaths={}", 
            self.base_url,
            collection_path,
            doc_id,
            self.api_key,
            update_mask
        );
        
        let mut update_fields = fields.clone();
        update_fields.insert("updated_at".to_string(), FirestoreValue::TimestampValue(Utc::now().to_rfc3339()));
        
        let request_body = UpdateDocumentRequest { fields: update_fields };
        
        let response = self.client
            .patch(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to update document: {}", error_text)));
        }
        
        Ok(())
    }
    
    pub async fn delete(&self, collection: &str, doc_id: &str) -> Result<()> {
        let collection_path = collection.trim_start_matches('/');
        let url = format!("{}/{}/{}?key={}", 
            self.base_url,
            collection_path,
            doc_id,
            self.api_key
        );
        
        let response = self.client
            .delete(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to delete document: {}", error_text)));
        }
        
        Ok(())
    }
    
    pub async fn list<T: FromFirestore>(&self, collection: &str, limit: Option<usize>) -> Result<Vec<T>> {
        let collection_path = collection.trim_start_matches('/');
        let mut url = format!("{}/{}?key={}", 
            self.base_url,
            collection_path,
            self.api_key
        );
        
        if let Some(limit) = limit {
            url.push_str(&format!("&pageSize={}", limit));
        }
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to list documents: {}", error_text)));
        }
        
        let list_response: ListDocumentsResponse = response.json().await?;
        
        let mut results = Vec::new();
        if let Some(documents) = list_response.documents {
            for doc in documents {
                match T::from_firestore(&doc.fields) {
                    Ok(item) => results.push(item),
                    Err(e) => eprintln!("Failed to parse document: {:?}", e),
                }
            }
        }
        
        Ok(results)
    }
    
    pub async fn query<T: FromFirestore>(&self, query: StructuredQuery) -> Result<Vec<T>> {
        let url = format!("{}:runQuery?key={}", 
            self.base_url,
            self.api_key
        );
        
        let request_body = RunQueryRequest {
            structured_query: query,
        };
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Failed to run query: {}", error_text)));
        }
        
        let response_text = response.text().await?;
        let lines: Vec<&str> = response_text.lines().collect();
        
        let mut results = Vec::new();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            
            if let Ok(query_response) = serde_json::from_str::<QueryDocumentResponse>(line) {
                if let Some(document) = query_response.document {
                    match T::from_firestore(&document.fields) {
                        Ok(item) => results.push(item),
                        Err(e) => eprintln!("Failed to parse document: {:?}", e),
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    pub fn query_builder(collection: &str) -> QueryBuilder {
        QueryBuilder::new(collection)
    }
    
    // Generic CRUD methods for working with serde_json::Value
    pub async fn create_document(&self, collection: &str, doc_id: Option<String>, data: serde_json::Value) -> Result<String> {
        let fields = json_to_firestore_fields(data)?;
        let request = CreateDocumentRequest { fields };
        
        let url = if let Some(id) = doc_id {
            format!("{}?key={}&documentId={}", 
                format!("{}/{}", self.base_url, collection.trim_start_matches('/')), 
                self.api_key, id)
        } else {
            format!("{}?key={}", 
                format!("{}/{}", self.base_url, collection.trim_start_matches('/')), 
                self.api_key)
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Create failed: {}", error_text)));
        }
        
        let document: Document = response.json().await?;
        let doc_id = document.name.split('/').last().unwrap_or("unknown").to_string();
        Ok(doc_id)
    }
    
    pub async fn get_document(&self, collection: &str, doc_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}?key={}", 
            format!("{}/{}", self.base_url, collection.trim_start_matches('/')), 
            doc_id, self.api_key);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if response.status().as_u16() == 404 {
            return Err(FirebaseError::NotFound(format!("Document '{}' not found in collection '{}'", doc_id, collection)));
        }
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Get failed: {}", error_text)));
        }
        
        let document: Document = response.json().await?;
        firestore_fields_to_json(document.fields)
    }
    
    pub async fn update_document(&self, collection: &str, doc_id: &str, data: serde_json::Value, merge: bool) -> Result<()> {
        let fields = json_to_firestore_fields(data)?;
        let request = UpdateDocumentRequest { fields };
        
        let update_mask = if merge {
            let field_names: Vec<String> = request.fields.keys().cloned().collect();
            format!("&updateMask.fieldPaths={}", field_names.join("&updateMask.fieldPaths="))
        } else {
            String::new()
        };
        
        let url = format!("{}/{}?key={}{}", 
            format!("{}/{}", self.base_url, collection.trim_start_matches('/')), 
            doc_id, self.api_key, update_mask);
        
        let response = self.client
            .patch(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Update failed: {}", error_text)));
        }
        
        Ok(())
    }
    
    pub async fn delete_document(&self, collection: &str, doc_id: &str) -> Result<()> {
        let url = format!("{}/{}?key={}", 
            format!("{}/{}", self.base_url, collection.trim_start_matches('/')), 
            doc_id, self.api_key);
        
        let response = self.client
            .delete(&url)
            .send()
            .await?;
        
        if response.status().as_u16() == 404 {
            return Err(FirebaseError::NotFound(format!("Document '{}' not found in collection '{}'", doc_id, collection)));
        }
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FirebaseError::DatabaseError(format!("Delete failed: {}", error_text)));
        }
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunQueryRequest {
    structured_query: StructuredQuery,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryDocumentResponse {
    document: Option<Document>,
    #[serde(rename = "readTime")]
    read_time: Option<String>,
}

// Helper functions for JSON <-> Firestore conversion
fn json_to_firestore_fields(value: serde_json::Value) -> Result<HashMap<String, FirestoreValue>> {
    let mut fields = HashMap::new();
    
    if let serde_json::Value::Object(map) = value {
        for (key, val) in map {
            fields.insert(key, json_value_to_firestore(val)?);
        }
    } else {
        return Err(FirebaseError::ValidationError("Root value must be an object".to_string()));
    }
    
    Ok(fields)
}

fn json_value_to_firestore(value: serde_json::Value) -> Result<FirestoreValue> {
    match value {
        serde_json::Value::String(s) => Ok(FirestoreValue::StringValue(s)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(FirestoreValue::IntegerValue(i.to_string()))
            } else if let Some(f) = n.as_f64() {
                Ok(FirestoreValue::DoubleValue(f))
            } else {
                Ok(FirestoreValue::StringValue(n.to_string()))
            }
        }
        serde_json::Value::Bool(b) => Ok(FirestoreValue::BooleanValue(b)),
        serde_json::Value::Null => Ok(FirestoreValue::NullValue),
        serde_json::Value::Array(arr) => {
            let mut values = Vec::new();
            for item in arr {
                values.push(json_value_to_firestore(item)?);
            }
            Ok(FirestoreValue::ArrayValue { values })
        }
        serde_json::Value::Object(map) => {
            let mut fields = HashMap::new();
            for (key, val) in map {
                fields.insert(key, json_value_to_firestore(val)?);
            }
            Ok(FirestoreValue::MapValue { fields })
        }
    }
}

fn firestore_fields_to_json(fields: HashMap<String, FirestoreValue>) -> Result<serde_json::Value> {
    let mut map = serde_json::Map::new();
    
    for (key, value) in fields {
        map.insert(key, firestore_value_to_json(value)?);
    }
    
    Ok(serde_json::Value::Object(map))
}

fn firestore_value_to_json(value: FirestoreValue) -> Result<serde_json::Value> {
    match value {
        FirestoreValue::StringValue(s) => Ok(serde_json::Value::String(s)),
        FirestoreValue::IntegerValue(i) => {
            Ok(serde_json::Value::Number(serde_json::Number::from(i.parse::<i64>().unwrap_or(0))))
        }
        FirestoreValue::DoubleValue(f) => {
            Ok(serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0))))
        }
        FirestoreValue::BooleanValue(b) => Ok(serde_json::Value::Bool(b)),
        FirestoreValue::NullValue => Ok(serde_json::Value::Null),
        FirestoreValue::TimestampValue(ts) => Ok(serde_json::Value::String(ts)),
        FirestoreValue::ArrayValue { values } => {
            let mut json_arr = Vec::new();
            for val in values {
                json_arr.push(firestore_value_to_json(val)?);
            }
            Ok(serde_json::Value::Array(json_arr))
        }
        FirestoreValue::MapValue { fields } => {
            let mut json_map = serde_json::Map::new();
            for (key, val) in fields {
                json_map.insert(key, firestore_value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(json_map))
        }
    }
}