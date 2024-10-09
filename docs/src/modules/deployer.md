### Creating a Sandbox

```sh
cd services/extraction
tc create [--sandbox SANDBOX] [-e ENV]
```


### Incremental updates

While developing, we often need to incrementally deploy certain components without recreating the entire topology. `tc` provides an update command that updates given component(s).

To update the code for a function (say page-mapper) in the current directory

```sh
tc update --sandbox test -e dev-af -c page-mapper
```

To update the IAM roles and policies

```sh
tc update --sandbox test -e dev-af -c roles
```

To update the eventbridge event rules:

```sh
tc update --sandbox test -e dev-af -c events
```

To update the environment variables or runtime parameters. Usually these are defined in infrastucture/tc/<topology>/vars dir

```sh
tc update --sandbox test -e dev-af -c vars
```

To build and update layers

```sh
tc update --sandbox test -e dev-af -c layers
```

To update the Statemachine flow

```sh
tc update --sandbox test -e dev-af -c flow

```

To update tags across stepfns, lambdas, roles, policies, eventbridge rules etc

```sh
tc update --sandbox test -e dev-af -c tags
```

To update logging and tracing config

```sh
tc update --sandbox test -e dev-af -c logs
```

```admonish info
Note that update works on unfrozen sandboxes. Most stable sandboxes are immutable and thus update is disabled for those. To mutate, unfreeze it.
