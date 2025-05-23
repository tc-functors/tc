mod library;
mod image;
mod layer;
mod inline;
mod extension;

use colored::Colorize;
use compiler::{
    Layer,
    Lang,
    spec::{
        BuildKind,
        BuildOutput,
        ConfigSpec,
        LangRuntime,
    },
    Build
};
use authorizer::Auth;
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

pub fn build_code(dir: &str, name: &str, langr: &LangRuntime, spec: &Build) -> String {
    match langr.to_lang() {
        Lang::Rust => inline::build(dir, name, langr, spec),
        _ => {
            let c = format!(r"{}", &spec.command);
            sh(&c, dir);
            format!("{}/lambda.zip", dir)
        }
    }
}

#[rustfmt::skip]
pub async fn build(
    dir: &str,
    name: Option<String>,
    kind: Option<BuildKind>,
    image: Option<String>,
    lang: Option<String>,
) -> Vec<BuildOutput> {

    let function = compiler::current_function(dir);

    if let Some(f) = function {

        let mut spec = f.build;

        let kind = match kind {
            Some(k) => k,
            None => spec.kind
        };

        let kind_str = &kind.to_str();

        let runtime = &f.runtime;
        let langr = match lang {
            Some(n) => &LangRuntime::from_str(&n).unwrap(),
            None => &f.runtime.lang
        };

        let name = u::maybe_string(name, &f.name);

        spec.kind = kind.clone();

        let image_kind = u::maybe_string(image, "code");

        println!("Building {} ({}/{})",
                 &name, &langr.to_str(), kind_str.blue());

        let path = match kind {
            BuildKind::Image     => image::build(dir, &name, langr, &spec.images, &image_kind, &runtime.uri),
            BuildKind::Inline    => inline::build(dir, &name, langr, &spec),
            BuildKind::Layer     => layer::build(dir, &name, langr),
            BuildKind::Library   => library::build(dir, langr),
            BuildKind::Slab      => library::build(dir, langr),
            BuildKind::Code      => build_code(dir, &name, langr, &spec),
            BuildKind::Extension => extension::build(dir, &name),
            BuildKind::Runtime   => todo!()
        };

        let out = BuildOutput {
            name: String::from(name),
            dir: dir.to_string(),
            artifact: path,
            kind: kind,
            runtime: langr.clone(),
        };
        vec![out]
    } else {
        vec![]
    }
}

fn should_build(layer: &Layer, dirty: bool) -> bool {
    if dirty {
        layer.dirty
    } else {
        &layer.kind == "implicit" || &layer.kind == "default"
    }
}

pub async fn build_recursive(
    dirty: bool,
    kind: Option<BuildKind>,
    image_kind: Option<String>,
) -> Vec<BuildOutput> {
    let mut outs: Vec<BuildOutput> = vec![];

    //TODO  parallelize

    let knd = match kind {
        Some(k) => k,
        None => BuildKind::Code,
    };

    match knd {
        BuildKind::Code => {
            let buildables = compiler::find_buildables(&u::pwd(), true);
            tracing::debug!("Building recursively {}", buildables.len());
            for b in buildables {
                let mut out = build(
                    &b.dir,
                    None,
                    Some(BuildKind::Code),
                    image_kind.clone(),
                    None,
                )
                .await;
                outs.append(&mut out);
            }
        }

        BuildKind::Layer => {
            let layers = compiler::find_layers();
            for layer in layers.clone() {
                if should_build(&layer, dirty) {
                    let mut out = build(
                        &layer.path,
                        Some(layer.name),
                        Some(BuildKind::Layer),
                        None,
                        None,
                    )
                    .await;
                    outs.append(&mut out)
                }
            }
        }

        BuildKind::Inline => {
            println!("building inline")
        }

        _ => todo!(),
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
            kit::sh(
                "rm -rf build && rm -f bootstrap",
                &b.dir,
            );
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
            _ => ()
        }
    }
}

pub async fn sync(auth: &Auth, builds: Vec<BuildOutput>) {
    println!("Attempting to sync latest code images for the following functions. This may take a while zzz...");
    for b in &builds {
        println!("{} - {}", b.name, b.artifact);
    }

    for build in builds {
        println!("Syncing {}", &build.artifact);
        match build.kind {
            BuildKind::Image => image::sync(auth, &build).await,
            _ => todo!()
        }
    }
}

pub async fn promote(
    auth: &Auth,
    name: Option<String>,
    dir: &str,
    version: Option<String>
) {
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
            _ => todo!()
        }
    } else {
        println!("No function found");
    }
}
