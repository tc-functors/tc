# Getting Started

`tc` is a tool to build, compose, interactively develop and deploy serverless components. The following is an example of developing and creating a basic topology to enhance, transform and load data.

A Basic Example: ETL

Let's call the topology `etl`. Our requirements are (I'm just making this up):

- Trigger the topology via a REST API and an Eventbridge event
- Create nano functions that do minimal things by decoupling them from their dependencies.
- Build a linear flow of the `enhancer`, `transformer` and `loader` functions.
- Write the enhancer and transformer functions in `Python` and loader in `Ruby` (huh, don't ask me why)
- Build and use a `transformer` ML model (oh, it's 5GB in size and has weird build steps they say)
- Deploy and test the entire thing interactively in dev sandboxes and atomically in prod sandboxes

Let's get started!

1. Create a new directory called `etl` and add a file called topology.yml to it.

2. Add the following to topology.yml in the `etl` directory

	topology.yml
	```yaml
	name: etl
	routes:
		etl:
			gateway: api-test
			proxy: '{{namespace}}_enhancer_{{sandbox}}'
			kind: http
			timeout: 10
			async: false
			method: POST
			path: "/api/etl"

	```
	We now have defined a basic topology that exposes an API endpoint to a function or proxy called `enhancer`. However, we haven't written or built `enhancer` function. Let's do that in the next step.


3. Create a directory called `enhancer` in the etl directory. Create a file called handler.py in etl/enhancer directory

	etl/enhancer/handler.py

	```python
		def handler(event, context):
			return {"data": "enhanced-data"}
	```

	Now this ain't doing much is it ? That's all we need for a function though with some business logic.

	Now we may need some libraries (shared etc). Let's go ahead add a pyproject.toml with our dependencies. Since we are using python, the size of the dependencies can increase thus beating the purpose of having a nano function. However, dependencies are inevitable and let's go with it.

4. Now that we added dependencies, we may need to define some additional metadata about the function. This definition is optional if we keep our functions lean with no dependencies. Anyway, let's create a file called `function.json` and add the following to it.

	```json
	{
		"name": "enhancer",
		"description": "enhance wer data",
		"runtime": {
			"lang": "python3.10",
			"package_type": "zip",
			"handler": "handler.handler",
			"layers": ["etl-enhancer"],
			"extensions": []
		},
		"tasks": {
			"build": "zip lambda.zip handler.py",
			"clean": "rm *.zip"
		}
	}
	```
	The above definition describes what our `enhancer` is, how to invoke it etc. Note that we need to specify the layer name for the dependencies. Follow along ...

5. Let's now build the dependencies. At this point, we may want to consider downloading [tc](./installation.md) (it's 5MB executable containing 15K lines of magic written in Rust). We need to login to an AWS env (say dev):

	```sh
	tc login -e dev

	cd etl
	tc build --publish
	```
	The above command builds and publishes the dependencies as a `lambda layer` to a centralized account (CICD). Now if our dependencies are really bloated, `tc build` will split the layers into 40MB (x 3) chunks. If we have nested directories (lib/ vendor/ etc), it will merge it. It will also be able to pull private repos, pull AWS assets when needed.

	To see if the dependency layer actually got published, run `tc build --list`

	```sh
	name                                      | version | created_date
	-------------------------------------------+---------+------------------------------
	etl-enhancer                              | 1       | 2024-01-04T17:24:28.363+0000
	```

	Note that the layer only contains the dependencies we added for etl-enhancer, not the enhancer code itself. That gets packed and deployed separately to our sandbox. _The reason the layer build and code packing steps are decoupled is because the former is heavy and the latter is leaner_.

6. Phew! building dependencies is not straightforward. It has to be built for the right CPU architecture, find shared objects, resolve shared libs, fetch private repositories, autofix incompatible transitive dependencies. That's a lot of complexity to absorb. Incidental complexity you say, eh ?
Anyhow, let's create a sandbox with our "enhanced" code.

	```sh
	tc create -s bob -e dev

	2024-01-15T19:57:03.865 Composing topology...
	2024-01-15T19:57:04.168 Initializing functor: etl@bob.dev/0.0.1
	2024-01-15T19:57:04.431 Creating function etl_enhancer_bob (214 B)
	2024-01-15T19:57:04.431 Creating route /api/etl (OK)
	```

	Voila! Our `enhancer` function is tiny and the bloated dependencies got layered away in the previous step. Dependencies don't change much do they ? Things that move fast ought to be lean.

7. Let's say we modify our code and would like to incrementally update the sandbox.

	We can do `tc diff -s bob -e dev` to see what the diff is between our local edits and the code in our remote lambda function. When satisfied:

	```sh
	cd etl
	tc update -s bob -e dev -c enhancer
	```

8. Well, there are other infrastructure components in a topology and that is something we prefer to isolate from the code. We can scaffold roles and vars json files to an `infrastructure` directory

	```sh
	tc scaffold --create functions
	```

	The above command will create roles and vars files in infrastructure/tc/etl/{vars, roles}/enhancer.json. We can add any additional env vars, secret uris and other function-specific IAM permissions.

	We can incrementally update the vars, roles etc

	```sh
	tc update -s bob -e dev -c roles
	tc update -s bob -e dev -c vars
	tc update -s bob -e dev -c routes
	```

9. Now we may need to create an eventbridge event to trigger our enhancer (Remember, that is a requirement). So let's add that to the topology defintiion.

	```yaml
	name: etl
	routes:
		etl:
		gateway: api-test
		proxy: '{{namespace}}_enhancer_{{sandbox}}'
		kind: http
		timeout: 10
		async: false
		method: POST
		path: "/api/etl"

	events:
		consumes:
			StartETL:
				producer: default
				function: '{{namespace}}_enhancer_{{sandbox}}'

	```

	Now just update the `events` component

	```sh
	tc update -s bob -e dev -c events
	```

10. One of the requirements is to build and use a ML model for the transformer.

	```json
	{
		"name": "transformer",
		"description": "tranform your soul",
		"runtime": {
			"lang": "python3.10",
			"package_type": "zip",
			"handler": "handler.handler",
			"layers": [],
			"extensions": []
		},
		"assets": {
			"MODEL_PATH": "/mnt/assets/etl/transformer/1.0/artifacts",
			"DEPS_PATH": "/mnt/assets/etl/transformer/deps"
		},
		"tasks": {
			"build": "zip lambda.zip handler.py",
			"clean": "rm *.zip"
		}
	}
	```

	Now building model (primarily using pytorch) is no child's play. Yet, `tc build` makes it simple

	```sh
	cd transformer
	tc build --kind artifacts --publish
	```

	If an `assets` key in present in `function.json` file, `tc build --kind deps --publish` publishes it to EFS. The models and deps are available to the function automagically.


11. Now, let's write our `loader` function in Ruby. Can `tc` build it ? Let's see.

	Add a Gemfile, a handler (handler.rb or a module) and function.json in loader directory.

	```json
	{
		"name": "loader",
		"description": "load your jiggle wiggle",
		"runtime": {
			"lang": "ruby3.2",
			"package_type": "zip",
			"handler": "handler.handler",
			"layers": [],
			"extensions": []
		},
		"tasks": {
			"build": "zip lambda.zip handler.rb",
			"clean": "rm *.zip"
		}
	}
	```

	Like we did with python dependencies, we can create a layer and publish it

	```sh
	cd loader
	tc build --publish
	```

	`tc build --list` to see if it got published

	```sh
	name                                      | version | created_date
	-------------------------------------------+---------+------------------------------
	etl-enhancer                              | 1       | 2024-01-04T17:24:28.363+0000
	etl-loader                                | 1       | 2024-01-04T18:24:28.363+0000
	```

12. Let's create the function:

	```
	cd etl
	tc create -s bob -e dev

	2024-01-15T19:57:03.865 Composing topology...
	2024-01-15T19:57:04.168 Initializing functor: etl@bob.dev/0.0.1
	2024-01-15T19:57:04.431 Creating function etl_enhancer_bob (214 B)
	2024-01-15T19:57:04.431 Creating function etl_transformer_bob (10 KiB)
	2024-01-15T19:57:04.431 Creating function etl_loader_bob (629 B)
	2024-01-15T19:57:04.431 Creating route /api/test (OK)
	```

13. Perhaps we can now create a flow of data between `enhancer` and `transformer` functions. We can define the flow using the AWS stepfunction ASL.

	```yaml
	name: etl
	routes:
		etl:
	   	gateway: api-test
		proxy: '{{namespace}}_enhancer_{{sandbox}}'
		kind: http
		timeout: 10
		async: false
		method: POST
		path: "/api/etl"

	events:
		consumes:
			StartETL:
				producer: default
				function: '{{namespace}}_enhancer_{{sandbox}}'
    flow:
		Comment: ETL
		StartAt: enhance
		TimeoutSeconds: 1200
		States:
			enhance:
				Type: Task
				Resource: arn:aws:states:::lambda:invoke
				OutputPath: $.Payload
				InputPath: $
				Parameters:
					FunctionName: '{{namespace}}_enhancer_{{sandbox}}'
					Payload:
						data.$: $
		        Next: transform
			transform:
				Type: Task
				Resource: arn:aws:states:::lambda:invoke
				OutputPath: $.Payload
				InputPath: $
				Parameters:
					FunctionName: '{{namespace}}_transformer_{{sandbox}}'
					Payload:
						data.$: $
		        Next: transform
			load:
				Type: Task
				Resource: arn:aws:states:::lambda:invoke
				OutputPath: $.Payload
				InputPath: $
				Parameters:
					FunctionName: '{{namespace}}_loader_{{sandbox}}'
					Payload:
						data.$: $
		        End: true
    ```

	To update the flow do:

	```sh
	tc update -s bob -e dev -c flow
	```

14. To invoke the stepfunction flow:

	```sh
	tc invoke -s bob -e dev --payload payload.json [--sync]
	```

15. Finally, lets delete our dev sandbox and deploy this to a stable sandbox in upper envs

	```
	tc delete -s bob -e dev
	```

### Release and CI workflow

Well, the above steps work well if we need to interactively build, test and try in our sandbox. Wouldn't it be nice to atomically create a sandbox and attach all the infrastructure components. Oh, while we are it, can we also version the topology ?

	tc deploy --sandbox stable --env qa --service etl --version 0.1.4

How do we bump the versions and _release_ it to a QA env ? `tc` provides a simplified versioning scheme. The following command bumps the _minor_ part of the semver and deploys to a QA sandbox

	tc release --service etl
	;=> 0.2.0

To see a meaningful changelog between releases:

	cd etl
	tc diff --changelog
