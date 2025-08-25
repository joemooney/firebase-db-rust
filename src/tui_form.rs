use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use serde_json::{Map, Value};
use crate::collections::{CollectionSchema, AutoFieldType};
use crate::error::FirebaseError;

#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: String,
    pub required: bool,
    pub description: Option<String>,
    pub default_value: Option<String>,
    pub auto_field: Option<AutoFieldType>,
}

#[derive(Debug, Clone)]
pub struct TuiForm {
    pub fields: Vec<FormField>,
    pub current_field: usize,
    pub title: String,
    pub help_text: Vec<String>,
    pub error_message: Option<String>,
    pub compact_mode: bool,
    cursor_position: usize,  // Track cursor position within the current field
    button_selected: bool,  // Track if a button is selected
    selected_button: usize,  // 0 = Add/Submit, 1 = Cancel
    schema: Option<CollectionSchema>,  // Store schema for automatic field generation
}

impl TuiForm {
    pub fn new(title: String) -> Self {
        Self {
            fields: Vec::new(),
            current_field: 0,
            title,
            help_text: vec![
                "Tab/↓ - Next | ↑ - Previous | ← → - Move cursor | Enter on button - Submit/Cancel".to_string(),
            ],
            error_message: None,
            compact_mode: true,
            cursor_position: 0,
            button_selected: false,
            selected_button: 0,
            schema: None,
        }
    }

    pub fn from_schema(collection_name: &str, schema: &CollectionSchema) -> Self {
        let mut form = Self::new(format!("Create Document in '{}'", collection_name));
        form.schema = Some(schema.clone());
        
        for field in &schema.fields {
            // Skip automatic fields - they will be generated automatically
            if field.auto_field.is_some() {
                continue;
            }
            
            // Start with empty values by default
            let empty_value = if field.field_type == "array" {
                "[]".to_string()
            } else {
                String::new()
            };
            
            // Create example text from sample values
            let example_text = if !field.sample_values.is_empty() {
                let example = field.sample_values.first().unwrap();
                format!("example: {}", example)
            } else {
                match field.field_type.as_str() {
                    "string" => "example: \"John Doe\"".to_string(),
                    "integer" => "example: 42".to_string(),
                    "number" => "example: 3.14".to_string(),
                    "boolean" => "example: true".to_string(),
                    "array" => "example: [\"item1\", \"item2\"]".to_string(),
                    "object" => "example: {\"key\": \"value\"}".to_string(),
                    "timestamp" => "example: 2024-01-01T12:00:00Z or 'now'".to_string(),
                    _ => "example: value".to_string(),
                }
            };
            
            form.fields.push(FormField {
                name: field.name.clone(),
                field_type: field.field_type.clone(),
                value: empty_value.clone(),
                required: field.is_required,
                description: Some(example_text),
                default_value: Some(empty_value),
                auto_field: None,
            });
        }
        
        if form.fields.is_empty() {
            // If no schema available, create basic fields
            form.add_field(FormField {
                name: "name".to_string(),
                field_type: "string".to_string(),
                value: String::new(),
                required: false,
                description: Some("example: \"John Doe\"".to_string()),
                default_value: Some(String::new()),
                auto_field: None,
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
                    description: Some(format!("current: {} | example: {}", 
                        format_value_display(value),
                        get_example_for_type(&infer_field_type(value))
                    )),
                    default_value: Some(format_value_for_editing(value)),
                    auto_field: None,
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
        
        // Add user-entered field values
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
        
        // Add automatic field values from schema
        if let Some(schema) = &self.schema {
            for field in &schema.fields {
                if let Some(auto_type) = &field.auto_field {
                    let auto_value = auto_type.generate_value();
                    map.insert(field.name.clone(), auto_value);
                }
            }
        }
        
        Ok(Value::Object(map))
    }

    fn validate_and_set_field(&mut self, index: usize, value: &str) -> Result<(), String> {
        if index >= self.fields.len() {
            return Ok(());
        }
        
        let field = &self.fields[index];
        
        // Check if required field is empty
        if field.required && value.trim().is_empty() {
            return Err(format!("Required field '{}' cannot be empty", field.name));
        }
        
        // Skip validation if field is empty and not required
        if !field.required && value.trim().is_empty() {
            self.fields[index].value = value.to_string();
            return Ok(());
        }
        
        // Validate based on field type
        match field.field_type.as_str() {
            "integer" => {
                if let Err(_) = value.parse::<i64>() {
                    return Err(format!("Invalid integer. Example: 42, -10, 0"));
                }
            }
            "number" => {
                if let Err(_) = value.parse::<f64>() {
                    return Err(format!("Invalid number. Example: 3.14, -0.5, 42"));
                }
            }
            "boolean" => {
                let lower = value.to_lowercase();
                if !["true", "false", "yes", "no", "1", "0"].contains(&lower.as_str()) {
                    return Err(format!("Invalid boolean. Use: true, false, yes, no, 1, or 0"));
                }
            }
            "array" => {
                if !value.trim().is_empty() && serde_json::from_str::<Value>(value).is_err() {
                    return Err(format!("Invalid JSON array. Example: [1, 2, 3] or [\"a\", \"b\"]"));
                }
            }
            "object" => {
                if !value.trim().is_empty() && serde_json::from_str::<Value>(value).is_err() {
                    return Err(format!("Invalid JSON object. Example: {{\"key\": \"value\"}}"));
                }
            }
            "timestamp" => {
                if !value.trim().is_empty() && value.to_lowercase() != "now" {
                    if chrono::DateTime::parse_from_rfc3339(value).is_err() {
                        return Err(format!("Invalid timestamp. Use ISO format (2024-01-01T12:00:00Z) or 'now'"));
                    }
                }
            }
            _ => {} // String type, no validation needed
        }
        
        self.fields[index].value = value.to_string();
        Ok(())
    }
    
    fn validate_all_fields(&self) -> Result<(), String> {
        for field in &self.fields {
            if field.required && field.value.trim().is_empty() {
                return Err(format!("Required field '{}' is empty", field.name));
            }
        }
        Ok(())
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
        // Start in editing mode for the first field
        let mut editing_field = true;
        let mut edit_buffer = if !self.fields.is_empty() {
            self.fields[0].value.clone()
        } else {
            String::new()
        };
        self.cursor_position = edit_buffer.len();  // Start cursor at end of default value

        loop {
            terminal.draw(|f| self.ui(f, editing_field, &edit_buffer, self.cursor_position))
                .map_err(|e| FirebaseError::ConfigError(format!("Terminal draw failed: {}", e)))?;

            if let Event::Key(key) = event::read()
                .map_err(|e| FirebaseError::ConfigError(format!("Event read failed: {}", e)))?
            {
                if key.kind == KeyEventKind::Press {
                    if editing_field {
                        match key.code {
                            KeyCode::Tab => {
                                // Validate and save current field
                                if !self.button_selected && self.current_field < self.fields.len() {
                                    if let Err(e) = self.validate_and_set_field(self.current_field, &edit_buffer) {
                                        self.error_message = Some(e);
                                    } else {
                                        self.error_message = None;
                                        // Move to next field or button
                                        if self.current_field == self.fields.len() - 1 {
                                            // Move to buttons
                                            self.button_selected = true;
                                            self.selected_button = 0;
                                            editing_field = false;
                                        } else {
                                            self.current_field += 1;
                                            edit_buffer = self.fields[self.current_field].value.clone();
                                            self.cursor_position = edit_buffer.len();
                                        }
                                    }
                                } else if self.button_selected {
                                    // Switch between buttons
                                    self.selected_button = (self.selected_button + 1) % 2;
                                }
                            }
                            KeyCode::BackTab => {
                                if self.button_selected {
                                    if self.selected_button > 0 {
                                        self.selected_button -= 1;
                                    } else {
                                        // Move back to fields
                                        self.button_selected = false;
                                        self.current_field = self.fields.len().saturating_sub(1);
                                        editing_field = true;
                                        edit_buffer = self.fields[self.current_field].value.clone();
                                        self.cursor_position = edit_buffer.len();
                                    }
                                } else {
                                    // Save current field and move to previous
                                    if self.current_field < self.fields.len() {
                                        self.fields[self.current_field].value = edit_buffer.clone();
                                        self.error_message = None;
                                    }
                                    if self.current_field > 0 {
                                        self.current_field -= 1;
                                    } else {
                                        self.current_field = self.fields.len().saturating_sub(1);
                                    }
                                    edit_buffer = self.fields[self.current_field].value.clone();
                                    self.cursor_position = edit_buffer.len();
                                }
                            }
                            KeyCode::Up => {
                                if self.button_selected {
                                    // Move back to last field
                                    self.button_selected = false;
                                    self.current_field = self.fields.len().saturating_sub(1);
                                    editing_field = true;
                                    edit_buffer = self.fields[self.current_field].value.clone();
                                    self.cursor_position = edit_buffer.len();
                                } else {
                                    // Save current field and move up
                                    if self.current_field < self.fields.len() {
                                        self.fields[self.current_field].value = edit_buffer.clone();
                                        self.error_message = None;
                                    }
                                    if self.current_field > 0 {
                                        self.current_field -= 1;
                                        edit_buffer = self.fields[self.current_field].value.clone();
                                        self.cursor_position = edit_buffer.len();
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if !self.button_selected {
                                    // Validate and save current field, then move down
                                    if self.current_field < self.fields.len() {
                                        if let Err(e) = self.validate_and_set_field(self.current_field, &edit_buffer) {
                                            self.error_message = Some(e);
                                        } else {
                                            self.error_message = None;
                                            if self.current_field < self.fields.len().saturating_sub(1) {
                                                self.current_field += 1;
                                                edit_buffer = self.fields[self.current_field].value.clone();
                                                self.cursor_position = edit_buffer.len();
                                            } else {
                                                // Move to buttons
                                                self.button_selected = true;
                                                self.selected_button = 0;
                                                editing_field = false;
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if self.button_selected {
                                    // Handle button press
                                    if self.selected_button == 0 {
                                        // Submit button
                                        // Validate all required fields
                                        if let Err(e) = self.validate_all_fields() {
                                            self.error_message = Some(e);
                                        } else {
                                            return Ok(Some(self.to_json()?));
                                        }
                                    } else {
                                        // Cancel button
                                        return Ok(None);
                                    }
                                } else {
                                    // Move to next field like Down
                                    if self.current_field < self.fields.len() {
                                        if let Err(e) = self.validate_and_set_field(self.current_field, &edit_buffer) {
                                            self.error_message = Some(e);
                                        } else {
                                            self.error_message = None;
                                            if self.current_field < self.fields.len().saturating_sub(1) {
                                                self.current_field += 1;
                                                edit_buffer = self.fields[self.current_field].value.clone();
                                                self.cursor_position = edit_buffer.len();
                                            } else {
                                                // Move to buttons
                                                self.button_selected = true;
                                                self.selected_button = 0;
                                                editing_field = false;
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Left => {
                                if self.cursor_position > 0 {
                                    self.cursor_position -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.cursor_position < edit_buffer.len() {
                                    self.cursor_position += 1;
                                }
                            }
                            KeyCode::Home => {
                                self.cursor_position = 0;
                            }
                            KeyCode::End => {
                                self.cursor_position = edit_buffer.len();
                            }
                            KeyCode::Esc => {
                                return Ok(None); // Cancel
                            }
                            KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                return Ok(None); // Cancel
                            }
                            KeyCode::Char('s') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                // Validate current field before submitting
                                if self.current_field < self.fields.len() {
                                    if let Err(e) = self.validate_and_set_field(self.current_field, &edit_buffer) {
                                        self.error_message = Some(e);
                                    } else {
                                        self.error_message = None;
                                        // Validate all required fields
                                        if let Err(e) = self.validate_all_fields() {
                                            self.error_message = Some(e);
                                        } else {
                                            return Ok(Some(self.to_json()?));
                                        }
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                edit_buffer.insert(self.cursor_position, c);
                                self.cursor_position += 1;
                                self.error_message = None; // Clear error on typing
                            }
                            KeyCode::Backspace => {
                                if self.cursor_position > 0 && !edit_buffer.is_empty() {
                                    self.cursor_position -= 1;
                                    edit_buffer.remove(self.cursor_position);
                                    self.error_message = None; // Clear error on typing
                                }
                            }
                            KeyCode::Delete => {
                                if self.cursor_position < edit_buffer.len() {
                                    edit_buffer.remove(self.cursor_position);
                                    self.error_message = None; // Clear error on typing
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // Handle button navigation when not editing
                        if self.button_selected {
                            match key.code {
                                KeyCode::Left => {
                                    self.selected_button = 0; // Move to Add button
                                }
                                KeyCode::Right => {
                                    self.selected_button = 1; // Move to Cancel button
                                }
                                KeyCode::Tab => {
                                    self.selected_button = (self.selected_button + 1) % 2;
                                }
                                KeyCode::Enter => {
                                    if self.selected_button == 0 {
                                        // Submit
                                        if let Err(e) = self.validate_all_fields() {
                                            self.error_message = Some(e);
                                        } else {
                                            return Ok(Some(self.to_json()?));
                                        }
                                    } else {
                                        // Cancel
                                        return Ok(None);
                                    }
                                }
                                KeyCode::Up => {
                                    // Go back to fields
                                    self.button_selected = false;
                                    self.current_field = self.fields.len().saturating_sub(1);
                                    editing_field = true;
                                    edit_buffer = self.fields[self.current_field].value.clone();
                                    self.cursor_position = edit_buffer.len();
                                }
                                KeyCode::Esc | KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                    return Ok(None);
                                }
                                _ => {}
                            }
                        } else {
                            // Should not reach here in compact mode as we start in editing mode
                            editing_field = true;
                            if self.current_field < self.fields.len() {
                                edit_buffer = self.fields[self.current_field].value.clone();
                            }
                        }
                    }
                }
            }
        }
    }

    fn ui(&self, f: &mut Frame, editing_field: bool, edit_buffer: &str, cursor_pos: usize) {
        // Calculate heights based on content
        let error_height = if self.error_message.is_some() { 3 } else { 0 };
        let form_height = (self.fields.len() as u16 * 2) + 2; // 2 lines per field + borders
        let button_height = 3; // Height for buttons
        let help_height = 2; // Always show help bar
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(form_height), // Compact form
                Constraint::Length(button_height), // Buttons
                Constraint::Length(error_height), // Error message (if any)
                Constraint::Length(help_height), // Help bar (always visible)
                Constraint::Min(0), // Remaining space
            ])
            .split(f.area());

        // Title with form status
        let title_text = if editing_field {
            format!("{} - Field {}/{}", self.title, self.current_field + 1, self.fields.len())
        } else {
            self.title.clone()
        };
        let title = Paragraph::new(Text::from(title_text))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .wrap(Wrap { trim: true });
        f.render_widget(title, chunks[0]);

        // Compact form - all fields in one block
        let mut form_lines = Vec::new();
        
        for (i, field) in self.fields.iter().enumerate() {
            let is_current = i == self.current_field;
            
            let field_name = if field.required {
                format!("{} *", field.name)
            } else {
                field.name.clone()
            };
            
            let display_value = if editing_field && is_current {
                edit_buffer
            } else {
                &field.value
            };
            
            // Field label line
            let label_style = if is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            // Create field label with example
            let mut label_spans = vec![
                Span::styled(format!("{:<20}", field_name), label_style),
                Span::styled(format!(" ({})", field.field_type), Style::default().fg(Color::DarkGray)),
            ];
            
            // Add example text if available
            if let Some(desc) = &field.description {
                label_spans.push(Span::styled(format!(" | {}", desc), Style::default().fg(Color::DarkGray)));
            }
            
            form_lines.push(Line::from(label_spans));
            
            // Field value line with proper cursor
            let value_style = if is_current && editing_field {
                Style::default().fg(Color::Cyan)
            } else if is_current {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            let value_prefix = if is_current { "→ " } else { "  " };
            
            // Build the value line with cursor at the right position
            if is_current && editing_field {
                let mut value_spans = vec![Span::raw(value_prefix)];
                
                // Split the text at cursor position
                let (before_cursor, after_cursor) = display_value.split_at(cursor_pos.min(display_value.len()));
                
                if !before_cursor.is_empty() {
                    value_spans.push(Span::styled(before_cursor, value_style));
                }
                
                // Add cursor
                value_spans.push(Span::styled("│", Style::default().fg(Color::Cyan).add_modifier(Modifier::RAPID_BLINK)));
                
                if !after_cursor.is_empty() {
                    value_spans.push(Span::styled(after_cursor, value_style));
                }
                
                form_lines.push(Line::from(value_spans));
            } else {
                form_lines.push(Line::from(vec![
                    Span::raw(value_prefix),
                    Span::styled(display_value, value_style),
                ]));
            }
        }
        
        let form_block = Paragraph::new(form_lines)
            .block(Block::default().borders(Borders::ALL).title("Fields"))
            .wrap(Wrap { trim: false });
        f.render_widget(form_block, chunks[1]);
        
        // Render buttons
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[2]);
        
        // Add/Submit button
        let add_style = if self.button_selected && self.selected_button == 0 {
            Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        };
        
        let add_button = Paragraph::new(Text::from(" [ ADD ] "))
            .style(add_style)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(add_button, button_chunks[0]);
        
        // Cancel button
        let cancel_style = if self.button_selected && self.selected_button == 1 {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };
        
        let cancel_button = Paragraph::new(Text::from(" [ CANCEL ] "))
            .style(cancel_style)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(cancel_button, button_chunks[1]);
        
        // Error message (if any)
        if let Some(error) = &self.error_message {
            let error_widget = Paragraph::new(Text::from(vec![
                Line::from(Span::styled(format!("⚠ {}", error), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)))
            ]))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(error_widget, chunks[3]);
        }
        
        // Help bar (always visible)
        let help_index = 4; // Always at index 4
        if help_index < chunks.len() {
            let help = Paragraph::new(Text::from(vec![
                Line::from(vec![
                    Span::styled("Navigation: ", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
                    Span::styled("Tab/↓", Style::default().fg(Color::Cyan)),
                    Span::styled(" Next | ", Style::default().fg(Color::DarkGray)),
                    Span::styled("↑/Shift+Tab", Style::default().fg(Color::Cyan)),
                    Span::styled(" Previous | ", Style::default().fg(Color::DarkGray)),
                    Span::styled("←→", Style::default().fg(Color::Cyan)),
                    Span::styled(" Move cursor | ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Ctrl+S", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(" SUBMIT | ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Ctrl+C/Esc", Style::default().fg(Color::Red)),
                    Span::styled(" CANCEL", Style::default().fg(Color::DarkGray)),
                ])
            ]))
                .block(Block::default().borders(Borders::TOP));
            f.render_widget(help, chunks[help_index]);
        }
    }
}

fn get_example_for_type(field_type: &str) -> String {
    match field_type {
        "string" => "\"John Doe\"".to_string(),
        "integer" => "42".to_string(),
        "number" => "3.14".to_string(),
        "boolean" => "true".to_string(),
        "array" => "[\"item1\", \"item2\"]".to_string(),
        "object" => "{\"key\": \"value\"}".to_string(),
        "timestamp" => "2024-01-01T12:00:00Z or 'now'".to_string(),
        _ => "value".to_string(),
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
        // Empty arrays should return empty array, not null
        if field_type == "array" {
            return Ok(Value::Array(Vec::new()));
        }
        return Ok(Value::Null);
    }

    match field_type {
        "string" => {
            // Don't require quotes for strings - take the value as-is
            Ok(Value::String(value_str.to_string()))
        }
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