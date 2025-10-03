mod code;
mod extension;
mod image;
mod inline;
mod layer;
mod library;
pub mod page;
mod types;

use crate::types::BuildOutput;
use colored::Colorize;
use compiler::spec::function::build::BuildKind;
use composer::Function;
use configurator::Config;
use kit as u;
use kit::sh;
use provider::Auth;
use std::{
    panic,
    str::FromStr,
};

pub fn just_images(recursive: bool) -> Vec<BuildOutput> {
    let buildables = composer::find_buildables(&u::pwd(), recursive);
    let config = Config::new();
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
    let auth = provider::init_centralized_auth(auth).await;

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
    println!("Compiling spec...");
    let spec = compiler::compile(dir, true);
    println!("Composing topology...");
    let topology = composer::compose(&spec);

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
    let auth = provider::init_centralized_auth(auth).await;
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
    let auth = provider::init_centralized_auth(auth).await;
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
    let lang = &compiler::guess_runtime(dir);
    layer::promote(auth, name, &lang.to_str(), version).await;
}

pub async fn shell(auth: &Auth, dir: &str, kind: Option<String>) {
    let auth = provider::init_centralized_auth(auth).await;
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
