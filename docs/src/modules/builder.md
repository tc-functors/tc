# Builder

<!-- toc -->

`tc` has a sophisticated `builder` that can build different kinds of artifacts (Dependencies, Extensions, Containers)

Design principles:
- A core design in `tc` is to build dependencies as Lambda layers and pack(zip) only the relevant code. Thus `building` is decoupled from `packing` to enable faster iteration (things that move fast ought to be lean).
- The function specification capture enough metadata on how and what to build

Usage:

```
tc build [--kind <deps|extension|model>] --env <env>

tc build OPTIONS
Options:
      --trace
      --kind <KIND>
      --pack
  -n, --name <NAME or PATH>
```

## Building Dependencies

To build deps manually for testing (Typically CI builds the layers, however we can build it alacarte)

```sh
tc build [--kind deps]
```
`tc build` generates a deps.zip file containing all the deps. It automatically detects the language-type and builds using the AWS lambda Runtime image (AL2).

```admonish info
By default the layer-name is the basename of the directory (typically the function-name). It takes any name. If the URI is a layer, it automatically splits the deps into multipart zip files (of 40MB each)
````

```admonish info
For EFS target, tc infers the target path to upload the asset to (deps.zip or model.zip) using the asset paths defined in function.json
```

## Building Extensions

Lambda extensions are like sidecars that intercept the input/output payload events and can do arbitrary processing on them.

```
tc build --kind extension
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
