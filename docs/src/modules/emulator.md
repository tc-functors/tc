# Emulator

<!-- toc -->


### Lambdas

To emulate the Lambda Runtime environment. The following command spins up a docker container with the defined layers in function.json, sets up the paths, environment variables, AWS access, local code and runtime parameters (mem, handlers etc)

```sh
cd <function-dir>
tc emulate
```

To run in foreground

```
tc emulate
```

You can now invoke a payload locally with this emulator

```
tc invoke --local [--payload <payload.json | json-str>]
```

### Stepfunctions


`tc` also provides a stepfunction emulator. In your top-level topology directory, do:

```
tc emulate
```

This spins up a container and runs the emulator on http://localhost:8083


Details to follow on creating and executing [wip]
