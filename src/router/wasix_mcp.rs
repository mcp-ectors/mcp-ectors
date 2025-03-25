use chrono::{DateTime, Utc};
use mcp_spec::{content::EmbeddedResource, prompt::{self}, ImageContent, ResourceContents};
use serde_json::{Map, Value as JsonValue};

use super::wasm_router::exports::wasix::{self, mcp::router::{PromptMessageContent, Value}};


impl From<wasix::mcp::router::Tool> for mcp_spec::Tool {
    fn from(tool: wasix::mcp::router::Tool) -> Self {
        mcp_spec::Tool {
            name: tool.name,
            description: tool.description,
            input_schema: value_to_json(tool.input_schema),
            // Convert additional fields here as needed.
        }
    }
}

impl From<wasix::mcp::router::McpResource> for mcp_spec::Resource {
    fn from(resource: wasix::mcp::router::McpResource) -> Self {
        mcp_spec::Resource {
            name: resource.name,
            description: resource.description,
            uri: resource.uri,
            mime_type: resource.mime_type,
            annotations: resource.annotations.map(|annotations: wasix::mcp::router::Annotations| {
                mcp_spec::Annotations::from(annotations) // Use the correct conversion here
            }),

        }
    }
}

impl From<wasix::mcp::router::ResourceContents> for mcp_spec::ResourceContents {
    fn from(resource: wasix::mcp::router::ResourceContents) -> Self {
        match resource {
            wasix::mcp::router::ResourceContents::Text(contents) => 
                mcp_spec::ResourceContents::TextResourceContents { 
                    uri: contents.uri, 
                    mime_type: contents.mime_type, 
                    text: contents.text,
                },
            wasix::mcp::router::ResourceContents::Blob(contents) => 
                mcp_spec::ResourceContents::BlobResourceContents {
                    uri: contents.uri,
                    mime_type: contents.mime_type,
                    blob: contents.blob,  // Assuming `data` is the field for the binary contents
                },
        }
    }
}

impl From<wasix::mcp::router::Annotations> for mcp_spec::Annotations {
    fn from(annotations: wasix::mcp::router::Annotations) -> Self {
        mcp_spec::Annotations {
            audience: Some(annotations.audience.map(|args| {
                args.into_iter()
                    .map(|arg| mcp_spec::Role::from(arg)) // Convert `Role` correctly
                    .collect::<Vec<mcp_spec::Role>>()
            }).unwrap_or_default()),
            priority: annotations.priority,
            timestamp: string_to_datetime(annotations.timestamp), // Ensure string_to_datetime works correctly
        }
    }
}


impl From<wasix::mcp::router::Role> for mcp_spec::Role {
    fn from(role: wasix::mcp::router::Role) -> Self {
        match role {
            wasix::mcp::router::Role::Assistant => mcp_spec::Role::Assistant,
            wasix::mcp::router::Role::User => mcp_spec::Role::User,
        }
    }
}

impl From<wasix::mcp::router::Prompt> for mcp_spec::prompt::Prompt {
    fn from(prompt: wasix::mcp::router::Prompt) -> Self {
        mcp_spec::prompt::Prompt {
            name: prompt.name,
            description: prompt.description,
            arguments: prompt.arguments.map(|args| {
                Some(args.into_iter()
                    .map(|arg| mcp_spec::prompt::PromptArgument::from(arg))
                    .collect::<Vec<mcp_spec::prompt::PromptArgument>>()
                )
            }).unwrap_or_default(),
        }
    }
}

impl From<wasix::mcp::router::PromptArgument> for mcp_spec::prompt::PromptArgument {
    fn from(prompt: wasix::mcp::router::PromptArgument) -> Self {
        mcp_spec::prompt::PromptArgument {
            name: prompt.name,
            description: prompt.description,
            required: prompt.required,
        }
    }
}

impl From<wasix::mcp::router::TextContent> for mcp_spec::TextContent {
    fn from(text: wasix::mcp::router::TextContent) -> Self {
        mcp_spec::TextContent {
            text: text.text,
            annotations: match text.annotations{
                Some(anno) => Some(mcp_spec::Annotations::from(anno)),
                None => None,
            }
        }
    }
}

impl From<wasix::mcp::router::Content> for mcp_spec::Content {
    fn from(content: wasix::mcp::router::Content) -> Self {
        match content {
            wasix::mcp::router::Content::Text(text_content) => mcp_spec::Content::Text(mcp_spec::TextContent::from(text_content)),
            wasix::mcp::router::Content::Image(image_content) => mcp_spec::Content::Image(mcp_spec::ImageContent::from(image_content)),
            wasix::mcp::router::Content::Embedded(embedded_resource) => mcp_spec::Content::Resource(mcp_spec::content::EmbeddedResource::from(embedded_resource)),
        }
    }
}


impl From<wasix::mcp::router::ServerCapabilities> for mcp_spec::protocol::ServerCapabilities {
    fn from(cap: wasix::mcp::router::ServerCapabilities) -> Self {
        mcp_spec::protocol::ServerCapabilities {
            prompts: match cap.prompts{
                Some(prompts) => {
                    Some(mcp_spec::protocol::PromptsCapability{
                        list_changed: prompts.list_changed
                    })
                },
                None => None,
            },
            resources: match cap.resources{
                Some(res) => {
                    Some(mcp_spec::protocol::ResourcesCapability{
                        list_changed: res.list_changed,
                        subscribe: res.subscribe,
                    })
                },
                None => None,
            },
            tools: match cap.tools{
                Some(tools) => {
                    Some(mcp_spec::protocol::ToolsCapability{
                        list_changed: tools.list_changed
                    })
                },
                None => None,
            },
        }
    }
}

impl From<wasix::mcp::router::ImageContent> for mcp_spec::ImageContent { 
    fn from(image: wasix::mcp::router::ImageContent) -> Self {
        ImageContent{
            data: image.data,
            mime_type: image.mime_type,
            annotations: match image.annotations{
                Some(anno) => Some(mcp_spec::Annotations::from(anno)),
                None => None,
            }
        }
    }
}
impl From<wasix::mcp::router::EmbeddedResource> for mcp_spec::content::EmbeddedResource { 
    fn from(embedded: wasix::mcp::router::EmbeddedResource) -> Self {
        EmbeddedResource{ 
            resource: match embedded.resource_contents {
                wasix::mcp::router::ResourceContents::Text(text) 
                    => ResourceContents::TextResourceContents { 
                        uri: text.uri, 
                        mime_type: text.mime_type, 
                        text: text.text, 
                    },
                wasix::mcp::router::ResourceContents::Blob(blob) 
                    => ResourceContents::BlobResourceContents { 
                        uri: blob.uri, 
                        mime_type: blob.mime_type,
                        blob: blob.blob 
                    },
            }, 
            annotations: match embedded.annotations {
                Some(annotations) => 
                    Some(mcp_spec::Annotations::from(annotations)),
                None => todo!(),
            }, 
        }
    }
}

impl From<wasix::mcp::router::PromptMessage> for mcp_spec::prompt::PromptMessage {
    fn from(prompt: wasix::mcp::router::PromptMessage) -> Self {
        mcp_spec::prompt::PromptMessage {
            role: match prompt.role {
                wasix::mcp::router::PromptMessageRole::User => mcp_spec::prompt::PromptMessageRole::User,
                wasix::mcp::router::PromptMessageRole::Assistant => mcp_spec::prompt::PromptMessageRole::Assistant,
            },
            content: match prompt.content {
                PromptMessageContent::Text(text) => mcp_spec::prompt::PromptMessageContent::Text { text: text.text },
                PromptMessageContent::Image(image) => 
                    mcp_spec::prompt::PromptMessageContent::Image { 
                        image: ImageContent::from(image)
                    },
                PromptMessageContent::McpResource(embedded) => 
                    prompt::PromptMessageContent::Resource { 
                        resource: EmbeddedResource { 
                            resource: mcp_spec::ResourceContents::from(embedded.resource_contents), 
                            annotations: match embedded.annotations {
                                Some(annotations) => 
                                    Some(mcp_spec::Annotations::from(annotations)),
                                None => todo!(),
                            }, 
                        }
                    },
                },
        }
    }
}

pub fn string_to_datetime(date_str: Option<String>) -> Option<DateTime<Utc>> {
    match date_str
    {
        Some(date) => {
            match DateTime::parse_from_rfc3339(&date) {
                Ok(dt) => Some(dt.with_timezone(&Utc)),
                Err(_) => None, // Return None if parsing fails
            }
        },
        None => None,
    }

}

pub fn value_to_json(val: wasix::mcp::router::Value) -> JsonValue {
    // Attempt to parse the data as JSON.
    // If parsing fails, fallback to using the string value.
    let parsed: JsonValue = serde_json::from_str(&val.data).unwrap_or(JsonValue::String(val.data));
    // Create a JSON object with the key/value.
    let mut map = Map::new();
    map.insert(val.key, parsed);
    JsonValue::Object(map)
}

pub fn json_to_value(val: JsonValue) -> Option<wasix::mcp::router::Value> {
    // Ensure it's an object, and extract the first key-value pair
    if let JsonValue::Object(map) = val {
        for (key, value) in map {
            match value.as_str() {
                Some(value) => {
                     // Return the first key-value pair found
                    let mcp_value = Value{key, data: value.to_string()};
                    return Some(mcp_value)
                }
                None => return None,
            }
        }
    }
    None // Return None if the value is not an object or empty
    
}

#[cfg(test)]
mod tests {
    use crate::router::wasm_router::exports::wasix::mcp::router::Annotations;

    use super::*;
    use serde_json::json;
    use chrono::{DateTime, Utc};

    // Test conversion from wasix::mcp::router::Tool to mcp_spec::Tool
    #[test]
    fn test_tool_conversion() {
        let wasix_tool = wasix::mcp::router::Tool {
            name: "Test Tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: wasix::mcp::router::Value {
                key: "input".to_string(),
                data: "{\"type\":\"object\"}".to_string(),
            },
        };

        let mcp_tool: mcp_spec::Tool = wasix_tool.into();
        assert_eq!(mcp_tool.name, "Test Tool");
        assert_eq!(mcp_tool.description, "A test tool");
        assert_eq!(
            mcp_tool.input_schema,
            json!({"input": {"type": "object"}})
        );
    }

    // Test conversion from wasix::mcp::router::Annotations to mcp_spec::Annotations
    #[test]
    fn test_annotations_conversion() {
        let wasix_annotations = wasix::mcp::router::Annotations {
            audience: Some(vec![wasix::mcp::router::Role::Assistant]),
            priority: Some(5.0),
            timestamp: Some("2022-03-01T12:00:00Z".to_string()),
        };

        let mcp_annotations: mcp_spec::Annotations = wasix_annotations.into();
        assert_eq!(mcp_annotations.priority, Some(5.0));
        assert_eq!(
            mcp_annotations.audience.unwrap(),
            vec![
                mcp_spec::Role::from(wasix::mcp::router::Role::Assistant)
            ]
        );
        assert!(mcp_annotations.timestamp.is_some());
    }

    // Test conversion from wasix::mcp::router::ResourceContents to mcp_spec::ResourceContents
    #[test]
    fn test_resource_contents_conversion() {
        let text_resource = wasix::mcp::router::ResourceContents::Text(wasix::mcp::router::TextResourceContents {
            uri: "http://example.com".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: "Test content".to_string(),
        });

        let mcp_resource: mcp_spec::ResourceContents = text_resource.into();
        match mcp_resource {
            mcp_spec::ResourceContents::TextResourceContents { uri, mime_type, text } => {
                assert_eq!(uri, "http://example.com");
                assert_eq!(mime_type, Some("text/plain".to_string()));
                assert_eq!(text, "Test content");
            }
            _ => panic!("Expected TextResourceContents"),
        }
    }

    // Test string_to_datetime function
    #[test]
    fn test_string_to_datetime() {
        let valid_date = Some("2022-03-01T12:00:00Z".to_string());
        let invalid_date = Some("invalid date".to_string());
        let empty_date = None;

        // Valid date
        let datetime: Option<DateTime<Utc>> = string_to_datetime(valid_date);
        assert!(datetime.is_some());

        // Invalid date
        let datetime_invalid: Option<DateTime<Utc>> = string_to_datetime(invalid_date);
        assert!(datetime_invalid.is_none());

        // None date
        let datetime_none: Option<DateTime<Utc>> = string_to_datetime(empty_date);
        assert!(datetime_none.is_none());
    }

    // Test value_to_json function
    #[test]
    fn test_value_to_json() {
        let val = wasix::mcp::router::Value {
            key: "test_key".to_string(),
            data: "{\"type\":\"object\"}".to_string(),
        };

        let json_val = value_to_json(val);
        assert_eq!(
            json_val,
            json!({
                "test_key": {"type": "object"}
            })
        );
    }

    // Test json_to_value function
    #[test]
    fn test_json_to_value() {
        let json_val = json!({
            "test_key": "test_value"
        });

        let mcp_value = json_to_value(json_val);
        assert!(mcp_value.is_some());
        let mcp_value = mcp_value.unwrap();
        assert_eq!(mcp_value.key, "test_key");
        assert_eq!(mcp_value.data, "test_value");
    }

    // Test conversion from wasix::mcp::router::PromptMessage to mcp_spec::prompt::PromptMessage
    #[test]
    fn test_prompt_message_conversion() {
        let wasix_prompt_message = wasix::mcp::router::PromptMessage {
            role: wasix::mcp::router::PromptMessageRole::User,
            content: PromptMessageContent::Text(
                wasix::mcp::router::TextContent{ 
                    text: "This is a prompt message".to_string(),
                    annotations: Some(Annotations{ 
                        audience: Some(vec![wasix::mcp::router::Role::Assistant]),
                        priority: Some(5.0), 
                        timestamp: None 
                    }),
                },
            ),
        };

        let mcp_prompt_message: mcp_spec::prompt::PromptMessage = wasix_prompt_message.into();
        match mcp_prompt_message.content {
            mcp_spec::prompt::PromptMessageContent::Text { text } => {
                assert_eq!(text, "This is a prompt message");
            }
            _ => panic!("Expected PromptMessageContent::Text"),
        }
    }
}
