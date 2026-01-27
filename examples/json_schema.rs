use display_error_chain::DisplayErrorChain;
use gemini_rust::{Content, FunctionCallingMode, FunctionDeclaration, Gemini, Message, Role, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::process::ExitCode;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct BookRecommendation {
    title: String,
    author: String,
    year: u32,
    genre: String,
    rating: f32,
    summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct SearchCriteria {
    genre: String,
    min_year: Option<u32>,
    max_year: Option<u32>,
    min_rating: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct SearchResults {
    books: Vec<BookRecommendation>,
    total_count: u32,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    match do_main().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let error_chain = DisplayErrorChain::new(e.as_ref());
            tracing::error!(error.debug = ?e, error.chained = %error_chain, "execution failed");
            ExitCode::FAILURE
        }
    }
}

async fn do_main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY environment variable not set");
    let client = Gemini::new(api_key).expect("unable to create Gemini API client");

    info!("=== Example 1: Using response_json_schema for structured output ===");

    let schema = json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "The title of the book"
            },
            "author": {
                "type": "string",
                "description": "The author's name"
            },
            "year": {
                "type": "integer",
                "description": "Publication year"
            },
            "genre": {
                "type": "string",
                "description": "The genre of the book"
            },
            "rating": {
                "type": "number",
                "description": "Rating from 1.0 to 5.0"
            },
            "summary": {
                "type": "string",
                "description": "A brief summary of the book"
            }
        },
        "required": ["title", "author", "year", "genre", "rating", "summary"]
    });

    let response = client
        .generate_content()
        .with_system_prompt("You recommend books in strict JSON format.")
        .with_user_message("Recommend a classic science fiction book.")
        .with_response_mime_type("application/json")
        .with_response_json_schema(schema)
        .execute()
        .await?;

    info!(
        response = response.text(),
        "structured JSON response received"
    );

    let book: BookRecommendation = serde_json::from_str(&response.text())?;
    info!(
        title = book.title,
        author = book.author,
        year = book.year,
        genre = book.genre,
        rating = book.rating,
        "parsed book recommendation"
    );

    info!(
        "\n=== Example 2: Using parametersJsonSchema and responseJsonSchema for function tools ==="
    );

    let search_books = FunctionDeclaration::new(
        "search_books",
        "Search for books by genre and optional year and rating filters",
        None,
    )
    .with_parameters_json_schema::<SearchCriteria>()
    .with_response_json_schema::<SearchResults>();

    let tool = Tool::with_functions(vec![search_books]);

    let response = client
        .generate_content()
        .with_system_prompt(
            "You help users find books. When asked about books, use the search_books function.",
        )
        .with_user_message(
            "Find me some fantasy books published after 2010 with a rating above 4.0",
        )
        .with_tool(tool.clone())
        .with_function_calling_mode(FunctionCallingMode::Any)
        .execute()
        .await?;

    if let Some(function_call) = response.function_calls().first() {
        info!(
            function_name = function_call.name,
            args = ?function_call.args,
            "function call received"
        );

        let criteria: SearchCriteria = serde_json::from_value(function_call.args.clone())?;
        info!(
            genre = criteria.genre,
            min_year = ?criteria.min_year,
            max_year = ?criteria.max_year,
            min_rating = ?criteria.min_rating,
            "parsed search criteria"
        );

        let search_results = SearchResults {
            books: vec![
                BookRecommendation {
                    title: "The Name of the Wind".to_string(),
                    author: "Patrick Rothfuss".to_string(),
                    year: 2007,
                    genre: "Fantasy".to_string(),
                    rating: 4.5,
                    summary: "A legendary hero tells his life story.".to_string(),
                },
                BookRecommendation {
                    title: "The Way of Kings".to_string(),
                    author: "Brandon Sanderson".to_string(),
                    year: 2010,
                    genre: "Fantasy".to_string(),
                    rating: 4.6,
                    summary: "Epic fantasy in a world of storms and magic.".to_string(),
                },
            ],
            total_count: 2,
        };

        let mut conversation = client.generate_content();

        conversation = conversation
            .with_system_prompt(
                "You help users find books. When asked about books, use the search_books function.",
            )
            .with_user_message(
                "Find me some fantasy books published after 2010 with a rating above 4.0",
            );

        let model_content = Content::function_call((*function_call).clone());
        let model_message = Message {
            content: model_content,
            role: Role::Model,
        };
        conversation = conversation.with_message(model_message);

        conversation = conversation.with_function_response("search_books", search_results)?;

        let final_response = conversation.with_tool(tool).execute().await?;

        info!(
            response = final_response.text(),
            "final response with book recommendations"
        );
    } else {
        info!("no function calls in response");
        info!(response = response.text(), "direct response received");
    }

    Ok(())
}
