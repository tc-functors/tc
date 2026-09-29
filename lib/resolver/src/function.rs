use super::Context;
use compiler::{
    TopologyKind,
    BuildKind,
    spec::{
        InfraSpec,
        NetworkSpec,
    },
};
use composer::{
    Function,
    Runtime,
    Topology,
    function::runtime::{
        Network,
    },
};
use futures::stream::{
    self,
    StreamExt,
};
use kit as u;
use kit::{
    AsyncMemo,
    *,
};
use provider::{
    Auth,
    aws,
};
use std::collections::HashMap;

pub async fn lookup_urls(auth: &Auth, fqn: &str) -> HashMap<String, String> {
    let client = aws::gateway::make_client(auth).await;
    let api = aws::gateway::find_api_id(&client, fqn).await;
    tracing::debug!("Looking up api-id for {}", &fqn);
    match api {
        Some(a) => {
            let endpoint = auth.api_endpoint(&a, "$default");
            let mut h: HashMap<String, String> = HashMap::new();
            h.insert(s!("API_GATEWAY_URL"), endpoint);
            h
        }
        _ => HashMap::new(),
    }
}

static URL_CACHE: AsyncMemo<(String, String), HashMap<String, String>> = AsyncMemo::new();

async fn cached_lookup_urls(auth: &Auth, fqn: &str) -> HashMap<String, String> {
    URL_CACHE
        .get_or_init((auth.name.clone(), fqn.to_string()), || async {
            tracing::debug!("Looking up api-id for {} (cache miss)", fqn);
            lookup_urls(auth, fqn).await
        })
        .await
}

fn render_config(s: &str, config: &HashMap<String, String>) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    for (k, v) in config {
        table.insert(&k, &v);
    }
    u::stencil(&s, table)
}

async fn resolve_vars(
    auth: &Auth,
    environment: HashMap<String, String>,
    fqn: &str,
    resolve_urls: bool,
) -> HashMap<String, String> {
    tracing::debug!("Resolving env vars");
    let client = aws::ssm::make_client(auth).await;

    let needs_urls = resolve_urls && environment.values().any(|v| v.starts_with("{{"));
    let config = if needs_urls {
        cached_lookup_urls(auth, fqn).await
    } else {
        HashMap::new()
    };

    let mut h: HashMap<String, String> = HashMap::new();
    for (k, v) in environment.iter() {
        if v.starts_with("ssm:/") {
            let key = kit::split_last(v, ":");
            let val = aws::ssm::get(client.clone(), &key).await.unwrap();
            h.insert(s!(k), val);
        } else if v.starts_with("{{") {
            if resolve_urls {
                let val = render_config(v, &config);
                h.insert(s!(k), val);
            }
        } else {
            h.insert(s!(k), s!(v));
        }
    }
    h
}

static LAYER_AUTH: AsyncMemo<(Option<String>, Option<String>), Auth> = AsyncMemo::new();

async fn make_layer_auth(ctx: &Context) -> Auth {
    let Context { auth, config,  .. } = ctx;
    let profile = config.aws.lambda.layers_profile.clone();
    let role = config.role_to_assume(profile.clone());
    let key = (profile.clone(), role.clone());
    LAYER_AUTH
        .get_or_init(key, || async {
            tracing::debug!("Assuming layer-auth profile (cache miss)");
            auth.assume(profile, role, Some(auth.region.clone())).await
        })
        .await
}

static LAYER_VERSION_CACHE: AsyncMemo<(Option<String>, Option<String>, String), String> =
    AsyncMemo::new();

async fn resolve_layer(ctx: &Context, layer_name: &str) -> String {
    let Context { config, .. } = ctx;
    let profile = config.aws.lambda.layers_profile.clone();
    let role = config.role_to_assume(profile.clone());
    let key = (profile, role, layer_name.to_string());
    LAYER_VERSION_CACHE
        .get_or_init(key, || async {
            tracing::debug!("Resolving layer {} (cache miss)", layer_name);
            let auth = make_layer_auth(ctx).await;
            let client = aws::layer::make_client(&auth).await;
            aws::layer::find_version(client, layer_name).await.unwrap()
        })
        .await
}

// arn
fn as_layer_arn(auth: &Auth, name: &str) -> String {
    format!(
        "arn:aws:lambda:{}:{}:layer:{}",
        auth.region, auth.account, name
    )
}

//
fn augment_vars(ctx: &Context, lang: &str) -> HashMap<String, String> {
    tracing::debug!("Augmenting vars {}", lang);
    let mut hmap: HashMap<String, String> = HashMap::new();
    let profile = &ctx.auth.name;
    let sandbox = &ctx.sandbox;
    match lang {
        "ruby3.2" => {
            if sandbox != "stable" {
                hmap.insert(
                    String::from("HONEYBADGER_ENV"),
                    format!("{}-{}", profile, sandbox),
                );
            } else {
                hmap.insert(String::from("HONEYBADGER_ENV"), s!(profile));
            }
        }
        _ => {
            if sandbox != "stable" {
                hmap.insert(
                    String::from("HONEYBADGER_ENVIRONMENT"),
                    format!("{}-{}", profile, sandbox),
                );
            } else {
                hmap.insert(String::from("HONEYBADGER_ENVIRONMENT"), s!(profile));
            }
        }
    }
    hmap
}

async fn resolve_environment(
    ctx: &Context,
    lang: &str,
    default_vars: &HashMap<String, String>,
    sandbox_vars: Option<HashMap<String, String>>,
    fqn: &str,
    resolve_urls: bool,
) -> HashMap<String, String> {
    let Context { auth, .. } = ctx;
    let mut default_vars = default_vars.clone();

    let augmented_vars = augment_vars(ctx, lang);
    default_vars.extend(augmented_vars);

    let combined = match sandbox_vars {
        Some(v) => {
            default_vars.extend(v);
            default_vars
        }
        None => default_vars,
    };

    resolve_vars(auth, combined.clone(), fqn, resolve_urls).await
}

async fn resolve_network(
    ctx: &Context,
    enable_network: bool,
    ns: Option<NetworkSpec>,
    network: Option<Network>,
) -> Option<Network> {
    let Context { auth, config, .. } = ctx;

    match network {
        Some(net) => Some(net),
        None => {
            if enable_network {
                if let Some(n) = ns {
                    let net = Network {
                        subnets: n.subnets.clone(),
                        security_groups: n.security_groups.clone(),
                    };
                    Some(net)
                } else {
                    None
                }
            } else {
                let cfg = &config.aws.efs.network;
                let cfg_net = cfg.get(&auth.name);
                match cfg_net {
                    Some(netc) => {
                        let net = Network {
                            subnets: netc.subnets.clone(),
                            security_groups: netc.security_groups.clone(),
                        };
                        Some(net)
                    }
                    None => None,
                }
            }
        }
    }
}

async fn get_extension_arn(auth: &Auth, path: &str) -> String {
    let client = aws::ssm::make_client(auth).await;
    let key = kit::split_last(path, ":");
    aws::ssm::get(client.clone(), &key).await.unwrap()
}

async fn resolve_layers(ctx: &Context, layers: Vec<String>) -> Vec<String> {
    let Context { auth, sandbox, .. } = ctx;
    let mut xs: Vec<String> = vec![];

    for layer in layers {
        if layer.starts_with("ssm:") {
            let arn = get_extension_arn(auth, &layer).await;
            xs.push(arn)
        } else if layer.contains(":") {
            xs.push(as_layer_arn(&auth, &layer))
        } else if *sandbox != "stable" {
            let name = match std::env::var("TC_USE_STABLE_LAYERS") {
                Ok(_) => layer,
                Err(_) => format!("{}-dev", &layer),
            };
            xs.push(resolve_layer(ctx, &name).await);
        } else {
            xs.push(resolve_layer(ctx, &layer).await)
        }
    }
    xs
}

fn augment_infra_spec(default: &InfraSpec, s: &InfraSpec) -> InfraSpec {
    InfraSpec {
        memory_size: match s.memory_size {
            Some(p) => {
                if p != 128 {
                    Some(p)
                } else {
                    default.memory_size
                }
            }
            None => default.memory_size,
        },
        timeout: match s.timeout {
            Some(p) => {
                if p != 300 {
                    Some(p)
                } else {
                    default.timeout
                }
            }
            None => default.timeout,
        },
        environment: match s.environment.clone() {
            Some(p) => {
                let mut def = default.environment.clone().unwrap();
                def.extend(p);
                Some(def)
            }
            None => default.environment.clone(),
        },
        image_uri: None,
        network: match s.network.clone() {
            Some(p) => Some(p),
            None => default.network.clone(),
        },
        filesystem: match &s.filesystem {
            Some(p) => Some(p.clone()),
            None => default.filesystem.clone()
        },
        provisioned_concurrency: match s.provisioned_concurrency {
            Some(p) => Some(p),
            None => default.provisioned_concurrency,
        },
        reserved_concurrency: match s.reserved_concurrency {
            Some(p) => Some(p),
            None => default.reserved_concurrency,
        },
        tags: None,
    }
}

fn get_infra_spec(
    infra_spec: &HashMap<String, InfraSpec>,
    profile: &str,
    sandbox: &str,
) -> InfraSpec {
    let profile_specific = infra_spec.get(profile);
    let sandbox_specific = infra_spec.get(sandbox);
    let default = infra_spec.get("default").unwrap();

    if let Some(s) = profile_specific {
        return augment_infra_spec(&default, s);
    }
    if let Some(s) = sandbox_specific {
        return augment_infra_spec(&default, s);
    }

    default.clone()
}

async fn resolve_runtime(
    ctx: &Context,
    function: &Function,
    fqn: &str,
    resolve_urls: bool,
    force: bool
) -> Runtime {
    let Context { auth, sandbox, .. } = ctx;

    let runtime = function.runtime.clone();
    let Runtime {
        layers,
        network,
        infra_spec,
        enable_network,
        ..
    } = &function.runtime;
    let mut r: Runtime = function.runtime.clone();



    let uri = if force {
        match &function.build.kind {
            BuildKind::Inline => &format!("{}/lambda.zip", &function.dir),
            _ => &function.runtime.uri
        }
    } else {
        &function.runtime.uri
    };

    r.uri = uri.to_string();

    let actual_infra = get_infra_spec(&infra_spec, &auth.name, sandbox);
    let InfraSpec {
        memory_size,
        timeout,
        environment,
        ..
    } = actual_infra;

    r.memory_size = memory_size;
    r.timeout = timeout;
    r.environment = resolve_environment(
        ctx,
        &runtime.lang.to_str(),
        &runtime.environment,
        environment,
        fqn,
        resolve_urls,
    )
    .await;
    if !layers.is_empty() {
        r.layers = resolve_layers(ctx, layers.clone()).await;
    }
    if *enable_network {
        r.network =
            resolve_network(ctx, r.enable_network, actual_infra.network, network.clone()).await;
    }

    let fs = match actual_infra.filesystem.as_ref() {
        Some(mfs) => mfs.get(&auth.region).clone(),
        None => None
    };


    r.fs = fs.cloned();
    r.infra_spec = HashMap::new();
    r
}

pub struct Root {
    pub namespace: String,
    pub fqn: String,
    pub kind: TopologyKind,
    pub version: String,
}

pub(crate) fn classify_modified<F: Clone>(
    fallback: &HashMap<String, F>,
    namespace: &str,
    target_version: &str,
    version: &str,
    diff_result: Result<HashMap<String, F>, differ::DiffError>,
) -> HashMap<String, F> {
    match diff_result {
        Ok(fns) => {
            if !fns.is_empty() {
                println!(
                    "Diff {} {}..{} ({})",
                    namespace,
                    target_version,
                    version,
                    fns.len()
                );
            }
            fns
        }
        Err(differ::DiffError::TagUnresolvable { tag }) => {
            tracing::warn!(
                "deployed version {} for {} has no resolvable git tag ({}); \
                 cannot compute incremental diff — redeploying all {} function(s)",
                target_version,
                namespace,
                tag,
                fallback.len()
            );
            fallback.clone()
        }
    }
}

pub async fn find_modified(
    auth: &Auth,
    root: &Root,
    topology: &Topology,
) -> HashMap<String, Function> {
    let Root {
        namespace,
        fqn,
        kind,
        version,
    } = root;

    let maybe_version = snapshotter::find_version(auth, fqn, kind).await;

    let target_version = match maybe_version {
        Some(v) => v,
        None => return topology.functions.clone(),
    };

    // Pass the *root* namespace to diff_fns so git tag construction
    // uses the namespace where tags actually live. `topology` here may
    // be a nested node whose own namespace doesn't match any tag.
    let diff_result = differ::diff_fns(topology, namespace, &target_version, version);
    classify_modified(
        &topology.functions,
        namespace,
        &target_version,
        version,
        diff_result,
    )
}

pub async fn resolve(
    ctx: &Context,
    root: &Root,
    topology: &Topology,
    force: bool,
) -> HashMap<String, Function> {
    let fns: HashMap<String, Function> = match std::env::var("TC_FORCE_DEPLOY") {
        Ok(_) => topology.functions.clone(),
        Err(_) => {
            if force {
                topology.functions.clone()
            } else {
                find_modified(&ctx.auth, root, topology).await
            }
        }
    };

    let resolve_urls = topology.routes.len() > 0;
    let concurrency = crate::resolve_concurrency();
    let fqn = root.fqn.clone();

    stream::iter(fns.into_iter())
        .map(|(name, f)| {
            let fqn = fqn.clone();
            async move {
                tracing::debug!("Resolving function {}", &name);
                let mut fu = f.clone();
                fu.runtime = resolve_runtime(ctx, &f, &fqn, resolve_urls, force).await;
                (name, fu)
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await
}

pub async fn resolve_given(
    ctx: &Context,
    root: &Root,
    topology: &Topology,
    component: &str,
) -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    let fns = &topology.functions;
    if let Some(f) = fns.get(component) {
        let mut fu: Function = f.clone();

        let resolve_urls = topology.routes.len() > 0;

        fu.runtime = resolve_runtime(ctx, &f, &root.fqn, resolve_urls, false).await;
        functions.insert(component.to_string(), fu.clone());
        functions
    } else {
        resolve(ctx, root, topology, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// `Ok(...)` is returned verbatim — the resolver trusts the differ
    /// when it has computed an answer.
    #[test]
    fn classify_modified_passes_through_ok() {
        let fallback: HashMap<String, String> = HashMap::from([
            ("a".to_string(), "FALLBACK_A".to_string()),
            ("b".to_string(), "FALLBACK_B".to_string()),
        ]);
        let computed: HashMap<String, String> =
            HashMap::from([("a".to_string(), "DIFFED_A".to_string())]);

        let out = classify_modified(&fallback, "ns", "0.0.39", "0.1.2", Ok(computed.clone()));

        assert_eq!(out, computed, "Ok payload must pass through unchanged");
    }

    /// `Ok(empty)` is returned as empty — meaningful "nothing changed"
    /// signal, not conflated with the error case.
    #[test]
    fn classify_modified_returns_empty_for_ok_empty() {
        let fallback: HashMap<String, String> =
            HashMap::from([("a".to_string(), "FALLBACK_A".to_string())]);
        let out = classify_modified(
            &fallback,
            "ns",
            "0.0.39",
            "0.1.2",
            Ok(HashMap::<String, String>::new()),
        );
        assert!(
            out.is_empty(),
            "Ok(empty) must NOT be conflated with the fallback path"
        );
    }

    /// THE regression test for stale-deployed-version silent no-op:
    /// when the differ reports `TagUnresolvable`, we must redeploy
    /// every function in the topology (the `fallback`), not return
    /// nothing.
    #[test]
    fn classify_modified_falls_back_on_tag_unresolvable() {
        let fallback: HashMap<String, String> = HashMap::from([
            ("a".to_string(), "FALLBACK_A".to_string()),
            ("b".to_string(), "FALLBACK_B".to_string()),
        ]);
        let err = differ::DiffError::TagUnresolvable {
            tag: "ns-0.0.39".to_string(),
        };

        let out = classify_modified(&fallback, "ns", "0.0.39", "0.1.2", Err(err));

        assert_eq!(
            out, fallback,
            "TagUnresolvable must trigger a full-topology redeploy, \
             not return an empty map (which would be a silent no-op)"
        );
    }

    /// The fallback path with an empty topology returns empty — a
    /// genuine no-op rather than a panic. Edge case but worth pinning.
    #[test]
    fn classify_modified_falls_back_to_empty_when_topology_empty() {
        let fallback: HashMap<String, String> = HashMap::new();
        let err = differ::DiffError::TagUnresolvable {
            tag: "ns-0.0.39".to_string(),
        };
        let out = classify_modified(&fallback, "ns", "0.0.39", "0.1.2", Err(err));
        assert!(out.is_empty());
    }
}
