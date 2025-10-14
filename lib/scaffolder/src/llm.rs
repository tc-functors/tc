use std::collections::HashMap;
use kit::*;
use kit as u;
use inquire::{
    Text,
};

use serde::{Deserialize, Serialize};

fn prompt(text: &str) -> String {
    let lines = v![
"You are an expert at creating tc topologies. tc is a graph-based, serverless application composer that uses high-level abstractions called Cloud Functors to define application architecture without infrastructure details.",
"",
"Here is the application you need to create a tc topology for:",
"",
"<application_description>",
format!("{text}"),
"</application_description>",
"",
"## Your Task",
"Create a complete, production-ready tc topology YAML file for the described application. Focus on business logic and relationships, not infrastructure implementation details.",
"",
"## Core Principles",
"- **Provider-agnostic**: Definitions work across cloud providers",
"- **Composable**: Functions chain together naturally  ",
"- **Namespaced**: All entities are isolated within their topology",
"- **Stateless**: No external state management required",
"- **Business-focused**: Abstract away infrastructure complexity",
"",
"## Available Entities",
"",
"### Routes",
"HTTP endpoints that trigger functions:",
"```yaml",
"routes:",
"  /api/endpoint:",
"    method: POST|GET|PUT|DELETE|PATCH",
"    function: function-name",
"```",
"",
"### Functions",
"Serverless compute units that can chain together:",
"```yaml",
"functions:",
"  function-name:",
"    function: next-function    # Chain to another function",
"    page: page-name         # Webapp",
"    event: event-name         # Trigger an event",
"    queue: queue-name         # Send to queue",
"    channel: channel-name     # Send to WebSocket",
"```",
"",
"### Events",
"Asynchronous event notifications:",
"```yaml",
"events:",
"  EventName:",
"    function: handler-function",
"    queue: queue-name",
"    channel: channel-name",
"```",
"",
"### Queues",
"Message queues for async processing:",
"```yaml",
"queues:",
"  queue-name:",
"    function: processor-function",
"    batch_size: 10",
"```",
"",
"### Channels",
"WebSocket connections for real-time communication:",
"```yaml",
"channels:",
"  channel-name:",
"    type: websocket",
"    function: handler-function",
"```",
"",
"### Mutations (Optional)",
"GraphQL mutations:",
"```yaml",
"mutations:",
"  mutationName:",
"    function: handler-function",
"```",
"",
"## Design Process",
"",
"1. **Analyze the application description** to understand the core use case and requirements",
"2. **Identify user interactions** and map them to HTTP routes",
"3. **Design the data flow** through chained functions",
"4. **Determine state requirements** for data persistence",
"5. **Add asynchronous processing** using events and queues where appropriate",
"6. **Include real-time features** using channels if needed",
"7. **Ensure proper composition** with logical function chaining",
"",
"## Naming Conventions",
"- Use kebab-case for all entity names",
"- Functions: action-based (`validate-input`, `process-payment`, `send-notification`)",
"- Events: past tense (`OrderCreated`, `PaymentProcessed`, `UserRegistered`)",
"- Queues: purpose-based (`processing-queue`, `email-queue`, `retry-queue`)",
"",
"## Design Patterns to Consider",
"",
"**Function Chaining**: For sequential operations",
"```yaml",
"functions:",
"  validate-input:",
"    function: process-data",
"  process-data:",
"    function: save-results",
"```",
"",
"**Event-Driven**: For fire-and-forget operations",
"```yaml",
"functions:",
"  main-handler:",
"    event: DataProcessed",
"events:",
"  DataProcessed:",
"    function: notification-handler",
"```",
"",
"**Async Processing**: For heavy or batch operations",
"```yaml",
"functions:",
"  api-handler:",
"    queue: processing-queue",
"queues:",
"  processing-queue:",
"    function: worker",
"```",
"",
"**Real-Time Updates**: For user notifications",
"```yaml",
"functions:",
"  update-handler:",
"    channel: live-updates",
"channels:",
"  live-updates:",
"    handler: default",
"```",
"",
"## Validation Requirements",
"- All referenced entities must be defined",
"- Routes must have valid HTTP methods",
"- Function chains must be logical and non-circular",
"",
"## Output Requirements",
"",
"Provide your response in the following format:",
"",
"1. **Brief architecture explanation** (2-3 sentences describing the overall design approach)",
"",
"2. **Complete topology YAML** inside <topology> tags with:",
"   - Descriptive topology name in kebab-case",
"   - All necessary entities properly defined",
"   - Logical composition and flow",
"   - Inline comments for key design decisions",
"",
"3. **Key design decisions** (bullet points explaining major architectural choices)",
"",
"## Example Structure",
"```yaml",
"name: descriptive-topology-name",
"",
"routes:",
"  # HTTP endpoints",
"",
"functions:",
"  # Business logic functions with chaining",
"",
"events:",
"  # Async event definitions",
"",
"queues:",
"  # Background processing queues",
"",
"channels:",
"  # Real-time WebSocket channels",
"",
"mutations:",
"  # GraphQL mutations (if needed)",
"```",
"",
"Remember: Focus on business logic and relationships, not infrastructure details. Create a topology that is composable, maintainable, and follows tc best practices"];
    lines.join("\n")

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

async fn send(text: &str) -> String {
    let payload =  Payload::new(text);
    let p = serde_json::to_string(&payload).unwrap();
    let url = "https://api.anthropic.com/v1/messages";
    let res = u::http_post(url, headers(), p).await.unwrap();
    let response: Response = serde_json::from_value(res).unwrap();
    let res = response.content.into_iter().nth(0).unwrap().text;
    res
}

pub async fn scaffold(dir: &str) {
    match std::env::var("CLAUDE_API_KEY") {
        Ok(_) => (),
        Err(_) => panic!("Please set CLAUDE_API_KEY env var")
    }

    let desc = Text::new("Architecture Description:").prompt();
    let text = &desc.unwrap();
    println!("Asking Claude...");
    let response = send(text).await;
    println!("Generating topology.yml...");
    let code = llm_toolkit::extract_markdown_block_with_lang(&response, "yaml").unwrap();
    println!("{}", &code);
    let topo_file = format!("{}/topology.yml", dir);
    u::write_str(&topo_file, &code);
}
