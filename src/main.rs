extern crate serde_derive;
use std::env;
use tracing_subscriber::{
    filter::{
        LevelFilter,
        Targets,
    },
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

extern crate log;
use clap::{
    Args,
    Parser,
    Subcommand,
};

#[derive(Debug, Parser)]
struct Tc {
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Build layers, extensions and pack function code
    Build(BuildArgs),
    /// Trigger deploy via CI
    #[clap(name = "ci-deploy", hide = true)]
    Deploy(DeployArgs),
    /// Trigger release via CI
    #[clap(name = "ci-release", hide = true)]
    Release(ReleaseArgs),
    /// List or clear resolver cache
    #[clap(hide = true)]
    Cache(CacheArgs),
    /// Generate changelog for topology
    Changelog(ChangelogArgs),
    /// Compose a Topology
    Compose(ComposeArgs),
    /// Show config
    #[clap(hide = true)]
    Config(DefaultArgs),
    /// Create a sandboxed topology
    Create(CreateArgs),
    /// Delete a sandboxed topology
    Delete(DeleteArgs),
    /// Freeze a sandbox and make it immutable
    Freeze(FreezeArgs),
    /// Invoke a topology synchronously or asynchronously
    Invoke(InvokeArgs),
    /// List resources in a topology
    List(ListArgs),
    /// Prune all resources in given sandbox
    Prune(PruneArgs),
    /// Resolve a topology
    Resolve(ResolveArgs),
    /// Route traffic to the given sandbox
    Route(RouteArgs),
    /// Scaffold functions
    Scaffold(ScaffoldArgs),
    /// Snapshot of current sandbox and env
    Snapshot(SnapshotArgs),
    /// Run tests in topology
    Test(TestArgs),
    /// Create semver tags scoped by a topology
    Tag(TagArgs),
    /// Unfreeze a sandbox and make it mutable
    Unfreeze(UnFreezeArgs),
    /// Update entity and components
    Update(UpdateArgs),
    /// upgrade tc version
    Upgrade(UpgradeArgs),
    /// display current tc version
    Version(DefaultArgs),
    /// Generate documentation
    #[clap(hide = true)]
    Doc(DefaultArgs),
}

#[derive(Debug, Args)]
pub struct DefaultArgs {}

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
    #[arg(long, short = 't', alias = "service")]
    topology: Option<String>,
    #[arg(long, short = 'e')]
    env: String,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long)]
    manifest: Option<String>,
    #[arg(long, short = 'v')]
    version: Option<String>,
    #[arg(long, short = 'b')]
    branch: Option<String>,
}

#[derive(Debug, Args)]
pub struct ReleaseArgs {
    #[arg(long, short = 't', alias = "service")]
    topology: Option<String>,
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
    entity: Option<String>,
    #[arg(long, action, short = 'q')]
    quiet: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    diff: bool,
    #[arg(long, action)]
    cache: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct SnapshotArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'f')]
    format: Option<String>,
    #[arg(long, action, short = 'm')]
    manifest: bool,
    #[arg(long, short = 'S')]
    save: Option<String>,
    #[arg(long, alias = "target-profile")]
    target_profile: Option<String>,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 'n')]
    name: Option<String>,
    #[arg(long, short = 'k')]
    kind: Option<String>,
    #[arg(long, short = 'i')]
    image: Option<String>,
    #[arg(long, short = 'l')]
    layer: Option<String>,
    #[arg(long, short = 'v')]
    version: Option<String>,
    #[arg(long, action)]
    clean: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
    #[arg(long, action, short = 'p')]
    publish: bool,
    #[arg(long, action)]
    promote: bool,
    #[arg(long, action)]
    shell: bool,
    #[arg(long, action, short = 's', alias = "sync-to-local")]
    sync: bool,
    #[arg(long, action)]
    parallel: bool,
    #[arg(long, action)]
    remote: bool,
}

#[derive(Debug, Args)]
pub struct PromoteArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, action)]
    name: Option<String>,
    #[arg(long, action)]
    version: Option<String>,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct ComposeArgs {
    #[arg(long, action)]
    versions: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    root: bool,
    #[arg(long, short = 'c')]
    entity: Option<String>,
    #[arg(long, short = 'd')]
    dir: Option<String>,
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
    cache: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
    #[arg(long, action, short = 'd')]
    dirty: bool,
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
    entity: Option<String>,
    #[arg(long, short = 'a')]
    asset: Option<String>,
    #[arg(long, action)]
    notify: bool,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    cache: bool,
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
    entity: Option<String>,
    #[arg(long, action, short = 'r')]
    recursive: bool,
    #[arg(long, action)]
    cache: bool,
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
    #[arg(long, short = 'c')]
    entity: Option<String>,
    #[arg(long, short = 'd')]
    dir: Option<String>,
    #[arg(long, action)]
    local: bool,
    #[arg(long, action)]
    dumb: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'c')]
    entity: Option<String>,
    #[arg(long, short = 'f')]
    format: Option<String>,
    #[arg(long, action, short = 'v')]
    versions: bool,
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
pub struct PruneArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, short = 'f')]
    filter: Option<String>,
    #[arg(long, action, alias = "dry-run")]
    dry_run: bool,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct ChangelogArgs {
    #[arg(long, short = 'b')]
    between: Option<String>,
    #[arg(long, short = 's')]
    search: Option<String>,
    #[arg(long, action, short = 'v')]
    verbose: bool,
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
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct UnFreezeArgs {
    #[arg(long, short = 'e')]
    profile: Option<String>,
    #[arg(long, short = 's')]
    sandbox: Option<String>,
    #[arg(long, action, short = 't')]
    trace: bool,
}

#[derive(Debug, Args)]
pub struct ScaffoldArgs {
    #[arg(long, short = 'k')]
    kind: Option<String>,
    #[arg(long, short = 'd')]
    dir: Option<String>,
}

async fn version() {
    let version = option_env!("PROJECT_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    println!("{}", version);
}

async fn build(args: BuildArgs) {
    let BuildArgs {
        name,
        recursive,
        clean,
        trace,
        kind,
        image,
        layer,
        publish,
        promote,
        version,
        profile,
        parallel,
        sync,
        remote,
        shell,
        ..
    } = args;

    init_tracing(trace);

    let dir = kit::pwd();
    let opts = tc::BuildOpts {
        clean: clean,
        recursive: recursive,
        kind: kind,
        image: image,
        layer: layer,
        sync: sync,
        publish: publish,
        parallel: parallel,
        promote: promote,
        version: version,
        remote: remote,
        shell: shell,
    };
    tc::build(profile, name, &dir, opts).await;
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
        cache,
        topology,
        trace,
        dirty,
        ..
    } = args;

    init_tracing(trace);
    tc::create(profile, sandbox, notify, recursive, cache, topology, dirty).await;
}

async fn update(args: UpdateArgs) {
    let UpdateArgs {
        profile,
        role,
        sandbox,
        entity,
        recursive,
        cache,
        trace,
        ..
    } = args;

    init_tracing(trace);
    let env = tc::init(profile, role).await;

    tc::update(env, sandbox, entity, recursive, cache).await;
}

async fn delete(args: DeleteArgs) {
    let DeleteArgs {
        profile,
        role,
        sandbox,
        entity,
        recursive,
        trace,
        cache,
        ..
    } = args;

    init_tracing(trace);

    let env = tc::init(profile, role).await;
    tc::delete(env, sandbox, entity, recursive, cache).await;
}

async fn compose(args: ComposeArgs) {
    let ComposeArgs {
        versions,
        recursive,
        entity,
        format,
        trace,
        root,
        dir,
        ..
    } = args;

    init_tracing(trace);

    let opts = tc::ComposeOpts {
        versions: versions,
        recursive: recursive,
        entity: entity,
        format: format.clone(),
    };
    if root {
        tc::compose_root(dir, format).await;
    } else {
        tc::compose(opts).await;
    }

}

async fn resolve(args: ResolveArgs) {
    let ResolveArgs {
        profile,
        role,
        sandbox,
        entity,
        recursive,
        cache,
        trace,
        ..
    } = args;

    init_tracing(trace);

    let env = tc::init(profile, role).await;
    tc::resolve(env, sandbox, entity, recursive, cache).await;
}

async fn invoke(args: InvokeArgs) {
    let InvokeArgs {
        profile,
        role,
        payload,
        sandbox,
        local,
        entity,
        dumb,
        trace,
        dir,
        ..
    } = args;

    init_tracing(trace);
    let opts = tc::InvokeOptions {
        sandbox: sandbox,
        payload: payload,
        dir: dir,
        local: local,
        entity: entity,
        dumb: dumb,
    };

    let env = tc::init(profile, role).await;
    tc::invoke(env, opts).await;
}

async fn upgrade(args: UpgradeArgs) {
    let UpgradeArgs { version, .. } = args;
    tc::upgrade(version).await
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
        sandbox,
        trace,
        ..
    } = args;
    init_tracing(trace);
    let env = tc::init(profile, None).await;
    tc::freeze(env, sandbox).await;
}

async fn unfreeze(args: UnFreezeArgs) {
    let UnFreezeArgs {
        profile,
        sandbox,
        trace,
        ..
    } = args;
    init_tracing(trace);
    let env = tc::init(profile, None).await;
    tc::unfreeze(env, sandbox).await;
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

async fn ci_deploy(args: DeployArgs) {
    let DeployArgs {
        env,
        sandbox,
        topology,
        version,
        ..
    } = args;

    tc::ci_deploy(topology, env, sandbox, version).await;
}

async fn ci_release(args: ReleaseArgs) {
    let ReleaseArgs {
        topology,
        suffix,
        unwind,
        ..
    } = args;

    tc::ci_release(topology, suffix, unwind).await;
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

async fn snapshot(args: SnapshotArgs) {
    let SnapshotArgs {
        profile,
        sandbox,
        format,
        save,
        manifest,
        target_profile,
        ..
    } = args;

    tc::snapshot(profile, sandbox, format, manifest, save, target_profile).await;
}

async fn changelog(args: ChangelogArgs) {
    let ChangelogArgs {
        search,
        between,
        verbose,
        ..
    } = args;

    tc::changelog(between, search, verbose).await;
}

fn init_tracing(trace: bool) {
    let should_trace = trace
        || match env::var("TC_TRACE") {
            Ok(t) => &t == "1",
            Err(_) => trace,
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

async fn doc(_args: DefaultArgs) {
    clap_markdown::print_help_markdown::<Tc>()
}

async fn prune(args: PruneArgs) {
    let PruneArgs {
        profile,
        sandbox,
        filter,
        trace,
        dry_run,
        ..
    } = args;
    init_tracing(trace);
    let env = tc::init(profile, None).await;
    tc::prune(&env, sandbox, filter, dry_run).await;
}

async fn scaffold(args: ScaffoldArgs) {
    tc::scaffold(args.kind);
}

async fn list(args: ListArgs) {
    let ListArgs {
        profile,
        sandbox,
        trace,
        entity,
        ..
    } = args;
    init_tracing(trace);
    let env = tc::init(profile, None).await;
    tc::list(&env, sandbox, entity).await;
}


async fn run() {
    let args = Tc::parse();

    match args.cmd {
        Cmd::Build(args)     => build(args).await,
        Cmd::Cache(args)     => cache(args).await,
        Cmd::Config(args)    => config(args).await,
        Cmd::Doc(args)       => doc(args).await,
        Cmd::Compose(args)   => compose(args).await,
        Cmd::Resolve(args)   => resolve(args).await,
        Cmd::Create(args)    => create(args).await,
        Cmd::Delete(args)    => delete(args).await,
        Cmd::Freeze(args)    => freeze(args).await,
        Cmd::Invoke(args)    => invoke(args).await,
        Cmd::List(args)      => list(args).await,
        Cmd::Prune(args)     => prune(args).await,
        Cmd::Route(args)     => route(args).await,
        Cmd::Snapshot(args)  => snapshot(args).await,
        Cmd::Tag(args)       => tag(args).await,
        Cmd::Test(args)      => test(args).await,
        Cmd::Unfreeze(args)  => unfreeze(args).await,
        Cmd::Update(args)    => update(args).await,
        Cmd::Upgrade(args)   => upgrade(args).await,
        Cmd::Changelog(args) => changelog(args).await,
        Cmd::Version(..)     => version().await,
        Cmd::Scaffold(args)  => scaffold(args).await,
        Cmd::Release(args)   => ci_release(args).await,
        Cmd::Deploy(args)    => ci_deploy(args).await,
    }
}

#[tokio::main]
async fn main() {
    run().await
}
