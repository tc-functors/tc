use std::collections::HashMap;
use kit::*;
use kit as u;
use serde::{Deserialize, Serialize};


fn prompt(text: &str) -> String {
    format!(r#"
You are an expert at creating tc topologies. tc is a graph-based, serverless application composer that uses high-level abstractions called Cloud Functors to define application architecture without infrastructure details.\n\nHere is the application you need to create a tc topology for:\n\n<application_description>\n{text}\n</application_description>\n\n## Your Task\nCreate a complete, production-ready tc topology YAML file for the described application. Focus on business logic and relationships, not infrastructure implementation details.\n\n## Core Principles\n- **Provider-agnostic**: Definitions work across cloud providers\n- **Composable**: Functions chain together naturally  \n- **Namespaced**: All entities are isolated within their topology\n- **Stateless**: No external state management required\n- **Business-focused**: Abstract away infrastructure complexity\n\n## Available Entities\n\n### Routes\nHTTP endpoints that trigger functions:\n```yaml\nroutes:\n  /api/endpoint:\n    method: POST|GET|PUT|DELETE|PATCH\n    function: function-name\n```\n\n### Functions\nServerless compute units that can chain together:\n```yaml\nfunctions:\n  function-name:\n    function: next-function    # Chain to another function\n    state: state-name         # Access a state store\n    event: event-name         # Trigger an event\n    queue: queue-name         # Send to queue\n    channel: channel-name     # Send to WebSocket\n    runtime: python3.11|nodejs18|go1.x|rust\n    memory: 128|256|512|1024  # MB\n    timeout: 5|10|30|60|300   # seconds\n```\n\n### Events\nAsynchronous event notifications:\n```yaml\nevents:\n  EventName:\n    function: handler-function\n    queue: queue-name\n    channel: channel-name\n```\n\n### States\nKey-value stores for persistent data:\n```yaml\nstates:\n  state-name:\n    type: dynamodb\n    primary_key: keyName\n    sort_key: sortKeyName  # Optional\n```\n\n### Queues\nMessage queues for async processing:\n```yaml\nqueues:\n  queue-name:\n    function: processor-function\n    batch_size: 10\n    visibility_timeout: 300\n```\n\n### Channels\nWebSocket connections for real-time communication:\n```yaml\nchannels:\n  channel-name:\n    type: websocket\n    function: handler-function\n```\n\n### Mutations (Optional)\nGraphQL mutations:\n```yaml\nmutations:\n  mutationName:\n    function: handler-function\n```\n\n## Design Process\n\n1. **Analyze the application description** to understand the core use case and requirements\n2. **Identify user interactions** and map them to HTTP routes\n3. **Design the data flow** through chained functions\n4. **Determine state requirements** for data persistence\n5. **Add asynchronous processing** using events and queues where appropriate\n6. **Include real-time features** using channels if needed\n7. **Ensure proper composition** with logical function chaining\n\n## Naming Conventions\n- Use kebab-case for all entity names\n- Functions: action-based (`validate-input`, `process-payment`, `send-notification`)\n- Events: past tense (`OrderCreated`, `PaymentProcessed`, `UserRegistered`)\n- States: content-based (`user-profiles`, `order-data`, `analytics-cache`)\n- Queues: purpose-based (`processing-queue`, `email-queue`, `retry-queue`)\n\n## Design Patterns to Consider\n\n**Function Chaining**: For sequential operations\n```yaml\nfunctions:\n  validate-input:\n    function: process-data\n  process-data:\n    function: save-results\n```\n\n**Event-Driven**: For fire-and-forget operations\n```yaml\nfunctions:\n  main-handler:\n    event: DataProcessed\nevents:\n  DataProcessed:\n    function: notification-handler\n```\n\n**Async Processing**: For heavy or batch operations\n```yaml\nfunctions:\n  api-handler:\n    queue: processing-queue\nqueues:\n  processing-queue:\n    function: worker\n```\n\n**Real-Time Updates**: For user notifications\n```yaml\nfunctions:\n  update-handler:\n    channel: live-updates\nchannels:\n  live-updates:\n    type: websocket\n```\n\n## Validation Requirements\n- All referenced entities must be defined\n- Routes must have valid HTTP methods\n- Function chains must be logical and non-circular\n\n## Output Requirements\n\nProvide your response in the following format:\n\n1. **Brief architecture explanation** (2-3 sentences describing the overall design approach)\n\n2. **Complete topology YAML** inside <topology> tags with:\n   - Descriptive topology name in kebab-case\n   - All necessary entities properly defined\n   - Logical composition and flow\n   - Reasonable defaults for memory/timeout\n   - Inline comments for key design decisions\n\n3. **Key design decisions** (bullet points explaining major architectural choices)\n\n## Example Structure\n```yaml\nname: descriptive-topology-name\n\nroutes:\n  # HTTP endpoints\n\nfunctions:\n  # Business logic functions with chaining\n\nevents:\n  # Async event definitions\n\nstates:\n  # Data persistence stores\n\nqueues:\n  # Background processing queues\n\nchannels:\n  # Real-time WebSocket channels\n\nmutations:\n  # GraphQL mutations (if needed)\n```\n\nRemember: Focus on business logic and relationships, not infrastructure details. Create a topology that is composable, maintainable, and follows tc best practices.
"#)

}



fn headers() -> HashMap<String, String> {
    let api_key = match std::env::var("CLAUDE_API_KEY") {
        Ok(p) => p,
        Err(_) => String::from("")
    };
    let mut h = HashMap::new();
    h.insert(s!("content-type"), s!("application/json"));
    h.insert(s!("anthropic-version"), s!("2023-06-01"));
    h.insert(s!("x-api-key"), api_key);
    h.insert(s!("accept"), s!("application/json"));
    h.insert(
        s!("user-agent"),
        s!("libcurl/7.64.1 r-curl/4.3.2 httr/1.4.2"),
    );
    h
}

#[derive(Deserialize, Serialize, Debug)]
struct Content {
    r#type: String,
    text: String
}

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: Vec<Content>
}

#[derive(Serialize, Debug)]
struct Payload {
    model: String,
    max_tokens: u16,
    messages: Vec<Message>
}


impl Payload {

    fn new(text: &str) -> Payload {

        let content = Content {
            r#type: s!("text"),
            text: prompt(text)
        };

        let message = Message {
            role: s!("user"),
            content: vec![content]
        };

        Payload {
            model: s!("claude-sonnet-4-5-20250929"),
            max_tokens: 20000,
            messages: vec![message]
        }
    }
}

#[derive(Deserialize)]
struct Response {
    content: Vec<Content>
}

pub async fn send(text: &str) -> String {
    let payload =  Payload::new(text);
    let p = serde_json::to_string(&payload).unwrap();
    let url = "https://api.anthropic.com/v1/messages";
    let res = u::http_post(url, headers(), p).await.unwrap();
    let response: Response = serde_json::from_value(res).unwrap();
    let text = response.content.into_iter().nth(0).unwrap().text;
    text
}
