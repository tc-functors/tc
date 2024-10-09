# Invoker

<!-- toc -->


## Specifying Payload

To simply invoke a functor

```
tc invoke --sandbox main --env dev
```
By default, tc picks up a `payload.json` file in the current directory. You could optionally specify a payload file

```
tc invoke --sandbox main --env dev --payload payload.json
```

or via stdin
```
cat payload.json | tc invoke --sandbox main --env dev
```

or as a param
```
tc invoke --sandbox main --env dev --payload '{"data": "foo"}'
```

### Invoking Events and Lambdas

By default, `tc` invokes a stepfn. We can also invoke a lambda or trigger an Eventbridge event

```
tc invoke --kind lambda -e dev --payload '{"data"...}'
tc invoke --kind event -e dev --payload '{"data"...}'
```

## Invoking with CSV data

To provide a CSV file to a stepfn whose entrypoint is a distributed map

```
tc invoke -f extraction-runner -e poc --target main -p extraction --map docs.csv
```

If you don't have an id but a list of ids to process (say doc ids):

```
tc invoke -f extraction-multitest -e poc --target STP-2755 -p provider --map doc-ids.txt
```

### Interactive execution

To interactively execute a stepfunction

```
tc invoke -e dev -s sandbox --interactive
functor@sandbox.dev> next
```
