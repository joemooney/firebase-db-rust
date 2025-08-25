use crate::error::{FirebaseError, Result};
use std::fmt;

#[derive(Debug, Clone)]
pub struct SecurityRules {
    rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub path: String,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Allow(Permission, Expression),
    Deny(Permission, Expression),
}

#[derive(Debug, Clone)]
pub enum Permission {
    Read,
    Write,
    Create,
    Update,
    Delete,
    List,
    Get,
}

#[derive(Debug, Clone)]
pub enum Expression {
    True,
    False,
    IsAuthenticated,
    IsOwner(String),
    HasRole(String),
    FieldEquals(String, String),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Custom(String),
}

impl SecurityRules {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
    
    pub fn collection(path: &str) -> RuleBuilder {
        RuleBuilder::new(path)
    }
    
    pub fn generate(&self) -> String {
        let mut output = String::from("rules_version = '2';\nservice cloud.firestore {\n  match /databases/{database}/documents {\n");
        
        for rule in &self.rules {
            output.push_str(&format!("\n    match {} {{\n", rule.path));
            
            for condition in &rule.conditions {
                output.push_str(&format!("      {}\n", condition));
            }
            
            output.push_str("    }\n");
        }
        
        output.push_str("  }\n}\n");
        output
    }
    
    pub fn export_to_file(&self, path: &str) -> Result<()> {
        let content = self.generate();
        std::fs::write(path, content)
            .map_err(|e| FirebaseError::ConfigError(format!("Failed to write rules file: {}", e)))?;
        Ok(())
    }
}

pub struct RuleBuilder {
    path: String,
    conditions: Vec<Condition>,
}

impl RuleBuilder {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            conditions: Vec::new(),
        }
    }
    
    pub fn allow_read_if(mut self, expr: Expression) -> Self {
        self.conditions.push(Condition::Allow(Permission::Read, expr));
        self
    }
    
    pub fn allow_write_if(mut self, expr: Expression) -> Self {
        self.conditions.push(Condition::Allow(Permission::Write, expr));
        self
    }
    
    pub fn allow_create_if(mut self, expr: Expression) -> Self {
        self.conditions.push(Condition::Allow(Permission::Create, expr));
        self
    }
    
    pub fn allow_update_if(mut self, expr: Expression) -> Self {
        self.conditions.push(Condition::Allow(Permission::Update, expr));
        self
    }
    
    pub fn allow_delete_if(mut self, expr: Expression) -> Self {
        self.conditions.push(Condition::Allow(Permission::Delete, expr));
        self
    }
    
    pub fn allow_authenticated_read(mut self) -> Self {
        self.conditions.push(Condition::Allow(Permission::Read, Expression::IsAuthenticated));
        self
    }
    
    pub fn allow_authenticated_write(mut self) -> Self {
        self.conditions.push(Condition::Allow(Permission::Write, Expression::IsAuthenticated));
        self
    }
    
    pub fn allow_owner_only(mut self, user_id_field: &str) -> Self {
        self.conditions.push(Condition::Allow(
            Permission::Read, 
            Expression::IsOwner(user_id_field.to_string())
        ));
        self.conditions.push(Condition::Allow(
            Permission::Write, 
            Expression::IsOwner(user_id_field.to_string())
        ));
        self
    }
    
    pub fn allow_admin_only(mut self) -> Self {
        self.conditions.push(Condition::Allow(
            Permission::Read,
            Expression::HasRole("admin".to_string())
        ));
        self.conditions.push(Condition::Allow(
            Permission::Write,
            Expression::HasRole("admin".to_string())
        ));
        self
    }
    
    pub fn public_read(mut self) -> Self {
        self.conditions.push(Condition::Allow(Permission::Read, Expression::True));
        self
    }
    
    pub fn deny_all(mut self) -> Self {
        self.conditions.push(Condition::Deny(Permission::Read, Expression::True));
        self.conditions.push(Condition::Deny(Permission::Write, Expression::True));
        self
    }
    
    pub fn build(self) -> Rule {
        Rule {
            path: self.path,
            conditions: self.conditions,
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Allow(perm, expr) => write!(f, "allow {}: if {};", perm, expr),
            Condition::Deny(perm, expr) => write!(f, "deny {}: if {};", perm, expr),
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::Read => write!(f, "read"),
            Permission::Write => write!(f, "write"),
            Permission::Create => write!(f, "create"),
            Permission::Update => write!(f, "update"),
            Permission::Delete => write!(f, "delete"),
            Permission::List => write!(f, "list"),
            Permission::Get => write!(f, "get"),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::True => write!(f, "true"),
            Expression::False => write!(f, "false"),
            Expression::IsAuthenticated => write!(f, "request.auth != null"),
            Expression::IsOwner(field) => write!(f, "request.auth.uid == resource.data.{}", field),
            Expression::HasRole(role) => write!(f, "request.auth.token.role == '{}'", role),
            Expression::FieldEquals(field, value) => write!(f, "resource.data.{} == '{}'", field, value),
            Expression::And(left, right) => write!(f, "({} && {})", left, right),
            Expression::Or(left, right) => write!(f, "({} || {})", left, right),
            Expression::Not(expr) => write!(f, "!({})", expr),
            Expression::Custom(expr) => write!(f, "{}", expr),
        }
    }
}

pub fn common_rules() -> SecurityRules {
    let mut rules = SecurityRules::new();
    
    rules.add_rule(
        RuleBuilder::new("/users/{userId}")
            .allow_read_if(Expression::IsAuthenticated)
            .allow_write_if(Expression::IsOwner("userId".to_string()))
            .build()
    );
    
    rules.add_rule(
        RuleBuilder::new("/public/{document=**}")
            .public_read()
            .allow_write_if(Expression::HasRole("admin".to_string()))
            .build()
    );
    
    rules.add_rule(
        RuleBuilder::new("/admin/{document=**}")
            .allow_read_if(Expression::HasRole("admin".to_string()))
            .allow_write_if(Expression::HasRole("admin".to_string()))
            .build()
    );
    
    rules
}