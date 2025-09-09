use crate::{FinishReason, FunctionCall, GenerationResponse, Part};
use serde_json::json;

#[test]
fn test_thought_signature_deserialization() {
    // Test JSON that includes thoughtSignature like in the provided API response
    let json_response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [
                        {
                            "functionCall": {
                                "name": "get_current_weather",
                                "args": {
                                    "location": "Kaohsiung Zuoying District"
                                }
                            },
                            "thoughtSignature": "CtwFAVSoXO4WSz0Ri3HddDzPQzsB8EaYsiQobiBKOzGOaAPM0d4DewrzUmhCnZbdboz+n+6v503fcy4epZC2bomn247laY6RHtKTc0UA8scj1DW/Y8w9AsfvjDX1adpIi043qjivTtowjxKAIesKoO69mFj6HTmGRI6sE1hamsIblZGZypowxnBQmxqJftl1aebB7kQN+MoYSeX+OU1z/8G+RXE+cb9cvwdAGIZjHXoGgEaIigYlrjTkZjRGBiI+gC2AcLNe32MHVla2/dmV8O7k8Cl45ksH+4srYABtmXLxjxwQK6s2bjVngvaRcBTCK4AUHiDb1j54n3Fls5J1i9k2sd6OcJYJuRlfwuxv2RMZ+V8zLdNthfSWtZwuJslkOD3uZCkEhO/hI6nAKcyuSokdAKtOw9g6LWORnEQoUJ+BaTVymN1tuJzbzrS9kPP5d3QJfFdQaILkk8CUdnGOEcngvlINN4MGNTQYN+0Au/JFWDWj33T5LZWkbDMp+yIpqFkZuRYwjW/9KOR6qFbxzvJyQcAKTxf0Sq7UfHTYBXTVp0/N4cDWRv+5DF0UOp+6emnPslCmaRK8JEGkmKkYXCzR6PpopfdzHHSDQHbNjjwr0h9ADZKehiB/cB1Jjy0oyBOM3HSHyuzcP8CO4NoAXOUM/VP5P41ys9TdeaPZAZ1E3cGQI4pifFVPdy3o33QSYqS1ce5Wxbeud06+d+sz2O7jJrfHMdgYpcO/2RcXQyK/GVIlDkWyxpYtBZhlkh3vLxPVmV/JJv5DQSS3YNTFSbfbwC8DtrI6YNFK5Vo07cl6mAY+U8b4ziFJk2HGuO27jq5EnhJE6v39HCfXTa8cKaLzpIURJSOs12S1rc3pqXdv4VBL6dp+Yjr8eQPxYRP93QzZMFXcYZ+Vc2H5mbnXbvTxVdYT7Qpu7aK1o6csSOMOx47NzZnOnlTWNJUxtU5UIZJ2JelOt/NsWnVJZY8D"
                        }
                    ],
                    "role": "model"
                },
                "finishReason": "STOP",
                "index": 0
            }
        ],
        "usageMetadata": {
            "promptTokenCount": 70,
            "candidatesTokenCount": 21,
            "totalTokenCount": 255,
            "thoughtsTokenCount": 164
        },
        "modelVersion": "gemini-2.5-pro",
        "responseId": "CCm8aJjzBaWh1MkP_cLEgQo"
    });

    // Test deserialization
    let response: GenerationResponse = serde_json::from_value(json_response).unwrap();

    // Verify basic structure
    assert_eq!(response.candidates.len(), 1);
    let candidate = &response.candidates[0];
    assert_eq!(candidate.finish_reason, Some(FinishReason::Stop));

    // Check content parts
    let parts = candidate.content.parts.as_ref().unwrap();
    assert_eq!(parts.len(), 1);

    // Verify the part is a function call with thought signature
    match &parts[0] {
        Part::FunctionCall {
            function_call,
            thought_signature,
        } => {
            assert_eq!(function_call.name, "get_current_weather");
            assert_eq!(function_call.args["location"], "Kaohsiung Zuoying District");

            // Verify thought signature is present and not empty
            assert!(thought_signature.is_some());
            let signature = thought_signature.as_ref().unwrap();
            assert!(!signature.is_empty());
            assert!(signature.starts_with("CtwFAVSoXO4WSz0Ri3HddDzPQzsB8EaYsiQobiBKOzGOaAPM"));
        }
        _ => panic!("Expected FunctionCall part"),
    }

    // Test the function_calls_with_thoughts method
    let function_calls_with_thoughts = response.function_calls_with_thoughts();
    assert_eq!(function_calls_with_thoughts.len(), 1);

    let (function_call, thought_signature) = &function_calls_with_thoughts[0];
    assert_eq!(function_call.name, "get_current_weather");
    assert!(thought_signature.is_some());

    // Test usage metadata with thinking tokens
    assert!(response.usage_metadata.is_some());
    let usage = response.usage_metadata.as_ref().unwrap();
    assert_eq!(usage.thoughts_token_count, Some(164));
}

#[test]
fn test_function_call_with_thought_signature() {
    // Test creating a FunctionCall with thought signature
    let function_call = FunctionCall::with_thought_signature(
        "test_function",
        json!({"param": "value"}),
        "test_thought_signature",
    );

    assert_eq!(function_call.name, "test_function");
    assert_eq!(function_call.args["param"], "value");
    assert_eq!(
        function_call.thought_signature,
        Some("test_thought_signature".to_string())
    );

    // Test serialization
    let serialized = serde_json::to_string(&function_call).unwrap();
    println!("Serialized FunctionCall: {}", serialized);

    // Test deserialization
    let deserialized: FunctionCall = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, function_call);
}

#[test]
fn test_function_call_without_thought_signature() {
    // Test creating a FunctionCall without thought signature (backward compatibility)
    let function_call = FunctionCall::new("test_function", json!({"param": "value"}));

    assert_eq!(function_call.name, "test_function");
    assert_eq!(function_call.args["param"], "value");
    assert_eq!(function_call.thought_signature, None);

    // Test serialization should not include thought_signature field when None
    let serialized = serde_json::to_string(&function_call).unwrap();
    println!("Serialized FunctionCall without thought: {}", serialized);
    assert!(!serialized.contains("thought_signature"));
}

#[test]
fn test_multi_turn_content_structure() {
    // Test that we can create proper multi-turn content structure for maintaining thought context
    use crate::{Content, Part, Role};

    // Simulate a function call with thought signature from first turn
    let function_call = FunctionCall::with_thought_signature(
        "get_weather",
        json!({"location": "Tokyo"}),
        "sample_thought_signature",
    );

    // Create model content with function call and thought signature
    let model_content = Content {
        parts: Some(vec![Part::FunctionCall {
            function_call: function_call.clone(),
            thought_signature: Some("sample_thought_signature".to_string()),
        }]),
        role: Some(Role::Model),
    };

    // Verify structure
    assert!(model_content.parts.is_some());
    assert_eq!(model_content.role, Some(Role::Model));

    // Test serialization of the complete structure first
    let serialized = serde_json::to_string(&model_content).unwrap();
    println!("Serialized multi-turn content: {}", serialized);

    // Verify it contains the thought signature
    assert!(serialized.contains("thoughtSignature"));
    assert!(serialized.contains("sample_thought_signature"));

    let parts = model_content.parts.unwrap();
    assert_eq!(parts.len(), 1);

    match &parts[0] {
        Part::FunctionCall {
            function_call,
            thought_signature,
        } => {
            assert_eq!(function_call.name, "get_weather");
            assert_eq!(
                thought_signature.as_ref().unwrap(),
                "sample_thought_signature"
            );
        }
        _ => panic!("Expected FunctionCall part"),
    }
}
