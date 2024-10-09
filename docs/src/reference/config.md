# Config




### Environment Variables


`tc` uses special environment variables as feature bits and config overrides. The following is the list of TC environment variables:

**TC_DIR**

We don't have to always be in the topology or function directory to run a contextual tc command. TC_DIR env var sets the directory context

```
TC_DIR=/path/to/services/fubar tc create -s sandbox -e env
```


**TC_USE_STABLE_LAYERS**

At times we may need to use stable layers in non-stable sandboxes. This env variable allows us to use stable layers

```
TC_USE_STABLE_LAYERS=1 tc create -s sandbox -e env
```

**TC_USE_SHARED_DEPS**

This feature flag uses common deps (in EFS) instead of function-specific deps.

```
TC_USE_SHARED_DEPS=1 tc create -s sandbox -e env
```

**TC_FORCE_BUILD**

Tries various fallback strategies to build layers. One of the strategies is to build locally instead of a docker container. Another fallback is to use a specific version of python even if the transitive dependencies need specific version of Ruby or Python

```
TC_FORCE_BUILD=1 tc build --trace
```

**TC_FORCE_DEPLOY**

To create or update stable sandboxes (which are prohibited by default), use this var to override.

```
TC_FORCE_DEPLOY=1 tc create -s sandbox -e env
```

**TC_UPDATE_METADATA**

To update `deploy metadata` to a dynamodb table (the only stateful stuff in TC) for stable sandboxes

```
TC_UPDATE_METADATA=1 tc create -s staging -e env
```

**TC_ECS_CLUSTER**

Use this to override the ECS Cluster name

```
TC_ECS_CLUSTER=my-cluster tc create -s sandbox -e env
```

**TC_USE_DEV_EFS**

Experimental EFS with deduped deps and models

```
TC_USE_DEV_EFS=1 tc create ...

```

**TC_SANDBOX**

Set this to have a fixed sandbox name for all your sandboxes

```
TC_SANDBOX=my-branch tc create -e env
```

**TC_SLACK_URL**

Used to notify releases, deploys etc

```
TC_SLACK_URL=slack-webhook-url tc create -e env...

```
