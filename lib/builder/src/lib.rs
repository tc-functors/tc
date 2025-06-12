mod extension;
mod image;
mod inline;
mod layer;
mod library;

use authorizer::Auth;
use colored::Colorize;
use compiler::{
    Build,
    Function,
    Lang,
    spec::{
        BuildKind,
        BuildOutput,
        ConfigSpec,
        LangRuntime,
    },
};
use kit as u;
use kit::sh;
use std::str::FromStr;

pub fn just_images(recursive: bool) -> Vec<BuildOutput> {
    let buildables = compiler::find_buildables(&u::pwd(), recursive);
    let config = ConfigSpec::new(None);
    let mut outs: Vec<BuildOutput> = vec![];
    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo,
    };

    for ref b in buildables {
        if b.kind == BuildKind::Image {
            let function = compiler::current_function(&b.dir);
            if let Some(ref f) = function {
                for (image, _) in &b.images {
                    if image == "base" {
                        let out = BuildOutput {
                            name: f.name.clone(),
                            dir: b.dir.clone(),
                            artifact: image::render_uri(&f.runtime.uri, repo),
                            kind: b.kind.clone(),
                            runtime: f.runtime.lang.clone(),
                        };
                        outs.push(out);
                    }
                }
            }
        }
    }
    outs
}

pub async fn build_code(dir: &str, name: &str, langr: &LangRuntime, spec: &Build) -> String {
    match langr.to_lang() {
        Lang::Rust => inline::build(dir, name, langr, spec).await,
        _ => {
            let c = format!(r"{}", &spec.command);
            sh(&c, dir);
            format!("{}/lambda.zip", dir)
        }
    }
}

pub async fn build(
    function: &Function,
    name: Option<String>,
    image: Option<String>,
    _layer: Option<String>,
    kind: Option<String>,
) -> Vec<BuildOutput> {
    let Function {
        dir,
        build,
        runtime,
        ..
    } = function;
    let Build { images, .. } = build;

    let langr = &runtime.lang;

    let kind = match kind {
        Some(k) => BuildKind::from_str(&k).unwrap(),
        None => build.kind.clone(),
    };
    let kind_str = &kind.to_str();

    let image_kind = u::maybe_string(image, "code");
    let name = u::maybe_string(name, &function.name);

    println!(
        "Building {} ({}/{})",
        &name,
        &langr.to_str(),
        kind_str.blue()
    );

    let path = match kind {
        BuildKind::Image => image::build(dir, &name, langr, &images, &image_kind, &runtime.uri),
        BuildKind::Inline => inline::build(dir, &name, langr, &build).await,
        BuildKind::Layer => layer::build(dir, &name, langr),
        BuildKind::Library => library::build(dir, langr),
        BuildKind::Slab => library::build(dir, langr),
        BuildKind::Code => build_code(dir, &name, langr, &build).await,
        BuildKind::Extension => extension::build(dir, &name),
        BuildKind::Runtime => todo!(),
    };

    let out = BuildOutput {
        name: String::from(name),
        dir: dir.to_string(),
        artifact: path,
        kind: kind.clone(),
        runtime: langr.clone(),
    };
    vec![out]
}

pub async fn build_recursive(
    dir: &str,
    _parallel: bool,
    image: Option<String>,
    layer: Option<String>,
) -> Vec<BuildOutput> {
    let mut outs: Vec<BuildOutput> = vec![];

    //TODO  parallelize

    let topology = compiler::compile(dir, true);

    for (_, function) in topology.functions {
        let mut out = build(&function, None, image.clone(), layer.clone(), None).await;
        outs.append(&mut out);
    }
    outs
}

pub fn clean_lang(dir: &str) {
    sh("rm -rf dist __pycache__ vendor deps.zip build", dir);
}

pub fn clean(recursive: bool) {
    let buildables = compiler::find_buildables(&u::pwd(), recursive);
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
    for build in builds {
        tracing::debug!("Publishing {}", &build.artifact);
        match build.kind {
            BuildKind::Layer | BuildKind::Library => layer::publish(auth, &build).await,
            BuildKind::Image => image::publish(auth, &build).await,
            _ => (),
        }
    }
}

pub async fn sync(auth: &Auth, builds: Vec<BuildOutput>) {
    println!(
        "Attempting to sync latest code images for the following functions. This may take a while zzz..."
    );
    for b in &builds {
        println!("{} - {}", b.name, b.artifact);
    }

    for build in builds {
        println!("Syncing {}", &build.artifact);
        match build.kind {
            BuildKind::Image => image::sync(auth, &build).await,
            _ => todo!(),
        }
    }
}

pub async fn promote(auth: &Auth, name: Option<String>, dir: &str, version: Option<String>) {
    let lang = &compiler::guess_runtime(dir);
    let layer_name = u::maybe_string(name.clone(), u::basedir(dir));
    layer::promote(auth, &layer_name, &lang.to_str(), version).await;
}

pub fn shell(dir: &str) {
    let function = compiler::current_function(dir);

    if let Some(f) = function {
        let spec = f.build;
        match spec.kind {
            BuildKind::Image => image::shell(dir, &f.runtime.uri),
            _ => todo!(),
        }
    } else {
        println!("No function found");
    }
}
