use inquire::Text;
use kit as u;
use kit::*;
use provider::{
    Auth,
    aws::bedrock,
};

const DEFAULT_BEDROCK_MODEL: &str = "us.anthropic.claude-sonnet-5-5";

pub fn default_tc_skills(text: &str) -> String {
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
        "- Do not have duplicates in functions, events, channels, mutations or queues",
        "- Avoid queues unless specified",
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
        "Remember: Focus on business logic and relationships, not infrastructure details. Create a topology that is composable, maintainable, and follows tc best practices"
    ];
    lines.join("\n")
}

fn process_response(dir: &str, response: &str) {
    println!("Generating topology.yml...");
    let code = llm_toolkit::extract_markdown_block_with_lang(&response, "yaml").unwrap();
    let topo_file = format!("{}/topology.yml", dir);
    u::write_str(&topo_file, &code);
}

fn make_prompt(skills: Option<String>) -> String {
    let f = match skills {
        Some(s) => s,
        None => "skills/topology.md".to_string(),
    };
    let text = if u::file_exists(&f) {
        u::slurp(&f)
    } else {
        let desc = Text::new("Architecture Description:").prompt();
        desc.unwrap()
    };
    default_tc_skills(&text)
}

pub async fn scaffold(auth: &Auth, skills: Option<String>, model: Option<String>, dir: &str) {
    let model = u::maybe_string(model, DEFAULT_BEDROCK_MODEL);
    println!("Using bedrock model {}", &model);
    let client = bedrock::make_client(auth).await;
    let prompt = make_prompt(skills);
    let response = bedrock::send(&client, &prompt, &model).await;
    process_response(dir, &response);
}
