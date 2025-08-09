use std::collections::HashMap;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use serde_json::{Map, Value};
use crate::collections::{CollectionSchema, FieldInfo};
use crate::error::FirebaseError;

#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: String,
    pub required: bool,
    pub description: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TuiForm {
    pub fields: Vec<FormField>,
    pub current_field: usize,
    pub title: String,
    pub help_text: Vec<String>,
}

impl TuiForm {
    pub fn new(title: String) -> Self {
        Self {
            fields: Vec::new(),
            current_field: 0,
            title,
            help_text: vec![
                "↑/↓ - Navigate fields".to_string(),
                "Enter - Edit field".to_string(),
                "Tab - Next field".to_string(),
                "Ctrl+S - Save and submit".to_string(),
                "Ctrl+C/Esc - Cancel".to_string(),
            ],
        }
    }

    pub fn from_schema(collection_name: &str, schema: &CollectionSchema) -> Self {
        let mut form = Self::new(format!("Create Document in '{}'", collection_name));
        
        for field in &schema.fields {
            form.fields.push(FormField {
                name: field.name.clone(),
                field_type: field.field_type.clone(),
                value: field.sample_values.first().map(|v| v.clone()).unwrap_or_default(),
                required: field.is_required,
                description: Some(format!("{} values, {} unique, frequency: {}", 
                    field.sample_values.len(), field.unique_values, field.frequency)),
                default_value: field.sample_values.first().map(|v| v.clone()),
            });
        }
        
        if form.fields.is_empty() {
            // If no schema available, create basic fields
            form.add_field(FormField {
                name: "name".to_string(),
                field_type: "string".to_string(),
                value: String::new(),
                required: false,
                description: Some("Document field".to_string()),
                default_value: None,
            });
        }
        
        form
    }

    pub fn from_existing_data(collection_name: &str, document_id: &str, data: &Value) -> Self {
        let mut form = Self::new(format!("Update Document '{}' in '{}'", document_id, collection_name));
        
        if let Value::Object(map) = data {
            for (key, value) in map {
                form.fields.push(FormField {
                    name: key.clone(),
                    field_type: infer_field_type(value),
                    value: format_value_for_editing(value),
                    required: false,
                    description: Some(format!("Current: {}", format_value_display(value))),
                    default_value: Some(format_value_for_editing(value)),
                });
            }
        }
        
        form
    }

    pub fn add_field(&mut self, field: FormField) {
        self.fields.push(field);
    }

    pub fn to_json(&self) -> Result<Value, FirebaseError> {
        let mut map = Map::new();
        
        for field in &self.fields {
            if !field.value.trim().is_empty() {
                let parsed_value = parse_field_value(&field.value, &field.field_type)?;
                map.insert(field.name.clone(), parsed_value);
            } else if field.required {
                return Err(FirebaseError::ValidationError(
                    format!("Required field '{}' cannot be empty", field.name)
                ));
            }
        }
        
        Ok(Value::Object(map))
    }

    pub fn run(&mut self) -> Result<Option<Value>, FirebaseError> {
        // Set up terminal
        enable_raw_mode().map_err(|e| FirebaseError::ConfigError(format!("Terminal setup failed: {}", e)))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| FirebaseError::ConfigError(format!("Terminal setup failed: {}", e)))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|e| FirebaseError::ConfigError(format!("Terminal setup failed: {}", e)))?;

        let result = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode().map_err(|e| FirebaseError::ConfigError(format!("Terminal restore failed: {}", e)))?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).map_err(|e| FirebaseError::ConfigError(format!("Terminal restore failed: {}", e)))?;
        terminal.show_cursor()
            .map_err(|e| FirebaseError::ConfigError(format!("Terminal restore failed: {}", e)))?;

        result
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<Option<Value>, FirebaseError> {
        let mut editing_field = false;
        let mut edit_buffer = String::new();

        loop {
            terminal.draw(|f| self.ui(f, editing_field, &edit_buffer))
                .map_err(|e| FirebaseError::ConfigError(format!("Terminal draw failed: {}", e)))?;

            if let Event::Key(key) = event::read()
                .map_err(|e| FirebaseError::ConfigError(format!("Event read failed: {}", e)))?
            {
                if key.kind == KeyEventKind::Press {
                    if editing_field {
                        match key.code {
                            KeyCode::Enter => {
                                if self.current_field < self.fields.len() {
                                    self.fields[self.current_field].value = edit_buffer.clone();
                                }
                                editing_field = false;
                                edit_buffer.clear();
                            }
                            KeyCode::Esc => {
                                editing_field = false;
                                edit_buffer.clear();
                            }
                            KeyCode::Char(c) => {
                                edit_buffer.push(c);
                            }
                            KeyCode::Backspace => {
                                edit_buffer.pop();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                return Ok(None); // Cancelled
                            }
                            KeyCode::Char('s') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                return Ok(Some(self.to_json()?)); // Submit
                            }
                            KeyCode::Esc => {
                                return Ok(None); // Cancelled
                            }
                            KeyCode::Up => {
                                if self.current_field > 0 {
                                    self.current_field -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.current_field < self.fields.len().saturating_sub(1) {
                                    self.current_field += 1;
                                }
                            }
                            KeyCode::Tab => {
                                self.current_field = (self.current_field + 1) % self.fields.len().max(1);
                            }
                            KeyCode::Enter => {
                                if self.current_field < self.fields.len() {
                                    editing_field = true;
                                    edit_buffer = self.fields[self.current_field].value.clone();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn ui(&self, f: &mut Frame, editing_field: bool, edit_buffer: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(10),    // Form
                Constraint::Length(self.help_text.len() as u16 + 2), // Help
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new(Text::from(self.title.as_str()))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .wrap(Wrap { trim: true });
        f.render_widget(title, chunks[0]);

        // Form fields
        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(2); self.fields.len().max(1)])
            .split(chunks[1]);

        for (i, field) in self.fields.iter().enumerate() {
            let is_current = i == self.current_field;
            let style = if is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let border_style = if is_current {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };

            let display_value = if editing_field && is_current {
                edit_buffer
            } else {
                &field.value
            };

            let field_text = if field.required {
                format!("{} *", field.name)
            } else {
                field.name.clone()
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::styled(field_text, Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" ({})", field.field_type), Style::default().fg(Color::Gray)),
                ])
            ];

            if let Some(desc) = &field.description {
                lines.push(Line::from(Span::styled(desc, Style::default().fg(Color::Gray))));
            }

            lines.push(Line::from(Span::styled(display_value, style)));

            let paragraph = Paragraph::new(Text::from(lines))
                .block(Block::default().borders(Borders::ALL).border_style(border_style))
                .wrap(Wrap { trim: false });

            if i < form_chunks.len() {
                f.render_widget(paragraph, form_chunks[i]);
                
                if editing_field && is_current {
                    // Show cursor
                    let cursor_x = form_chunks[i].x + display_value.len() as u16 + 1;
                    let cursor_y = form_chunks[i].y + 3; // Account for border and description
                    f.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }

        // Help text
        let help_items: Vec<ListItem> = self.help_text
            .iter()
            .map(|h| ListItem::new(Line::from(Span::styled(h, Style::default().fg(Color::Gray)))))
            .collect();

        let help = List::new(help_items)
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help, chunks[2]);
    }
}

fn infer_field_type(value: &Value) -> String {
    match value {
        Value::String(_) => "string".to_string(),
        Value::Number(n) => {
            if n.is_i64() {
                "integer".to_string()
            } else {
                "number".to_string()
            }
        }
        Value::Bool(_) => "boolean".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
        Value::Null => "string".to_string(),
    }
}

fn format_value_for_editing(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
        Value::Null => String::new(),
    }
}

fn format_value_display(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::Array(arr) => format!("Array({} items)", arr.len()),
        Value::Object(obj) => format!("Object({} fields)", obj.len()),
        _ => format_value_for_editing(value),
    }
}

fn parse_field_value(value_str: &str, field_type: &str) -> Result<Value, FirebaseError> {
    if value_str.trim().is_empty() {
        return Ok(Value::Null);
    }

    match field_type {
        "string" => Ok(Value::String(value_str.to_string())),
        "integer" => {
            value_str.parse::<i64>()
                .map(|n| Value::Number(n.into()))
                .map_err(|_| FirebaseError::ValidationError(format!("Invalid integer: {}", value_str)))
        }
        "number" => {
            value_str.parse::<f64>()
                .map(|n| Value::Number(serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0))))
                .map_err(|_| FirebaseError::ValidationError(format!("Invalid number: {}", value_str)))
        }
        "boolean" => {
            match value_str.to_lowercase().as_str() {
                "true" | "yes" | "1" => Ok(Value::Bool(true)),
                "false" | "no" | "0" => Ok(Value::Bool(false)),
                _ => Err(FirebaseError::ValidationError(format!("Invalid boolean: {} (use true/false)", value_str)))
            }
        }
        "array" | "object" => {
            serde_json::from_str(value_str)
                .map_err(|_| FirebaseError::ValidationError(format!("Invalid JSON: {}", value_str)))
        }
        "timestamp" => {
            // Try parsing as ISO 8601 timestamp
            if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(value_str) {
                Ok(Value::String(datetime.to_rfc3339()))
            } else {
                // Fall back to current timestamp if "now" or empty
                if value_str.to_lowercase() == "now" {
                    Ok(Value::String(chrono::Utc::now().to_rfc3339()))
                } else {
                    Err(FirebaseError::ValidationError(format!("Invalid timestamp: {} (use ISO 8601 format or 'now')", value_str)))
                }
            }
        }
        _ => Ok(Value::String(value_str.to_string())), // Default to string
    }
}