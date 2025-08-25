use firebase_db::tui_form::{TuiForm, FormField};

fn main() {
    println!("Testing Compact TUI Form Validation\n");
    
    // Create a test form
    let mut form = TuiForm::new("Test User Creation".to_string());
    
    // Add fields with different types - now starting empty with examples
    form.add_field(FormField {
        name: "user_id".to_string(),
        field_type: "string".to_string(),
        value: "user123".to_string(), // Adding value for testing
        required: true,
        description: Some("example: \"David Wilson\"".to_string()),
        default_value: Some(String::new()),
    });
    
    form.add_field(FormField {
        name: "age".to_string(),
        field_type: "integer".to_string(),
        value: "25".to_string(), // Adding value for testing
        required: true,
        description: Some("example: 42".to_string()),
        default_value: Some(String::new()),
    });
    
    form.add_field(FormField {
        name: "balance".to_string(),
        field_type: "number".to_string(),
        value: "1234.56".to_string(), // Adding value for testing
        required: false,
        description: Some("example: 3.14".to_string()),
        default_value: Some(String::new()),
    });
    
    form.add_field(FormField {
        name: "tags".to_string(),
        field_type: "array".to_string(),
        value: "[]".to_string(), // Empty array by default
        required: false,
        description: Some("example: [\"premium\", \"verified\"]".to_string()),
        default_value: Some("[]".to_string()),
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
    
    // Test empty array field (should work)
    println!("Test: Empty array field");
    form.fields[0].value = "user123".to_string(); // Fix required field
    form.fields[1].value = "25".to_string(); // Fix age field
    form.fields[3].value = "[]".to_string(); // Empty array
    match form.to_json() {
        Ok(json) => println!("✅ Empty array works: {:?}", json.get("tags")),
        Err(e) => println!("❌ Empty array failed: {}", e),
    }
    
    // Test array with values
    println!("Test: Array with values");
    form.fields[3].value = "[\"premium\", \"verified\"]".to_string();
    match form.to_json() {
        Ok(json) => println!("✅ Array with values works: {:?}", json.get("tags")),
        Err(e) => println!("❌ Array with values failed: {}", e),
    }
    
    println!("\n✨ Form validation testing complete!");
    println!("\nThe new compact form features:");
    println!("• All fields visible in single view");
    println!("• Visual ADD and CANCEL buttons");
    println!("• Real-time validation with helpful error messages");
    println!("• Tab/Shift+Tab/↑/↓ navigation between fields and buttons");
    println!("• ← → arrow keys to move cursor within field or between buttons");
    println!("• Home/End keys to jump to start/end of field");
    println!("• Visual cursor (│) shows exact edit position");
    println!("• Enter on ADD button submits, Enter on CANCEL cancels");
    println!("• Ctrl+S to SUBMIT form, Ctrl+C/Esc to CANCEL (shortcuts)");
    println!("• Required fields marked with *");
    println!("• String fields don't need quotes - just type the text");
    println!("• Array fields default to empty [] instead of sample data");
    println!("• Examples shown in field labels (e.g., 'name * (string) | example: \"David Wilson\"')");
    println!("• Fields start empty by default, not pre-filled with sample data");
    println!("• Help bar always visible at bottom");
    println!("• Error messages with examples of valid input");
}