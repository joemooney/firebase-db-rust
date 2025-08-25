use firebase_db::tui_form::{TuiForm, FormField};

fn main() {
    println!("Testing Compact TUI Form Validation\n");
    
    // Create a test form
    let mut form = TuiForm::new("Test User Creation".to_string());
    
    // Add fields with different types
    form.add_field(FormField {
        name: "user_id".to_string(),
        field_type: "string".to_string(),
        value: "user123".to_string(),
        required: true,
        description: Some("User ID".to_string()),
        default_value: None,
    });
    
    form.add_field(FormField {
        name: "age".to_string(),
        field_type: "integer".to_string(),
        value: "25".to_string(),
        required: true,
        description: Some("User's age".to_string()),
        default_value: None,
    });
    
    form.add_field(FormField {
        name: "balance".to_string(),
        field_type: "number".to_string(),
        value: "1234.56".to_string(),
        required: false,
        description: Some("Account balance".to_string()),
        default_value: None,
    });
    
    form.add_field(FormField {
        name: "is_active".to_string(),
        field_type: "boolean".to_string(),
        value: "true".to_string(),
        required: false,
        description: Some("Account status".to_string()),
        default_value: None,
    });
    
    // Test valid form submission
    println!("Test: Valid form data");
    match form.to_json() {
        Ok(json) => println!("✅ Form is valid!\n   Data: {:?}\n", json),
        Err(e) => println!("❌ Unexpected error: {}\n", e),
    }
    
    // Test with invalid integer
    println!("Test: Invalid integer field");
    form.fields[1].value = "not_a_number".to_string();
    match form.to_json() {
        Ok(_) => println!("❌ Should have failed validation\n"),
        Err(e) => println!("✅ Correctly caught error: {}\n", e),
    }
    
    // Test with empty required field
    println!("Test: Empty required field");
    form.fields[0].value = "".to_string();
    form.fields[1].value = "25".to_string(); // Fix the age field
    match form.to_json() {
        Ok(_) => println!("❌ Should have failed validation\n"),
        Err(e) => println!("✅ Correctly caught error: {}\n", e),
    }
    
    // Test boolean variations
    println!("Test: Boolean field variations");
    form.fields[0].value = "user123".to_string(); // Fix required field
    for val in ["true", "false", "yes", "no", "1", "0", "invalid"] {
        form.fields[3].value = val.to_string();
        match form.to_json() {
            Ok(_) => println!("   '{}' - Valid ✓", val),
            Err(_) => println!("   '{}' - Invalid ✗", val),
        }
    }
    
    println!("\n✨ Form validation testing complete!");
    println!("\nThe new compact form features:");
    println!("• All fields visible in single view");
    println!("• Real-time validation with helpful error messages");
    println!("• Tab/Shift+Tab/↑/↓ navigation between fields");
    println!("• ← → arrow keys to move cursor within field");
    println!("• Home/End keys to jump to start/end of field");
    println!("• Visual cursor (│) shows exact edit position");
    println!("• Ctrl+S to SUBMIT form (clearly indicated)");
    println!("• Ctrl+C or Esc to CANCEL");
    println!("• Required fields marked with *");
    println!("• Type hints for each field");
    println!("• Error messages with examples of valid input");
}