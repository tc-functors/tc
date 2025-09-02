mod code;
mod extension;
mod image;
mod inline;
mod layer;
mod library;
pub mod page;
mod types;

use crate::types::BuildOutput;
use authorizer::Auth;
use colored::Colorize;
use composer::{
    Function,
    spec::{
        BuildKind,
        ConfigSpec,
    },
};
use kit as u;
use kit::sh;
use std::{
    panic,
    str::FromStr,
};

pub fn just_images(recursive: bool) -> Vec<BuildOutput> {
    let buildables = composer::find_buildables(&u::pwd(), recursive);
    let config = ConfigSpec::new(None);
    let mut outs: Vec<BuildOutput> = vec![];
    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo,
    };

    for ref b in buildables {
        if b.kind == BuildKind::Image {
            let function = composer::current_function(&b.dir);
            if let Some(ref f) = function {
                let out = BuildOutput {
                    name: f.name.clone(),
                    dir: b.dir.clone(),
                    artifact: image::render_uri(&f.runtime.uri, repo),
                    kind: b.kind.clone(),
                    runtime: f.runtime.lang.clone(),
                    version: b.version.clone(),
                };
                outs.push(out);
            }
        }
    }
    outs
}

async fn init(profile: Option<String>, assume_role: Option<String>) -> Auth {
    match std::env::var("TC_ASSUME_ROLE") {
        Ok(_) => {
            let role = match assume_role {
                Some(r) => Some(r),
                None => {
                    let config = composer::config(&kit::pwd());
                    let p = u::maybe_string(profile.clone(), "default");
                    config.ci.roles.get(&p).cloned()
                }
            };
            Auth::new(profile.clone(), role).await
        }
        Err(_) => Auth::new(profile.clone(), assume_role).await,
    }
}

async fn init_centralized_auth(given_auth: &Auth) -> Auth {
    let config = ConfigSpec::new(None);
    let profile = config.aws.lambda.layers_profile.clone();
    match profile {
        Some(_) => {
            let cauth = init(profile.clone(), None).await;
            let centralized = cauth
                .assume(profile.clone(), config.role_to_assume(profile))
                .await;
            centralized
        }
        None => given_auth.clone(),
    }
}

pub async fn build(
    auth: &Auth,
    function: &Function,
    name: Option<String>,
    kind: Option<String>,
    code_only: bool,
) -> Vec<BuildOutput> {
    let Function {
        dir,
        build,
        runtime,
        ..
    } = function;

    let langr = &runtime.lang;

    let kind = match kind {
        Some(k) => BuildKind::from_str(&k).unwrap(),
        None => build.kind.clone(),
    };

    let name = u::maybe_string(name, &function.name);
    let auth = init_centralized_auth(auth).await;

    let build_status = match kind {
        BuildKind::Image => {
            image::build(&auth, dir, &name, langr, &runtime.uri, &build, code_only).await
        }
        BuildKind::Inline => inline::build(&auth, dir, &name, langr, &build).await,
        BuildKind::Layer => layer::build(dir, &name, langr, &build),
        BuildKind::Library => library::build(dir, langr),
        BuildKind::Slab => library::build(dir, langr),
        BuildKind::Code => code::build(&auth, dir, &name, langr, &build).await,
        BuildKind::Extension => extension::build(dir, &name, langr),
        BuildKind::Runtime => todo!(),
    };

    if !build_status.status {
        println!("{}", build_status.out.red());
        println!("{}", build_status.err);
        panic::set_hook(Box::new(|_| {
            println!("Build Failed");
        }));
        panic!("Build failed")
    }

    let out = BuildOutput {
        name: String::from(name),
        dir: dir.to_string(),
        artifact: build_status.path,
        kind: kind.clone(),
        runtime: langr.clone(),
        version: build.version.clone(),
    };
    vec![out]
}

pub async fn build_recursive(auth: &Auth, dir: &str, _parallel: bool) -> Vec<BuildOutput> {
    let mut outs: Vec<BuildOutput> = vec![];

    //TODO  parallelize

    let topology = composer::compose(dir, true);

    for (_, function) in topology.functions {
        let mut out = build(auth, &function, None, None, false).await;
        outs.append(&mut out);
    }
    outs
}

pub fn clean_lang(dir: &str) {
    sh("rm -rf dist __pycache__ vendor deps.zip build", dir);
}

pub fn clean(recursive: bool) {
    let buildables = composer::find_buildables(&u::pwd(), recursive);
    for b in buildables {
        if b.kind == BuildKind::Inline {
            kit::sh("rm -rf build && rm -f bootstrap", &b.dir);
        } else {
            kit::sh(
                "rm -rf lambda.zip deps.zip build && rm -f bootstrap",
                &b.dir,
            );
        }
    }
}

pub async fn publish(auth: &Auth, builds: Vec<BuildOutput>) {
    let auth = init_centralized_auth(auth).await;
    for build in builds {
        tracing::debug!("Publishing {}", &build.artifact);
        match build.kind {
            BuildKind::Layer | BuildKind::Library => layer::publish(&auth, &build).await,
            BuildKind::Image => image::publish(&auth, &build).await,
            _ => (),
        }
    }
}

pub async fn sync(auth: &Auth, builds: Vec<BuildOutput>) {
    let auth = init_centralized_auth(auth).await;
    println!(
        "Attempting to sync latest code images for the following functions. This may take a while zzz..."
    );
    for b in &builds {
        println!("{} - {}", b.name, b.artifact);
    }

    for build in builds {
        println!("Syncing {}", &build.artifact);
        match build.kind {
            BuildKind::Image => image::sync(&auth, &build).await,
            _ => todo!(),
        }
    }
}

pub async fn promote(auth: &Auth, name: &str, dir: &str, version: Option<String>) {
    let lang = &composer::guess_runtime(dir);
    layer::promote(auth, name, &lang.to_str(), version).await;
}

pub async fn shell(auth: &Auth, dir: &str, kind: Option<String>) {
    let auth = init_centralized_auth(auth).await;
    let function = composer::current_function(dir);

    if let Some(f) = function {
        let spec = f.build;

        let kind = match kind {
            Some(k) => BuildKind::from_str(&k).unwrap(),
            None => spec.kind.clone(),
        };

        image::shell(&auth, dir, &f.runtime.uri, spec.version, kind).await
    } else {
        println!("No function found");
    }
}
