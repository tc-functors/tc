extern crate serde_derive;
use std::env;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    #[clap(name = "ci-deploy",  hide = true)]
    Deploy(DeployArgs),
    /// Trigger release via CI
    #[clap(name = "ci-release",  hide = true)]
    Release(ReleaseArgs),
    /// List or clear resolver cache
    Cache(CacheArgs),
    /// Compile a Topology
    Compile(CompileArgs),
    /// Show config
    Config(DefaultArgs),
    /// Create a sandboxed topology
    Create(CreateArgs),
    /// Delete a sandboxed topology
    Delete(DeleteArgs),
    /// Freeze a sandbox and make it immutable
    Freeze(FreezeArgs),
    /// Emulate Runtime environments
    Emulate(EmulateArgs),
    /// Inspect via browser
    Inspect(InspectArgs),
    /// Invoke a topology synchronously or asynchronously
    Invoke(InvokeArgs),
    /// List created entities
    List(ListArgs),
    /// Publish layers
    Publish(PublishArgs),
    /// Resolve a topology from functions, events, states description
    Resolve(ResolveArgs),
    /// Route events to functors
    Route(RouteArgs),
    /// Scaffold roles and infra vars
    Scaffold(ScaffoldArgs),
    /// Run unit tests for functions in the topology dir
    Test(TestArgs),
    /// Create semver tags scoped by a topology
    Tag(TagArgs),
    /// Unfreeze a sandbox and make it mutable
    Unfreeze(UnFreezeArgs),
    /// Update components
    Update(UpdateArgs),
    /// upgrade tc version
    Upgrade(UpgradeArgs),
    /// display current tc version
    Version(DefaultArgs),
    /// Generate documentation
    Doc(DocArgs),
}

#[derive(Debug, Args)]
pub struct DefaultArgs {}

#[derive(Debug, Args)]
pub struct ScaffoldArgs {}

#[derive(Debug, Args)]
pub struct InspectArgs {
    #[arg(long, action, short = 't')]
    trace: bool,
}

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
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct CacheArgs {
    #[arg(long, action)]
    clear: bool,
    #[arg(long, action)]
    list: bool,
    #[arg(long, short = 'n')]
    namespace: Option<String>,
    #[arg(long, short = 'e')]
    env: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, action, short = 't')]
    trace: bool,
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
    #[arg(long, action, short = 't')]
    trace: bool,
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
    #[arg(long, action, short = 't')]
    trace: bool,
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
    #[arg(long, action)]
    no_cache: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'k')]
    kind: Option<String>,
    #[arg(long, short = 'n')]
    name: Option<String>,
    #[arg(long, short = 'i')]
    image: Option<String>,
    #[arg(long, action)]
    clean: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    dirty: bool,
    #[arg(long, action)]
    merge: bool,
    #[arg(long, action)]
    split: bool,
    #[arg(long, action)]
    task: Option<String>,
    #[arg(long, action, short = 't')]
    trace: bool,
    #[arg(long, action, short = 'p')]
    publish: bool,
}

#[derive(Debug, Args)]
pub struct PublishArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'R')]
    role: Option<String>,
    #[arg(long, short = 'k')]
    kind: Option<String>,
    #[arg(long, action)]
    name: Option<String>,
    #[arg(long, action)]
    list: bool,
    #[arg(long, action)]
    promote: bool,
    #[arg(long, action)]
    demote: bool,
    #[arg(long, action)]
    download: bool,
    #[arg(long, action)]
    version: Option<String>,
    #[arg(long, action)]
    task: Option<String>,
    #[arg(long, action)]
    target: Option<String>,
    #[arg(long, action, short = 't')]
    trace: bool,
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
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct TestArgs {
    #[arg(long, short = 'd')]
    dir: Option<String>,
    #[arg(long, short = 'l')]
    lang: Option<String>,
    #[arg(long, action)]
    with_deps: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'R')]
    role: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'T')]
    topology: Option<String>,
    #[arg(long, action)]
    notify: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    no_cache: bool,
    #[arg(long, action, short = 't')]
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
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct UpgradeArgs {
    #[arg(long, short = 'v')]
    version: Option<String>,
    #[arg(long, action, short = 't')]
    trace: bool,
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
    no_cache: bool,
    #[arg(long, action, short = 't')]
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
    #[arg(long, action)]
    no_cache: bool,
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
    #[arg(long, action, short = 't')]
    trace: bool,
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
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct ReplArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, action, short = 't')]
    trace: bool,
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
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct EmulateArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, action, short = 's')]
    shell: bool,
    #[arg(long, action, short = 'd')]
    dev: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct RouteArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'E')]
    event: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'S')]
    service: String,
    #[arg(long, short = 'r')]
    rule: Option<String>,
    #[arg(long, action)]
    list: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct FreezeArgs {
    #[arg(long, short = 'd')]
    service: Option<String>,
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: String,
   #[arg(long, action)]
    all:  bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct UnFreezeArgs {
    #[arg(long, short = 'd')]
    service: Option<String>,
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: String,
    #[arg(long, action)]
    all: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct DocArgs {
    #[arg(long, short = 's')]
    spec: Option<String>
}

async fn version() {
    let version = option_env!("PROJECT_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    println!("{}", version);
}

async fn build(args: BuildArgs) {
    let BuildArgs {
        kind,
        name,
        recursive,
        clean,
        dirty,
        merge,
        split,
        trace,
        image,
        publish,
        profile,
        ..
    } = args;

    init_tracing(trace);

    let dir = kit::pwd();
    let opts = tc::BuildOpts {
        clean: clean,
        dirty: dirty,
        recursive: recursive,
        split: split,
        merge: merge,
        image_kind: image,
        publish: publish
    };
    let env = tc::init(profile, None).await;
    tc::build(&env, kind, name, &dir, opts).await;
}

async fn test(_args: TestArgs) {
    tc::test().await;
}

async fn create(args: CreateArgs) {
    let CreateArgs {
        profile,
        sandbox,
        notify,
        recursive,
        no_cache,
        topology,
        trace,
        ..
    } = args;

    init_tracing(trace);
    tc::create(profile, sandbox, notify, recursive, no_cache, topology).await;
}

async fn update(args: UpdateArgs) {
    let UpdateArgs {
        profile,
        role,
        sandbox,
        component,
        recursive,
        no_cache,
        trace,
        ..
    } = args;

    init_tracing(trace);
    let env = tc::init(profile, role).await;

    if kit::option_exists(component.clone()) {
        tc::update_component(env, sandbox, component, recursive).await;
    } else {
        tc::update(env, sandbox, recursive, no_cache).await;
    }
}

async fn delete(args: DeleteArgs) {
    let DeleteArgs {
        profile,
        role,
        sandbox,
        component,
        recursive,
        trace,
        ..
    } = args;

    init_tracing(trace);

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
        trace,
        ..
    } = args;

    init_tracing(trace);

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
        no_cache,
        trace,
        ..
    } = args;

    init_tracing(trace);

    let env = tc::init(profile, role).await;
    let plan = tc::resolve(env, sandbox, component, recursive, no_cache).await;
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
        name,
        local,
        kind,
        dumb,
        trace,
        ..
    } = args;

    init_tracing(trace);
    let opts =   tc::InvokeOptions {
        sandbox: sandbox,
        payload: payload,
        name: name,
        local: local,
        kind: kind,
        dumb: dumb,
    };

    let env = tc::init(profile, role).await;
    tc::invoke(env, opts).await;
}

async fn upgrade(args: UpgradeArgs) {
    let UpgradeArgs { version, .. } = args;
    tc::upgrade(version).await
}

async fn list(args: ListArgs) {
    let ListArgs {
        profile,
        role,
        sandbox,
        component,
        format,
        trace,
        ..
    } = args;
    init_tracing(trace);
    let env = tc::init(profile, role).await;
    tc::list(env, sandbox, component, format).await;
}

async fn publish(args: PublishArgs) {
    let PublishArgs {
        profile,
        name,
        promote,
        demote,
        version,
        list,
        kind,
        download,
        trace,
        ..
    } = args;

    init_tracing(trace);

    let opts = tc::PublishOpts {
        promote: promote,
        demote: demote,
        version: version,
    };
    let dir = kit::pwd();
    let env = tc::init_repo_profile(profile).await;
    if list {
        tc::list_published_assets(env, kind).await
    } else if download {
        tc::download_layer(env, name).await
    } else {
        tc::publish(env, name, &dir, opts).await;
    }
}

async fn scaffold(_args: ScaffoldArgs) {
    tc::scaffold().await;
}

async fn route(args: RouteArgs) {
    let RouteArgs {
        profile,
        event,
        service,
        sandbox,
        rule,
        trace,
        ..
    } = args;
    init_tracing(trace);
    let env = tc::init(profile, None).await;
    tc::route(env, event, service, sandbox, rule).await;
}

async fn freeze(args: FreezeArgs) {
    let FreezeArgs {
        profile,
        service,
        sandbox,
        trace,
        ..
    } = args;
    init_tracing(trace);
    let env = tc::init(profile, None).await;
    tc::freeze(env, service, sandbox).await;
}

async fn unfreeze(args: UnFreezeArgs) {
    let UnFreezeArgs {
        profile,
        service,
        sandbox,
        trace,
        ..
    } = args;
    init_tracing(trace);
    let env = tc::init(profile, None).await;
    tc::unfreeze(env, service, sandbox).await;
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
    let EmulateArgs { profile, dev, shell, trace, .. } = args;
    init_tracing(trace);
    let env = tc::init_repo_profile(profile).await;
    tc::emulate(env, dev, shell).await;
}

async fn tag(args: TagArgs) {
    let TagArgs {
        service,
        next,
        dry_run,
        push,
        suffix,
        trace,
        ..
    } = args;

    init_tracing(trace);
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


async fn cache(args: CacheArgs) {
    let CacheArgs {
        clear,
        namespace,
        env,
        sandbox,
        ..
    } = args;

    if clear {
        tc::clear_cache().await;
    } else {
        tc::list_cache(namespace, env, sandbox).await;
    }
}


async fn config(_args: DefaultArgs) {
    tc::show_config().await;
}

async fn inspect(args: InspectArgs) {
    let InspectArgs { trace } = args;
    init_tracing(trace);
    tc::inspect().await;
}

fn init_tracing(trace: bool) {
    let should_trace = trace || match env::var("TC_TRACE") {
        Ok(t) => &t == "1",
        Err(_) => trace
    };
    if should_trace {
        let filter = Targets::new()
            .with_target("tc", tracing::Level::DEBUG)
            .with_default(tracing::Level::DEBUG)
            .with_target("sqlx", LevelFilter::OFF);

        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(filter)
            .init();
    }
}


async fn doc(args: DocArgs) {
    let DocArgs { spec } = args;

    match spec {
        Some(s) => tc::generate_doc(&s),
        None => clap_markdown::print_help_markdown::<Tc>()
    }
}

async fn run() {
    let args = Tc::parse();

    match args.cmd {
        Cmd::Bootstrap(args) => bootstrap(args).await,
        Cmd::Build(args)     => build(args).await,
        Cmd::Cache(args)     => cache(args).await,
        Cmd::Config(args)    => config(args).await,
        Cmd::Doc(args)       => doc(args).await,
        Cmd::Compile(args)   => compile(args).await,
        Cmd::Resolve(args)   => resolve(args).await,
        Cmd::Create(args)    => create(args).await,
        Cmd::Delete(args)    => delete(args).await,
        Cmd::Deploy(args)    => deploy(args).await,
        Cmd::Emulate(args)   => emulate(args).await,
        Cmd::Freeze(args)    => freeze(args).await,
        Cmd::Inspect(args)   => inspect(args).await,
        Cmd::Invoke(args)    => invoke(args).await,
        Cmd::List(args)      => list(args).await,
        Cmd::Publish(args)   => publish(args).await,
        Cmd::Release(args)   => release(args).await,
        Cmd::Route(args)     => route(args).await,
        Cmd::Scaffold(args)  => scaffold(args).await,
        Cmd::Tag(args)       => tag(args).await,
        Cmd::Test(args)      => test(args).await,
        Cmd::Unfreeze(args)  => unfreeze(args).await,
        Cmd::Update(args)    => update(args).await,
        Cmd::Upgrade(args)   => upgrade(args).await,
        Cmd::Version(..)     => version().await,

    }
}

#[tokio::main]
async fn main() {
    run().await
}
