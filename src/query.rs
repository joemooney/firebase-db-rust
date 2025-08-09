use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::FirestoreValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredQuery {
    pub from: Vec<CollectionSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#where: Option<Filter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<Vec<Order>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSelector {
    pub collection_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_descendants: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Filter {
    CompositeFilter(CompositeFilter),
    FieldFilter(FieldFilter),
    UnaryFilter(UnaryFilter),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositeFilter {
    pub op: CompositeOperator,
    pub filters: Vec<Filter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompositeOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldFilter {
    pub field: FieldReference,
    pub op: FieldOperator,
    pub value: FirestoreValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldReference {
    pub field_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FieldOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
    ArrayContains,
    In,
    ArrayContainsAny,
    NotIn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnaryFilter {
    pub op: UnaryOperator,
    pub field: FieldReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnaryOperator {
    IsNan,
    IsNull,
    IsNotNan,
    IsNotNull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub field: FieldReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    Ascending,
    Descending,
}

pub struct QueryBuilder {
    query: StructuredQuery,
}

impl QueryBuilder {
    pub fn new(collection: &str) -> Self {
        Self {
            query: StructuredQuery {
                from: vec![CollectionSelector {
                    collection_id: collection.to_string(),
                    all_descendants: None,
                }],
                r#where: None,
                order_by: None,
                limit: None,
                offset: None,
            },
        }
    }
    
    pub fn where_eq(mut self, field: &str, value: FirestoreValue) -> Self {
        let filter = Filter::FieldFilter(FieldFilter {
            field: FieldReference { field_path: field.to_string() },
            op: FieldOperator::Equal,
            value,
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_lt(mut self, field: &str, value: FirestoreValue) -> Self {
        let filter = Filter::FieldFilter(FieldFilter {
            field: FieldReference { field_path: field.to_string() },
            op: FieldOperator::LessThan,
            value,
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_gt(mut self, field: &str, value: FirestoreValue) -> Self {
        let filter = Filter::FieldFilter(FieldFilter {
            field: FieldReference { field_path: field.to_string() },
            op: FieldOperator::GreaterThan,
            value,
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_lte(mut self, field: &str, value: FirestoreValue) -> Self {
        let filter = Filter::FieldFilter(FieldFilter {
            field: FieldReference { field_path: field.to_string() },
            op: FieldOperator::LessThanOrEqual,
            value,
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_gte(mut self, field: &str, value: FirestoreValue) -> Self {
        let filter = Filter::FieldFilter(FieldFilter {
            field: FieldReference { field_path: field.to_string() },
            op: FieldOperator::GreaterThanOrEqual,
            value,
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_ne(mut self, field: &str, value: FirestoreValue) -> Self {
        let filter = Filter::FieldFilter(FieldFilter {
            field: FieldReference { field_path: field.to_string() },
            op: FieldOperator::NotEqual,
            value,
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_in(mut self, field: &str, values: Vec<FirestoreValue>) -> Self {
        let filter = Filter::FieldFilter(FieldFilter {
            field: FieldReference { field_path: field.to_string() },
            op: FieldOperator::In,
            value: FirestoreValue::ArrayValue { values },
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_array_contains(mut self, field: &str, value: FirestoreValue) -> Self {
        let filter = Filter::FieldFilter(FieldFilter {
            field: FieldReference { field_path: field.to_string() },
            op: FieldOperator::ArrayContains,
            value,
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_is_null(mut self, field: &str) -> Self {
        let filter = Filter::UnaryFilter(UnaryFilter {
            op: UnaryOperator::IsNull,
            field: FieldReference { field_path: field.to_string() },
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn where_is_not_null(mut self, field: &str) -> Self {
        let filter = Filter::UnaryFilter(UnaryFilter {
            op: UnaryOperator::IsNotNull,
            field: FieldReference { field_path: field.to_string() },
        });
        self.query.r#where = Some(filter);
        self
    }
    
    pub fn and(mut self, filters: Vec<Filter>) -> Self {
        let composite = Filter::CompositeFilter(CompositeFilter {
            op: CompositeOperator::And,
            filters,
        });
        self.query.r#where = Some(composite);
        self
    }
    
    pub fn or(mut self, filters: Vec<Filter>) -> Self {
        let composite = Filter::CompositeFilter(CompositeFilter {
            op: CompositeOperator::Or,
            filters,
        });
        self.query.r#where = Some(composite);
        self
    }
    
    pub fn order_by(mut self, field: &str, descending: bool) -> Self {
        let order = Order {
            field: FieldReference { field_path: field.to_string() },
            direction: Some(if descending { Direction::Descending } else { Direction::Ascending }),
        };
        
        if let Some(ref mut orders) = self.query.order_by {
            orders.push(order);
        } else {
            self.query.order_by = Some(vec![order]);
        }
        self
    }
    
    pub fn limit(mut self, limit: i32) -> Self {
        self.query.limit = Some(limit);
        self
    }
    
    pub fn offset(mut self, offset: i32) -> Self {
        self.query.offset = Some(offset);
        self
    }
    
    pub fn build(self) -> StructuredQuery {
        self.query
    }
}

pub fn create_filter(field: &str, op: FieldOperator, value: FirestoreValue) -> Filter {
    Filter::FieldFilter(FieldFilter {
        field: FieldReference { field_path: field.to_string() },
        op,
        value,
    })
}