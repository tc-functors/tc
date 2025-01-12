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
    /// Trigger deploy via CI
    #[clap(name = "ci-deploy")]
    Deploy(DeployArgs),
    /// Trigger release via CI
    #[clap(name = "ci-release")]
    Release(ReleaseArgs),
    /// Compile a Topology
    Compile(CompileArgs),
    /// Show config
    Config(DefaultArgs),
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
    /// Create semver tags scoped by a topology
    Tag(TagArgs),
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
    #[arg(long, short = 'R')]
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
pub struct DeployArgs {
    #[arg(long, short = 'S')]
    service: Option<String>,
    #[arg(long, short = 'e')]
    env: String,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long)]
    manifest: Option<String>,
    #[arg(long, short = 'v')]
    version: String,
}

#[derive(Debug, Args)]
pub struct ReleaseArgs {
    #[arg(long, short = 's')]
    service: Option<String>,
    #[arg(long, short = 'v')]
    version: Option<String>,
    #[arg(long, short = 'r')]
    repo: Option<String>,
    #[arg(long, short = 'S')]
    suffix: Option<String>,
    #[arg(long, action, short = 'u')]
    unwind: bool,
    #[arg(long, action, short = 'g')]
    github: bool,
    #[arg(long)]
    tag: Option<String>,
    #[arg(long)]
    asset: Option<String>,
}

#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'R')]
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
    #[arg(long, short = 'R')]
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
    #[arg(long, short = 'R')]
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
    #[arg(long, short = 'R')]
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
    #[arg(long, short = 'R')]
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
    #[arg(long, short = 'R')]
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
pub struct TagArgs {
    #[arg(long, short = 'n')]
    next: Option<String>,
    #[arg(long, short = 's')]
    service: Option<String>,
    #[arg(long, action)]
    dry_run: bool,
    #[arg(long, action)]
    push: bool,
    #[arg(long, action)]
    unwind: bool,
    #[arg(long, short = 'S')]
    suffix: Option<String>,
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
    #[arg(long, short = 'r')]
    role: Option<String>,
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
    let opts = tc::BuildOpts {
        pack: pack,
        no_docker: no_docker,
        trace: trace,
        clean: clean,
        delete: delete,
        parallel: parallel,
        dirty: dirty,
        merge: merge,
    };
    tc::build(kind, name, &dir, opts).await;
}

async fn test(_args: TestArgs) {
    tc::test().await;
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

    let env = tc::init(profile, role).await;
    tc::create(env, sandbox, notify, recursive).await;
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

    let env = tc::init(profile, role).await;

    if kit::option_exists(component.clone()) {
        tc::update_component(env, sandbox, component, recursive).await;
    } else {
        tc::update(env, sandbox, recursive).await;
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

    let env = tc::init(profile, role).await;

    if kit::option_exists(component.clone()) {
        tc::delete_component(env, sandbox, component, recursive).await;
    } else {
        tc::delete(env, sandbox, recursive).await;
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
    let opts = tc::CompileOpts {
        versions: versions,
        recursive: recursive,
        component: component,
        format,
    };
    let topology = tc::compile(opts).await;
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

    let env = tc::init(profile, role).await;
    let plan = tc::resolve(env, sandbox, component, recursive).await;
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
    let opts =   tc::InvokeOptions {
        sandbox: sandbox,
        mode: mode,
        payload: payload,
        name: name,
        local: local,
        kind: kind,
        dumb: dumb,
    };

    let env = tc::init(profile, role).await;
    tc::invoke(env, opts).await;
}

async fn upgrade() {
    tc::upgrade().await
}

async fn list(args: ListArgs) {
    let ListArgs {
        profile,
        role,
        sandbox,
        component,
        format,
        ..
    } = args;
    let env = tc::init(profile, role).await;
    tc::list(env, sandbox, component, format).await;
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
    let opts = tc::PublishOpts {
        trace: trace,
        promote: promote,
        demote: demote,
        version: version,
    };
    let dir = kit::pwd();
    let env = tc::init(profile, role).await;
    if list {
        tc::list_published_assets(env).await
    } else {
        tc::publish(env, kind, name, &dir, opts).await;
    }
}

async fn scaffold(_args: ScaffoldArgs) {
    tc::scaffold().await;
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
    let env = tc::init(profile, None).await;
    tc::bootstrap(env, role, create, delete, show).await;
}

async fn emulate(args: EmulateArgs) {
    let EmulateArgs { profile, shell, .. } = args;
    let env = tc::init(profile, None).await;
    tc::emulate(env, shell).await;
}

async fn tag(args: TagArgs) {
    let TagArgs {
        service,
        next,
        dry_run,
        push,
        suffix,
        ..
    } = args;

    tc::tag(service, next, dry_run, push, suffix).await;
}

async fn deploy(args: DeployArgs) {
    let DeployArgs {
        env,
        sandbox,
        service,
        version,
        ..
    } = args;

    tc::deploy(service, env, sandbox, version).await;
}

async fn release(args: ReleaseArgs) {
    let ReleaseArgs {
        service,
        suffix,
        unwind,
        ..
    } = args;

    tc::release(service, suffix, unwind).await;
}


async fn config(_args: DefaultArgs) {
    tc::show_config().await;
}

async fn run() {
    let args = Tc::parse();

    match args.cmd {
        Cmd::Bootstrap(args) => bootstrap(args).await,
        Cmd::Build(args)     => build(args).await,
        Cmd::Config(args)    => config(args).await,
        Cmd::Compile(args)   => compile(args).await,
        Cmd::Resolve(args)   => resolve(args).await,
        Cmd::Create(args)    => create(args).await,
        Cmd::Delete(args)    => delete(args).await,
        Cmd::Deploy(args)    => deploy(args).await,
        Cmd::Emulate(args)   => emulate(args).await,
        Cmd::Invoke(args)    => invoke(args).await,
        Cmd::List(args)      => list(args).await,
        Cmd::Publish(args)   => publish(args).await,
        Cmd::Release(args)   => release(args).await,
        Cmd::Scaffold(args)  => scaffold(args).await,
        Cmd::Tag(args)       => tag(args).await,
        Cmd::Test(args)      => test(args).await,
        Cmd::Update(args)    => update(args).await,
        Cmd::Upgrade(..)     => upgrade().await,
        Cmd::Version(..)     => version().await,
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
