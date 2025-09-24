use gemini_rust::{FunctionCallingMode, FunctionDeclaration, Gemini, Message, Schema, Tool};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");

    // Create client
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    println!("--- Nested Schema Example ---");

    // Create a nested address schema
    let address_schema = Schema::object()
        .with_property("street", Schema::string("Street address"), true)
        .with_property("city", Schema::string("City name"), true)
        .with_property("state", Schema::string("State or province"), true)
        .with_property(
            "postal_code",
            Schema::string("Postal or ZIP code").with_pattern(r"^\d{5}(-\d{4})?$"),
            true,
        )
        .with_property("country", Schema::string("Country name"), false);

    // Create a contact info schema
    let contact_schema = Schema::object()
        .with_property(
            "email",
            Schema::string("Email address").with_pattern(r"^[^\s@]+@[^\s@]+\.[^\s@]+$"),
            true,
        )
        .with_property(
            "phone",
            Schema::string("Phone number").with_pattern(r"^\+?[\d\s\-\(\)]+$"),
            false,
        )
        .with_property("website", Schema::string("Website URL"), false);

    // Create a preferences schema with enum and nested objects
    let preferences_schema = Schema::object()
        .with_property(
            "communication_method",
            Schema::enum_type(
                "Preferred communication method",
                ["email", "phone", "sms", "mail"],
            ),
            true,
        )
        .with_property(
            "notification_settings",
            Schema::object()
                .with_property(
                    "marketing",
                    Schema::boolean("Receive marketing emails"),
                    false,
                )
                .with_property("updates", Schema::boolean("Receive product updates"), true)
                .with_property(
                    "reminders",
                    Schema::boolean("Receive appointment reminders"),
                    true,
                ),
            false,
        )
        .with_property(
            "languages",
            Schema::array(
                "Preferred languages",
                Schema::string("Language code (e.g., 'en', 'es')"),
            ),
            false,
        );

    // Create a complex user management function with deeply nested schemas
    let create_user = FunctionDeclaration::new(
        "create_user",
        "Create a new user profile with detailed information",
        Schema::object()
            .with_property(
                "personal_info",
                Schema::object()
                    .with_property("first_name", Schema::string("First name"), true)
                    .with_property("last_name", Schema::string("Last name"), true)
                    .with_property(
                        "date_of_birth",
                        Schema::string("Date of birth in YYYY-MM-DD format")
                            .with_pattern(r"^\d{4}-\d{2}-\d{2}$"),
                        false,
                    )
                    .with_property(
                        "gender",
                        Schema::enum_type(
                            "Gender",
                            ["male", "female", "non-binary", "prefer-not-to-say"],
                        ),
                        false,
                    ),
                true,
            )
            .with_property("address", address_schema.clone(), true)
            .with_property("contact", contact_schema, true)
            .with_property("preferences", preferences_schema, false)
            .with_property(
                "tags",
                Schema::array("User tags for categorization", Schema::string("Tag name")),
                false,
            )
            .with_property(
                "metadata",
                Schema::object()
                    .with_property("source", Schema::string("Registration source"), false)
                    .with_property("referrer", Schema::string("Referrer information"), false)
                    .with_property(
                        "custom_fields",
                        Schema::object(), // Empty object for custom key-value pairs
                        false,
                    ),
                false,
            ),
    );

    // Create another function that uses array of objects
    let search_users = FunctionDeclaration::new(
        "search_users",
        "Search for users based on various criteria",
        Schema::object()
            .with_property(
                "filters",
                Schema::array(
                    "Search filters to apply",
                    Schema::object()
                        .with_property("field", Schema::string("Field name to filter on"), true)
                        .with_property(
                            "operator",
                            Schema::enum_type(
                                "Comparison operator",
                                [
                                    "equals",
                                    "contains",
                                    "starts_with",
                                    "greater_than",
                                    "less_than",
                                ],
                            ),
                            true,
                        )
                        .with_property("value", Schema::string("Value to compare against"), true),
                ),
                true,
            )
            .with_property(
                "sort",
                Schema::object()
                    .with_property("field", Schema::string("Field to sort by"), true)
                    .with_property(
                        "direction",
                        Schema::enum_type("Sort direction", ["asc", "desc"]),
                        false,
                    ),
                false,
            )
            .with_property(
                "pagination",
                Schema::object()
                    .with_property(
                        "page",
                        Schema::integer("Page number (1-based)").with_range(Some(1.0), None),
                        false,
                    )
                    .with_property(
                        "limit",
                        Schema::integer("Items per page").with_range(Some(1.0), Some(100.0)),
                        false,
                    ),
                false,
            ),
    );

    // Create a function that demonstrates complex response schema
    let get_user_analytics = FunctionDeclaration::new(
        "get_user_analytics",
        "Get detailed analytics for user engagement",
        Schema::object()
            .with_property("user_id", Schema::string("User identifier"), true)
            .with_property(
                "date_range",
                Schema::object()
                    .with_property(
                        "start_date",
                        Schema::string("Start date (YYYY-MM-DD)"),
                        true,
                    )
                    .with_property("end_date", Schema::string("End date (YYYY-MM-DD)"), true),
                true,
            )
            .with_property(
                "metrics",
                Schema::array(
                    "Metrics to include in the report",
                    Schema::enum_type(
                        "Available metrics",
                        [
                            "page_views",
                            "session_duration",
                            "bounce_rate",
                            "conversion_rate",
                            "click_through_rate",
                            "engagement_score",
                        ],
                    ),
                ),
                false,
            ),
    )
    .with_response(
        Schema::object()
            .with_property("user_id", Schema::string("User identifier"), true)
            .with_property(
                "analytics",
                Schema::object()
                    .with_property(
                        "summary",
                        Schema::object()
                            .with_property(
                                "total_sessions",
                                Schema::integer("Total number of sessions"),
                                true,
                            )
                            .with_property(
                                "average_session_duration",
                                Schema::number("Average session duration in minutes"),
                                true,
                            )
                            .with_property(
                                "total_page_views",
                                Schema::integer("Total page views"),
                                true,
                            ),
                        true,
                    )
                    .with_property(
                        "daily_breakdown",
                        Schema::array(
                            "Daily analytics data",
                            Schema::object()
                                .with_property("date", Schema::string("Date (YYYY-MM-DD)"), true)
                                .with_property(
                                    "sessions",
                                    Schema::integer("Number of sessions"),
                                    true,
                                )
                                .with_property(
                                    "page_views",
                                    Schema::integer("Number of page views"),
                                    true,
                                )
                                .with_property(
                                    "duration",
                                    Schema::number("Total session duration"),
                                    true,
                                ),
                        ),
                        true,
                    )
                    .with_property(
                        "conversion_funnel",
                        Schema::array(
                            "Conversion funnel stages",
                            Schema::object()
                                .with_property("stage", Schema::string("Funnel stage name"), true)
                                .with_property(
                                    "users",
                                    Schema::integer("Number of users at this stage"),
                                    true,
                                )
                                .with_property(
                                    "conversion_rate",
                                    Schema::number("Conversion rate to next stage"),
                                    false,
                                ),
                        ),
                        false,
                    ),
                true,
            )
            .with_property(
                "generated_at",
                Schema::string("Report generation timestamp"),
                true,
            ),
    );

    // Create a tool with all the nested schema functions
    let user_management_tool =
        Tool::with_functions(vec![create_user, search_users, get_user_analytics]);

    // Create a request to demonstrate the nested schemas
    let response = client
        .generate_content()
        .with_system_prompt(
            "You are a helpful assistant that can manage user profiles and analytics. \
            When asked to create a user profile, use the provided information to call \
            the create_user function with all the appropriate nested data structures."
        )
        .with_user_message(
            "I want to create a comprehensive user profile for John Doe. \
            He lives at 123 Main St, San Francisco, CA 94102, USA. \
            His email is john.doe@example.com and phone is +1-555-123-4567. \
            He prefers email communication, wants product updates but no marketing emails. \
            He speaks English and Spanish. Please create his profile with appropriate tags and metadata."
        )
        .with_tool(user_management_tool.clone())
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;

    // Execute the request
    println!("\nSending request with nested schema functions...");

    // Process function calls
    if let Some(function_call) = response.function_calls().first() {
        println!("Function Call: {}", function_call.name);
        println!("Arguments (showing nested schema structure):");
        println!("{}", serde_json::to_string_pretty(&function_call.args)?);

        // Simulate function execution for create_user
        if function_call.name == "create_user" {
            let mock_response = json!({
                "user_id": "user_12345",
                "status": "created",
                "message": "User profile created successfully with nested data structure",
                "profile": {
                    "personal_info": {
                        "first_name": "John",
                        "last_name": "Doe"
                    },
                    "address": {
                        "street": "123 Main St",
                        "city": "San Francisco",
                        "state": "CA",
                        "postal_code": "94102",
                        "country": "USA"
                    },
                    "contact": {
                        "email": "john.doe@example.com",
                        "phone": "+1-555-123-4567"
                    },
                    "preferences": {
                        "communication_method": "email",
                        "notification_settings": {
                            "marketing": false,
                            "updates": true,
                            "reminders": true
                        },
                        "languages": ["en", "es"]
                    },
                    "tags": ["new_user", "multilingual"],
                    "metadata": {
                        "source": "manual_creation",
                        "created_at": "2024-01-15T10:30:00Z"
                    }
                }
            });

            println!("\nMock function response (demonstrating nested response structure):");
            println!("{}", serde_json::to_string_pretty(&mock_response)?);

            // Send function response back to continue conversation
            let followup_response = client
                .generate_content()
                .with_system_prompt(
                    "You are a helpful assistant that can manage user profiles and analytics."
                )
                .with_user_message(
                    "I want to create a comprehensive user profile for John Doe. \
                    He lives at 123 Main St, San Francisco, CA 94102, USA. \
                    His email is john.doe@example.com and phone is +1-555-123-4567. \
                    He prefers email communication, wants product updates but no marketing emails. \
                    He speaks English and Spanish. Please create his profile with appropriate tags and metadata."
                )
                .with_tool(user_management_tool.clone())
                .with_message(Message {
                    content: gemini_rust::Content::function_call((*function_call).clone()),
                    role: gemini_rust::Role::Model,
                })
                .with_function_response_str(&function_call.name, mock_response.to_string())?
                .execute()
                .await?;

            println!("\nFinal Response: {}", followup_response.text());
        }
    } else {
        println!("Response: {}", response.text());
    }

    println!("\n--- Nested Schema Example completed ---");
    println!("This example demonstrates:");
    println!("1. Nested object schemas (address, contact, preferences)");
    println!("2. Array schemas with complex item types");
    println!("3. Enum schemas for constrained values");
    println!("4. Schema validation with patterns (regex)");
    println!("5. Range validation for numbers");
    println!("6. Complex response schemas");
    println!("7. Multiple levels of nesting in function parameters");

    Ok(())
}
