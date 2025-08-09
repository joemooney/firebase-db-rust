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