extern crate serde_derive;
use std::env;

extern crate log;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
struct Tc {
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Bootstrap IAM roles, extensions etc.
    Bootstrap(BootstrapArgs),
    /// Build layers, extensions and pack function code
    Build(BuildArgs),
    /// Compile a Topology
    Compile(CompileArgs),
    /// Create a sandboxed topology
    Create(CreateArgs),
    /// Delete a sandboxed topology
    Delete(DeleteArgs),
    /// Emulate Runtime environments
    Emulate(EmulateArgs),
    /// Invoke a topology synchronously or asynchronously
    Invoke(InvokeArgs),
    /// List created entities
    List(ListArgs),
    /// Publish layers
    Publish(PublishArgs),
    /// Resolve a topology from functions, events, states description
    Resolve(ResolveArgs),
    /// Scaffold roles and infra vars
    Scaffold(ScaffoldArgs),
    /// Run unit tests for functions in the topology dir
    Test(TestArgs),
    /// Update components
    Update(UpdateArgs),
    /// upgrade tc version
    Upgrade(DefaultArgs),
    /// display current tc version
    Version(DefaultArgs),
}

#[derive(Debug, Args)]
pub struct DefaultArgs {}

#[derive(Debug, Args)]
pub struct ScaffoldArgs {}

#[derive(Debug, Args)]
pub struct BootstrapArgs {
    #[arg(long, short = 'r')]
    role: Option<String>,
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, action)]
    create: bool,
    #[arg(long, action)]
    delete: bool,
    #[arg(long, action)]
    show: bool,
}

#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'r')]
    role: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'c')]
    component: Option<String>,
    #[arg(long, action, short = 'q')]
    quiet: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    diff: bool,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, action)]
    kind: Option<String>,
    #[arg(long, action)]
    name: Option<String>,
    #[arg(long, action)]
    pack: bool,
    #[arg(long, action)]
    no_docker: bool,
    #[arg(long, action)]
    clean: bool,
    #[arg(long, action)]
    delete: bool,
    #[arg(long, action)]
    trace: bool,
    #[arg(long, action)]
    parallel: bool,
    #[arg(long, action)]
    dirty: bool,
    #[arg(long, action)]
    merge: bool,
    #[arg(long, action)]
    task: Option<String>,
}

#[derive(Debug, Args)]
pub struct PublishArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'r')]
    role: Option<String>,
    #[arg(long, action)]
    kind: Option<String>,
    #[arg(long, action)]
    name: Option<String>,
    #[arg(long, action)]
    list: bool,
    #[arg(long, action)]
    trace: bool,
    #[arg(long, action)]
    promote: bool,
    #[arg(long, action)]
    demote: bool,
    #[arg(long, action)]
    version: Option<String>,
    #[arg(long, action)]
    task: Option<String>,
    #[arg(long, action)]
    target: Option<String>,
}

#[derive(Debug, Args)]
pub struct CompileArgs {
    #[arg(long, action)]
    versions: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, short = 'c')]
    component: Option<String>,
    #[arg(long, short = 'f')]
    format: Option<String>,
}

#[derive(Debug, Args)]
pub struct TestArgs {
    #[arg(long, short = 'd')]
    dir: Option<String>,
    #[arg(long, short = 'l')]
    lang: Option<String>,
    #[arg(long, action)]
    with_deps: bool,
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'r')]
    role: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, action)]
    notify: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    #[arg(long, short = 'f')]
    from: String,
    #[arg(long, short = 't')]
    to: String,
    #[arg(long, action)]
    dry_run: bool,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'r')]
    role: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'c')]
    component: Option<String>,
    #[arg(long, short = 'a')]
    asset: Option<String>,
    #[arg(long, action)]
    notify: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct DeleteArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'r')]
    role: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'c')]
    component: Option<String>,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct InvokeArgs {
    #[arg(long, short = 'p')]
    payload: Option<String>,
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'r')]
    role: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'M')]
    mode: Option<String>,
    #[arg(long, short = 'n')]
    name: Option<String>,
    #[arg(long, short = 'S')]
    step: Option<String>,
    #[arg(long, short = 'k')]
    kind: Option<String>,
    #[arg(long, action)]
    local: bool,
    #[arg(long, action)]
    dumb: bool,
}

#[derive(Debug, Args)]
pub struct ReplArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'c')]
    component: Option<String>,
    #[arg(long, short = 'f')]
    format: Option<String>,
}

#[derive(Debug, Args)]
pub struct EmulateArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, action, short = 's')]
    shell: bool,
}

async fn version() {
    let version = option_env!("PROJECT_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    println!("{}", version);
}

async fn build(args: BuildArgs) {
    let BuildArgs {
        kind,
        name,
        pack,
        no_docker,
        clean,
        trace,
        delete,
        parallel,
        dirty,
        merge,
        ..
    } = args;

    let dir = kit::pwd();
    let opts = libtc::BuildOpts {
        pack: pack,
        no_docker: no_docker,
        trace: trace,
        clean: clean,
        delete: delete,
        parallel: parallel,
        dirty: dirty,
        merge: merge,
    };
    libtc::build(kind, name, &dir, opts).await;
}

async fn test(_args: TestArgs) {
    libtc::test().await;
}

async fn create(args: CreateArgs) {
    let CreateArgs {
        profile,
        role,
        sandbox,
        notify,
        recursive,
        ..
    } = args;

    let env = libtc::init(profile, role).await;
    libtc::create(env, sandbox, notify, recursive).await;
}

async fn update(args: UpdateArgs) {
    let UpdateArgs {
        profile,
        role,
        sandbox,
        component,
        recursive,
        ..
    } = args;

    let env = libtc::init(profile, role).await;

    if kit::option_exists(component.clone()) {
        libtc::update_component(env, sandbox, component, recursive).await;
    } else {
        libtc::update(env, sandbox, recursive).await;
    }
}

async fn delete(args: DeleteArgs) {
    let DeleteArgs {
        profile,
        role,
        sandbox,
        component,
        recursive,
        ..
    } = args;

    let env = libtc::init(profile, role).await;

    if kit::option_exists(component.clone()) {
        libtc::delete_component(env, sandbox, component, recursive).await;
    } else {
        libtc::delete(env, sandbox, recursive).await;
    }
}

async fn compile(args: CompileArgs) {
    let CompileArgs {
        versions,
        recursive,
        component,
        format,
        ..
    } = args;
    let opts = libtc::CompileOpts {
        versions: versions,
        recursive: recursive,
        component: component,
        format,
    };
    let topology = libtc::compile(opts).await;
    println!("{topology}");
}

async fn resolve(args: ResolveArgs) {
    let ResolveArgs {
        profile,
        role,
        sandbox,
        component,
        quiet,
        recursive,
        ..
    } = args;

    let profile = libtc::init(&env, role).await;
    let plan = libtc::resolve(env, sandbox, component, recursive).await;
    if !quiet {
        println!("{plan}");
    }
}

async fn invoke(args: InvokeArgs) {
    let InvokeArgs {
        profile,
        role,
        payload,
        sandbox,
        mode,
        name,
        local,
        kind,
        dumb,
        ..
    } = args;
    let opts =   libtc::InvokeOptions {
        sandbox: sandbox,
        mode: mode,
        payload: payload,
        name: name,
        local: local,
        kind: kind,
        dumb: dumb,
    };

    let env = libtc::init(profile, role).await;
    libtc::invoke(env, opts).await;
}

async fn upgrade() {
    libtc::upgrade().await
}

async fn list(args: ListArgs) {
    let ListArgs {
        profile,
        sandbox,
        component,
        format,
        ..
    } = args;
    let env = libtc::init(profile, role).await;
    libtc::list(env, sandbox, component, format).await;
}

async fn publish(args: PublishArgs) {
    let PublishArgs {
        profile,
        role,
        kind,
        name,
        promote,
        demote,
        version,
        list,
        trace,
        ..
    } = args;
    let opts = libtc::PublishOpts {
        trace: trace,
        promote: promote,
        demote: demote,
        version: version,
    };
    let dir = kit::pwd();
    let env = libtc::init(profile, role);
    if list {
        libtc::list_published_assets(env).await
    } else {
        libtc::publish(env, kind, name, &dir, opts).await;
    }
}

async fn scaffold(_args: ScaffoldArgs) {
    libtc::scaffold().await;
}

async fn bootstrap(args: BootstrapArgs) {
    let BootstrapArgs {
        profile,
        role,
        create,
        delete,
        show,
        ..
    } = args;
    let env = libtc::init(profile, None).await;
    libtc::bootstrap(env, role, create, delete, show).await;
}

async fn emulate(args: EmulateArgs) {
    let EmulateArgs { env, shell, .. } = args;
    let env = libc::init(env, None);
    libtc::emulate(env, shell).await;
}

async fn run() {
    let args = Tc::parse();

    match args.cmd {
        Cmd::Bootstrap(args) => bootstrap(args).await,
        Cmd::Build(args) => build(args).await,
        Cmd::Compile(args) => compile(args).await,
        Cmd::Resolve(args) => resolve(args).await,
        Cmd::Create(args) => create(args).await,
        Cmd::Delete(args) => delete(args).await,
        Cmd::Emulate(args) => emulate(args).await,
        Cmd::Invoke(args) => invoke(args).await,
        Cmd::List(args) => list(args).await,
        Cmd::Publish(args) => publish(args).await,
        Cmd::Scaffold(args) => scaffold(args).await,
        Cmd::Test(args) => test(args).await,
        Cmd::Update(args) => update(args).await,
        Cmd::Upgrade(..) => upgrade().await,
        Cmd::Version(..) => version().await,
    }
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "tc");
    env::set_var("AWS_MAX_ATTEMPTS", "10");
    env::set_var("DOCKER_BUILDKIT", "1");
    env::set_var("AWS_RETRY_MODE", "standard");
    env::set_var("DOCKER_DEFAULT_PLATFORM", "linux/amd64");

    run().await
}
