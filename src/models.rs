use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub name: String,
    pub email: String,
    pub age: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: String, email: String, age: u32) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            name,
            email,
            age,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirestoreDocument<T> {
    pub name: String,
    pub fields: HashMap<String, FirestoreValue>,
    #[serde(skip)]
    pub _phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FirestoreValue {
    StringValue(String),
    IntegerValue(String),
    DoubleValue(f64),
    BooleanValue(bool),
    TimestampValue(String),
    MapValue { 
        #[serde(default)]
        fields: HashMap<String, FirestoreValue> 
    },
    ArrayValue { 
        #[serde(default)]
        values: Vec<FirestoreValue> 
    },
    #[serde(rename = "nullValue")]
    NullValue(Option<()>),
    // Handle cases where Firestore returns different field names
    #[serde(other)]
    Unknown,
}

mod array_value_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    
    #[derive(Serialize, Deserialize)]
    struct ArrayValueWrapper {
        values: Vec<FirestoreValue>,
    }
    
    pub fn serialize<S>(values: &Vec<FirestoreValue>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ArrayValueWrapper { 
            values: values.clone() 
        }.serialize(serializer)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<FirestoreValue>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wrapper = ArrayValueWrapper::deserialize(deserializer)?;
        Ok(wrapper.values)
    }
}

pub trait ToFirestore {
    fn to_firestore(&self) -> HashMap<String, FirestoreValue>;
}

pub trait FromFirestore: Sized {
    fn from_firestore(fields: &HashMap<String, FirestoreValue>) -> crate::error::Result<Self>;
}

impl ToFirestore for User {
    fn to_firestore(&self) -> HashMap<String, FirestoreValue> {
        let mut fields = HashMap::new();
        
        if let Some(id) = &self.id {
            fields.insert("id".to_string(), FirestoreValue::StringValue(id.clone()));
        }
        
        fields.insert("name".to_string(), FirestoreValue::StringValue(self.name.clone()));
        fields.insert("email".to_string(), FirestoreValue::StringValue(self.email.clone()));
        fields.insert("age".to_string(), FirestoreValue::IntegerValue(self.age.to_string()));
        fields.insert("created_at".to_string(), FirestoreValue::TimestampValue(self.created_at.to_rfc3339()));
        fields.insert("updated_at".to_string(), FirestoreValue::TimestampValue(self.updated_at.to_rfc3339()));
        
        fields
    }
}

impl FromFirestore for User {
    fn from_firestore(fields: &HashMap<String, FirestoreValue>) -> crate::error::Result<Self> {
        let id = match fields.get("id") {
            Some(FirestoreValue::StringValue(v)) => Some(v.clone()),
            _ => None,
        };
        
        let name = match fields.get("name") {
            Some(FirestoreValue::StringValue(v)) => v.clone(),
            _ => return Err(crate::error::FirebaseError::DatabaseError("Missing name field".to_string())),
        };
        
        let email = match fields.get("email") {
            Some(FirestoreValue::StringValue(v)) => v.clone(),
            _ => return Err(crate::error::FirebaseError::DatabaseError("Missing email field".to_string())),
        };
        
        let age = match fields.get("age") {
            Some(FirestoreValue::IntegerValue(v)) => v.parse().unwrap_or(0),
            _ => 0,
        };
        
        let created_at = match fields.get("created_at") {
            Some(FirestoreValue::TimestampValue(v)) => DateTime::parse_from_rfc3339(v)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            _ => Utc::now(),
        };
        
        let updated_at = match fields.get("updated_at") {
            Some(FirestoreValue::TimestampValue(v)) => DateTime::parse_from_rfc3339(v)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            _ => Utc::now(),
        };
        
        Ok(User {
            id,
            name,
            email,
            age,
            created_at,
            updated_at,
        })
    }
}