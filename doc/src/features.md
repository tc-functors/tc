## Features

![Features](/images/features.png)

`tc` provides 3 useful features when creating functors

1. Sandboxing
2. Modular components
3. Dynamic sidecars

To deploy all the functions in a service atomically, run the following in your service directory.

```sh
cd services/extraction
tc build
tc create [--env <env>] [--sandbox <name> | [--component routes|states|events|functions] [--manifest <manifest-file>]

```

#### Sandboxing

Sandboxing is the ability to run a topology in a protected and isolated environment. The components or modules of the topology viz. routes, events, states and functions are namespaced to run within the specified sandbox.

The following command deploys a topology with the given sandbox name.

```sh
tc create --env prod --sandbox mysandbox
```


```admonish info
Sandboxes have isolated IAM permissions. The functions(lambdas), events and routes are scoped by the sandbox name. Sandboxing is possible because of modeling the service as a graph of functions, events, routes and states.
If no sandbox name is given in the deploy command, it picks up the Jira ticket name in the branch (STP-xxxx, MLE-xxxx, AI-xxxx so forth)
```


## Modular components

A topology is composed of nano functions, states, events and routes. These modules can be deployed independently, when needed.


To deploy specific components:

```sh
tc create --component layers
tc create --component functions
tc create --component events
tc create --component states
tc create --component routes
tc create --component logs
tc create --component vars
tc create --component secrets
```

```admonish info
If no component is given, it provisions all the components. tc determines the order of deployment based on the graph. This datastructure is also helpful for clean teardowns and rollbacks.

```
### Dynamic sidecars

To enable dynamic log aggregation, just deploy the `logf` component

```sh
tc create --component logf --sandbox mysandbox
```

```admonish info title="Inbuilt Functors"
logs component provides a inbuilt functor called `logf`. This is a tiny piece of configurable python code that gets dyamically scaffolded, zipped and deployed as a lambda. These tiny functors are packaged as part of the `tc` binary and are generic enough.
```


To disable log aggregation

```sh
tc delete --component logs --sandbox mysandbox
```
