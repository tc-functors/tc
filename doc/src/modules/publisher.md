# Publisher

<!-- toc -->

`tc build --publish` is an alias to build and publish in one incantation.

`tc publish` provides lot more publishing options and manages the lifecycle of assets (layers, slabs etc).

```
Usage: tc publish [OPTIONS]

Options:
  -e, --env <ENV>
      --kind <KIND>
      --name <NAME>
      --list
      --trace
      --promote
      --demote
      --version <VERSION>
      --task <TASK>
```
```admonish info
For EFS target, tc infers the target path to upload the asset to (deps.zip or model.zip) using the asset paths defined in function.json
```

By default, the file to publish is deps.zip generated via the build command. However, there may be cases where you just need to publish an arbitrary asset to efs. In such cases, do the following:

```sh
tc --publish --kind artifacts
```

## Updating Layers

Lambda extensions are like sidecars that intercept the input/output payload events and can do arbitrary processing on them.

```
tc publish --kind extension
```

## Add or update layers in the function

Add the layer-name to function.json:layers

```json
"layers": ["<layer-name>"]
```

Update the layers

```
cd ..
tc update --sandbox <sandbox> --env <env> -c layers
```

## Updating slabs (Artifacts)

Lets say we uploaded the built deps to EFS

```
tc publish --kind artifacts
```

We can use that in the lambda by adding the following to function json.

```json
"assets": {
 "MODEL_PATH": "/mnt/assets/classifier/page-classifier/0.1.2/artifacts"
 "DEPS_PATH": "/mnt/assets/classifier/page-classifier/0.1.2/deps"
}
```

`MODEL_PATH` key in the `assets` map gets created as an Environment variable in the lamdba function.
`DEPS_PATH` is added to the lang-specific classpath or libpath

`tc` will automatically mount EFS and make these available to the lambda function with zero configuration.


### Promoting Layers

By default, layers are created with a `-dev` suffix to prevent mutating stable layers. A `stable` layer is a promoted dev layer. It is not rebuilt but an existing dev layer is promoted as stable

To promote a layer:

```sh
tc publish --promote --name <layer-name-without-dev-suffix> [--version x]
```


We can also create a dev layer from a stable layer

```
tc publish --demote --name <layer-name-without-dev-suffix> [--version x]
```


### Listing layers and assets

To list layers

```sh

tc publish --list --target layer --kind deps
```

To list EFS assets


```sh
tc publish --list --target efs --kind deps|assets
```


### Running arbitrary task to manage EFS

To run arbitrary task, for example:

```
tc publish --task "cp -r /mnt/assets/configurable_platform/document_fields/stable/dev/schemas/v1/form_ssa_1099.py /mnt/assets/configurable_platform/document_fields/mtm/prod/schemas/v1/form_ssa_1099.py"
```
