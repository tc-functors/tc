# Specification

<!-- toc -->

## topology.yml

```yaml
name: <topology-name>
infra: path/to/infra  [optional]
nodes: [optional]
	ignore:
		- <unrelated-node>
functions: [optional]
	shared:
		- ../shared/function1
	    - ../shared/function2
events: [optional]
  consumes:
    <EventName>:
      producer: <producerName>
    <EventName>:
      producer: <producerName>
  produces:
    <EventName>:
      consumer: consumerName
routes: [optional]
  <name>:
	kind: rest|http|websocket
    gateway: <API-NAME>
    authorizer: <AUTHORIZER-NAME>
	proxy: none|default|<function-name>
    timeout: INT
    async: BOOL
    method: POST|GET|DELETE
    path: STRING

flow: ./states.json | <definition>  [optional]
```

`infra` is either an absolute or relative path to the infrastructure configs (vars, roles etc). This field is optional and tc tries best to discover the infrastructure path in the current git repo.

`events`, `routes` and `flow` are optional.

`flow` can contain a path to a step-function definition or an inline definition. tc automatically namespaces any inlined or external flow definition.

## function.json

function.json file in the function directory is optional. `tc` infers the language and build instructions from the function code. However, for custom options, add a function.json that looks like the following


```json
{
  "name": "<Function Name>",
  "description": "<Description of the function>",
  "runtime": {
    "lang": "<python3.10 | ruby3.2 | janet | rust>",
    "package_type": "<zip | image>",
    "handler": "lambda_function.handler",
    "layers": ["python-lambda-base"],
	"extensions": ["tracer", "s3-logger"]
  },
  "assets": {
	"MODEL_PATH": "",
	"DEPS_PATH": ""
  },

  "tasks": {
    "build": "zip -9 lambda.zip lambda_function.py",
    "clean": "rm *.zip",
	"test": "pytest ."
  }
}
```
