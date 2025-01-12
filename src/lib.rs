use aws::Env;
use colored::Colorize;
use compiler::Topology;
use resolver::Plan;
use kit as u;
use std::panic;
use std::time::Instant;

pub struct BuildOpts {
    pub pack: bool,
    pub no_docker: bool,
    pub trace: bool,
    pub clean: bool,
    pub delete: bool,
    pub parallel: bool,
    pub merge: bool,
    pub dirty: bool,
}

pub async fn build(kind: Option<String>, name: Option<String>, dir: &str, opts: BuildOpts) {
    let kind = builder::determine_kind(kind);
    let name = u::maybe_string(name, u::basedir(&u::pwd()));

    let BuildOpts {
        pack,
        no_docker,
        trace,
        clean,
        dirty,
        merge,
        ..
    } = opts;

    if pack {
        builder::pack_all(dir);
    } else if clean {
        builder::clean(dir);
    } else if merge {

       if &kind == "code" {
            let dirs = u::list_dir(dir);
            builder::merge_dirs(dirs)

        } else {
            let layers = compiler::find_layers();
            let mergeable_layers = builder::mergeable_layers(layers);
            builder::merge(&name, mergeable_layers);
        }

    } else {
        builder::build(&dir, &name, &kind, no_docker, trace, dirty).await;
    }
}

pub struct PublishOpts {
    pub trace: bool,
    pub promote: bool,
    pub demote: bool,
    pub version: Option<String>,
}

pub async fn publish(
    env: Env,
    kind: Option<String>,
    name: Option<String>,
    dir: &str,
    opts: PublishOpts,
) {
    let PublishOpts {
        promote,
        demote,
        version,
        ..
    } = opts;

    if promote {
        let lang = &compiler::guess_lang(&dir);
        let bname = u::maybe_string(name, u::basedir(&u::pwd()));
        publisher::promote(&env, &bname, &lang, version).await;
    } else if demote {
        let lang = "python3.10";
        publisher::demote(&env, name, &lang).await;
    } else {
        let lang = &compiler::guess_lang(&dir);
        let target = compiler::determine_target(dir);
        let name = u::maybe_string(name, u::basedir(dir));
        let kind = u::maybe_string(kind, "deps");
        let builds = builder::just_build_out(&dir, &name, &lang, &target);
        match kind.as_ref() {
            "deps" | "extension" => {
                for build in builds {
                    publisher::publish_deps(
                        &env,
                        &build.dir,
                        &build.zipfile,
                        &build.lang,
                        &build.name,
                        &build.target,
                    )
                    .await
                }
            }
            _ => (),
        }
    }
}

pub async fn list_published_assets(env: Env) {
    publisher::list(&env).await
}

pub async fn test() {
    let dir = u::pwd();
    let spec = compiler::compile(&dir, false);
    for (path, function) in spec.functions {
        tester::test(&path, function).await;
    }
}

pub struct CompileOpts {
    pub versions: bool,
    pub recursive: bool,
    pub component: Option<String>,
    pub format: Option<String>,
}

pub async fn compile(opts: CompileOpts) -> String {
    let CompileOpts {
        recursive,
        component,
        format,
        ..
    } = opts;

    let dir = u::pwd();
    let format = u::maybe_string(format, "json");

    match component {
        Some(c) => compiler::show_component(&c, &format),
        None => {
            let topology = compiler::compile(&dir, recursive);
            u::pretty_json(topology)
        }
    }
}

pub async fn resolve(
    env: Env,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
) -> String {
    let resolve = resolver::should_resolve(component.clone());
    let topology = compiler::compile(&u::pwd(), recursive);
    let plans = resolver::resolve(&env, sandbox, &topology, resolve).await;
    let component = u::maybe_string(component, "all");
    resolver::render(plans, &component)
}

async fn create_plan(plan: Plan, _notify: bool) {
    let Plan { functions, .. } = plan.clone();

    for (_, function) in functions {
        let lang = function.runtime.lang;
        let mtask = &function.tasks.get("build");
        let dir = &function.dir.unwrap();
        match mtask {
            Some(task) => {
                builder::pack(&lang, dir, &task);
            }
            _ => {
                builder::pack(&lang, dir, "zip -q lambda.zip *.rb *.py");
            }
        }
    }
    deployer::create(plan).await;
}

fn count_of(topology: &Topology) -> String {
    let Topology { functions, .. } = topology;
    format!("{} functions", functions.len())
}

pub async fn create(
    env: Env,
    sandbox: Option<String>,
    notify: bool,
    recursive: bool,
) {
    let dir = u::pwd();
    let start = Instant::now();

    println!("Compiling topology");
    let topology = compiler::compile(&dir, recursive);

    println!("Resolving topology ({}) ", count_of(&topology).cyan());
    let plans = resolver::resolve(&env, sandbox, &topology, true).await;
    for plan in plans.clone() {
        create_plan(plan, notify).await;
    }

    if plans.len() > 0 {
        let root_plan = plans.first().unwrap();
        let Plan { namespace, sandbox, version, env, .. } = root_plan;
        let tag = format!("{}-{}", namespace, version);
        let msg = format!(
            "Deployed `{}` to *{}*::{}_{}",
            tag, &env.name, namespace, &sandbox.name
        );
        notifier::notify(namespace, &msg).await;
    }
    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

async fn update_plan(plan: Plan) {
    let Plan { dir, .. } = plan.clone();
    builder::pack_all(&dir);
    deployer::update(plan.clone()).await;
}

pub async fn update(env: Env, sandbox: Option<String>, recursive: bool) {
    let start = Instant::now();

    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    println!("Resolving topology ({}) ", count_of(&topology).cyan());
    let plans = resolver::resolve(&env, sandbox, &topology, true).await;

    for plan in plans {
        update_plan(plan).await;
    }
    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

pub async fn update_component(
    env: Env,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
) {
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    println!("Resolving topology ({}) ", count_of(&topology).cyan());
    let plans = resolver::resolve(&env, sandbox, &topology, true).await;

    for plan in plans {
        deployer::update_component(plan.clone(), component.clone()).await;
    }
}

pub async fn delete(env: Env, sandbox: Option<String>, recursive: bool) {
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    println!("Resolving topology ({}) ", count_of(&topology).cyan());
    let plans = resolver::resolve(&env, sandbox, &topology, false).await;

    for plan in plans {
        deployer::delete(plan).await
    }
}

pub async fn delete_component(
    env: Env,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
) {
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    println!("Resolving topology");
    let plans = resolver::resolve(&env, sandbox, &topology, false).await;

    for plan in plans {
        deployer::delete_component(plan, component.clone()).await
    }
}

pub async fn list(
    env: Env,
    sandbox: Option<String>,
    component: Option<String>,
    format: Option<String>,
) {
    if u::option_exists(component.clone()) {
        lister::list_component(&env, sandbox, component, format).await;
    } else {
        lister::list(&env, sandbox).await;
    }
}

pub async fn scaffold() {
    let dir = u::pwd();
    let kind = compiler::kind_of();
    match kind.as_ref() {
        "function" => {
            let function = compiler::current_function(&dir);
            match function {
                Some(f) => scaffolder::create_function(&f.name, &f.infra_dir).await,
                None => panic!("No function found"),
            }
        }
        "step-function" => {
            let functions = compiler::just_functions();
            for (_, f) in functions {
                scaffolder::create_function(&f.name, &f.infra_dir).await;
            }
        }
        _ => {
            let function = compiler::current_function(&dir);
            match function {
                Some(f) => scaffolder::create_function(&f.name, &f.infra_dir).await,
                None => panic!("No function found"),
            }
        }
    }
}

pub async fn bootstrap(
    env: Env,
    role_name: Option<String>,
    create: bool,
    delete: bool,
    show: bool,
) {
    match role_name {
        Some(role) => {
            if create {
                aws::bootstrap::create_role(&env, &role).await;
            } else if delete {
                aws::bootstrap::delete_role(&env, &role).await;
            } else if show {
                aws::bootstrap::show_role(&env, &role).await;
            } else {
                aws::bootstrap::show_role(&env, &role).await;
            }
        }
        None => println!("No role name given"),
    }
}

pub struct InvokeOptions {
    pub sandbox: Option<String>,
    pub mode: Option<String>,
    pub payload: Option<String>,
    pub name: Option<String>,
    pub kind: Option<String>,
    pub local: bool,
    pub dumb: bool,
}

pub async fn invoke(env: Env, opts: InvokeOptions) {
    let InvokeOptions {
        sandbox,
        mode,
        payload,
        name,
        local,
        kind,
        dumb,
        ..
    } = opts;

    if local {
        invoker::run_local(payload).await;
    } else {
        let inferred_kind = compiler::kind_of();
        let kind = u::maybe_string(kind, &inferred_kind);
        let sandbox = resolver::as_sandbox(sandbox);

        invoker::invoke(&env, &sandbox, &kind, name, payload, mode, dumb).await;
    }
}

pub async fn emulate(env: Env, shell: bool) {
    let kind = compiler::kind_of();
    match kind.as_ref() {
        "step-function" => emulator::sfn().await,
        "function" => {
            if shell {
                emulator::shell(&env).await;
            } else {
                emulator::lambda(&env).await;
            }
        }
        _ => emulator::lambda(&env).await,
    }
}

pub async fn tag(
    prefix: Option<String>,
    next: Option<String>,
    dry_run: bool,
    push: bool,
    suffix: Option<String>,
) {
    let prefix = match prefix {
        Some(p) => p,
        None => panic!("No prefix given")
    };
    let next = u::maybe_string(next, "patch");
    let suffix = u::maybe_string(suffix, "default");
    tagger::create_tag(&next, &prefix, &suffix, push, dry_run).await
}

pub async fn upgrade() {
    github::self_upgrade("tc", "").await
}


pub async fn init(profile: Option<String>, assume_role: Option<String>) -> Env {
    let config_path = match std::env::var("TC_CONFIG_PATH") {
        Ok(p) => kit::expand_path(&p),
        Err(_) => {
            let root = kit::sh("git rev-parse --show-toplevel", &kit::pwd());
            format!("{}/infrastructure/tc/config.toml", root)

        }
    };

    match std::env::var("TC_TRACE") {
        Ok(_) => kit::init_trace(),
        Err(_) => kit::init_log(),
    }
    aws::init(profile, assume_role, &config_path).await
}
