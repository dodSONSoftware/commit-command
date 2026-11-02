fn main() {
    println!(
        "{}",
        serde_json::json!({"request":gitfleet_core::application::Application::request_schema(),"response":gitfleet_core::application::Application::response_schema()})
    );
}
