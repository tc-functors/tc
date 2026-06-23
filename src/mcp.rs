use kit as u;
use rmcp::{
    ServiceExt,
    handler::server::wrapper::Parameters,
    schemars,
    tool,
    tool_router,
    transport::stdio,
};
use std::time::Instant;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ComposeRequest {
    #[schemars(description = "Path to topology dir")]
    pub dir: String,
    #[schemars(description = "recurse through directories")]
    pub recursive: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BuildRequest {
    #[schemars(description = "Path to function dir")]
    pub dir: String,
    #[schemars(description = "AWS PROFILE or environment")]
    pub profile: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateRequest {
    #[schemars(description = "Path to topology dir")]
    pub dir: String,
    #[schemars(description = "Whether to recurse through topologies")]
    pub recursive: bool,
    #[schemars(description = "AWS PROFILE or environment")]
    pub profile: String,
    #[schemars(description = "Sandbox name")]
    pub sandbox: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateRequest {
    #[schemars(description = "Path to topology dir")]
    pub dir: String,
    #[schemars(description = "Whether to recurse through topologies")]
    pub recursive: bool,
    #[schemars(description = "AWS PROFILE or environment")]
    pub profile: String,
    #[schemars(description = "Sandbox name")]
    pub sandbox: String,
    #[schemars(
        description = "Entity/Component - functions, events, routes, mutations, channels, states"
    )]
    pub entity: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteRequest {
    #[schemars(description = "Path to topology dir")]
    pub dir: String,
    #[schemars(description = "Whether to recurse through topologies")]
    pub recursive: bool,
    #[schemars(description = "AWS PROFILE or environment")]
    pub profile: String,
    #[schemars(description = "Sandbox name")]
    pub sandbox: String,
    #[schemars(
        description = "Entity/Component - functions, events, routes, mutations, channels, states"
    )]
    pub entity: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InvokeRequest {
    #[schemars(description = "Path to topology or function dir")]
    pub dir: String,
    #[schemars(description = "AWS PROFILE or environment")]
    pub profile: String,
    #[schemars(description = "Sandbox name")]
    pub sandbox: String,
    #[schemars(
        description = "Entity/Component - functions, events, routes, mutations, channels, states"
    )]
    pub entity: Option<String>,
    #[schemars(description = "Payload in JSON")]
    pub payload: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TestRequest {
    #[schemars(description = "Path to topology or function dir")]
    pub dir: String,
    #[schemars(description = "Whether to recurse through topologies")]
    pub recursive: bool,
    #[schemars(description = "AWS PROFILE or environment")]
    pub profile: String,
    #[schemars(description = "Sandbox name")]
    pub sandbox: String,
    #[schemars(description = "Unit test name")]
    pub unit: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ResolveRequest {
    #[schemars(description = "Path to topology or function dir")]
    pub dir: String,
    #[schemars(description = "Whether to recurse through topologies")]
    pub recursive: bool,
    #[schemars(description = "AWS PROFILE or environment")]
    pub profile: String,
    #[schemars(description = "Sandbox name")]
    pub sandbox: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ChangelogRequest {
    #[schemars(description = "Path to topology or function dir")]
    pub dir: String,
    #[schemars(description = "diff versions of form  a.bc..x.y.z")]
    pub between: Option<String>,
    #[schemars(description = "Search for arbitrary string in changelogs")]
    pub search: Option<String>,
    #[schemars(description = "Show versions")]
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub struct Tc;

#[tool_router(server_handler)]
impl Tc {
    #[tool(description = "Compose a topology")]
    fn compose(
        &self,
        Parameters(ComposeRequest { dir, recursive }): Parameters<ComposeRequest>,
    ) -> String {
        let topology = composer::compose(&dir, recursive);
        topology.to_str()
    }

    #[tool(description = "Build function")]
    async fn build(
        &self,
        Parameters(BuildRequest { dir, profile }): Parameters<BuildRequest>,
    ) -> String {
        let maybe_fn = composer::current_function(&dir);
        match maybe_fn {
            Some(f) => {
                let auth = tc::init_centralized_auth(Some(profile)).await;
                let _builds = builder::build(&auth, &f, Some(f.name.clone()), None, false).await;
                "success".to_string()
            }
            None => String::from("No function found"),
        }
    }

    #[tool(description = "Create a topology in a sandbox")]
    async fn create(
        &self,
        Parameters(CreateRequest {
            dir,
            profile,
            sandbox,
            recursive,
        }): Parameters<CreateRequest>,
    ) -> String {
        let start = Instant::now();
        let auth = tc::init(Some(profile), None).await;
        let ct = composer::compose(&dir, recursive);
        let rt = resolver::resolve(&auth, &sandbox, &ct, false, false).await;
        deployer::guard::prevent_stable_updates(&auth, &sandbox, &rt).await;
        composer::count_of(&ct);

        tc::create_topology(&auth, &rt, false).await;
        let duration = start.elapsed();
        format!("Time elapsed: {:#}", u::time_format(duration))
    }

    #[tool(description = "Update a topology in a sandbox")]
    async fn update(
        &self,
        Parameters(UpdateRequest {
            dir,
            profile,
            sandbox,
            recursive,
            entity,
        }): Parameters<UpdateRequest>,
    ) -> String {
        let auth = tc::init(Some(profile), None).await;
        let topology = composer::compose(&dir, recursive);
        let rt = resolver::render(&auth, &sandbox, &topology).await;
        deployer::guard::prevent_stable_updates(&auth, &sandbox, &rt).await;
        tc::update_aux(&auth, &sandbox, &topology, entity).await;
        let msg = composer::count_of(&topology);
        msg
    }

    #[tool(description = "Delete a topology in given sandbox")]
    async fn delete(
        &self,
        Parameters(DeleteRequest {
            dir,
            profile,
            sandbox,
            recursive,
            entity,
        }): Parameters<DeleteRequest>,
    ) -> String {
        let auth = tc::init(Some(profile), None).await;
        let topology = composer::compose(&dir, recursive);
        let rt = resolver::render(&auth, &sandbox, &topology).await;
        deployer::guard::prevent_stable_updates(&auth, &sandbox, &rt).await;

        let root = resolver::try_resolve(&auth, &sandbox, &topology, &entity, false, false).await;

        deployer::try_delete(&auth, &root, &entity).await;

        for (_, node) in root.nodes {
            deployer::try_delete(&auth, &node, &entity).await;
        }
        composer::count_of(&topology)
    }

    #[tool(description = "Invoke entity or topology in given sandbox")]
    async fn invoke(
        &self,
        Parameters(InvokeRequest {
            dir,
            profile,
            sandbox,
            entity,
            payload,
        }): Parameters<InvokeRequest>,
    ) -> String {
        let auth = tc::init(Some(profile), None).await;
        let topology = composer::compose(&dir, true);
        let resolved = resolver::render(&auth, &sandbox, &topology).await;
        invoker::invoke(&auth, entity, &resolved, Some(payload), true).await;
        //FIXME: get output of invoke
        String::from("")
    }

    #[tool(description = "Test entity or topology in given sandbox")]
    async fn test(
        &self,
        Parameters(TestRequest {
            dir,
            recursive,
            profile,
            sandbox,
            unit,
        }): Parameters<TestRequest>,
    ) -> String {
        let auth = tc::init(Some(profile), None).await;
        if composer::is_topology_dir(&dir) {
            let topology = composer::compose(&dir, recursive);
            let resolved = resolver::render(&auth, &sandbox, &topology).await;
            tester::test_topology(&auth, &resolved, unit).await;
        } else {
            if let Some(f) = composer::current_function(&dir) {
                tester::test_function(&auth, &sandbox, &f, unit).await;
            }
        }
        // FIXME: capture output
        String::from("")
    }

    #[tool(description = "Resovle a topology")]
    async fn resolve(
        &self,
        Parameters(ResolveRequest {
            dir,
            recursive,
            profile,
            sandbox,
        }): Parameters<ResolveRequest>,
    ) -> String {
        let auth = tc::init(Some(profile), None).await;
        let ct = composer::compose(&dir, recursive);
        let rt = resolver::resolve(&auth, &sandbox, &ct, false, false).await;
        rt.to_str()
    }

    #[tool(description = "Generate changelog for a topology")]
    async fn changelog(
        &self,
        Parameters(ChangelogRequest {
            dir,
            between,
            search,
            verbose,
        }): Parameters<ChangelogRequest>,
    ) -> String {
        let topology = composer::compose(&dir, false);
        let namespace = topology.namespace;
        match search {
            Some(s) => {
                let is_root = composer::is_root_dir(&dir);
                if is_root {
                    let namespaces = composer::root_namespaces(&dir);
                    for (_, namespace) in namespaces {
                        let version = tagger::find_version_history(&namespace, &s).await;
                        if let Some(v) = version {
                            println!("{},{},{}", &s, &namespace, &v);
                        }
                    }
                }
            }
            None => tagger::changelog(&namespace, between, verbose),
        }
        //FIXME: capture output
        String::from("")
    }
}

pub async fn serve() {
    let service = Tc.serve(stdio()).await.expect("Failed to start MCP server");
    let _ = service.waiting().await;
}
