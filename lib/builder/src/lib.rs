mod python;
mod ruby;
mod rust;
mod clojure;

use std::str::FromStr;
use compiler::Layer;
use glob::glob;
use kit as u;
use kit::sh;
use serde_derive::{Serialize, Deserialize};
use compiler::spec::{LangRuntime, Lang, BuildKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOutput {
    pub name: String,
    pub dir: String,
    pub runtime: LangRuntime,
    pub kind: BuildKind,
    pub zipfile: String,
}

fn _should_split(dir: &str) -> bool {
    let zipfile = "deps.zip";
    let size;
    if u::path_exists(dir, zipfile) {
        size = u::path_size(dir, zipfile);
    } else {
        return false;
    }
    size >= 70000000.0
}

fn _split(dir: &str, name: &str, kind: &BuildKind, runtime: &LangRuntime) -> Vec<BuildOutput> {
    let zipfile = format!("{}/deps.zip", dir);
    let size;
    if u::file_exists(&zipfile) {
        size = u::file_size(&zipfile);
    } else {
        panic!("No zip found");
    }
    if size >= 70000000.0 {
        let cmd = format!("zipsplit {} -n 50000000", zipfile);
        u::runcmd_stream(&cmd, dir);
    }
    let zips = glob("deps*.zip").expect("Failed to read glob pattern");
    let mut outs: Vec<BuildOutput> = vec![];
    for (n, entry) in zips.enumerate() {
        match entry {
            Ok(z) => {
                if &z.to_string_lossy() != &zipfile && n != 0 {
                    let zname = format!("{}-{}", name, n);
                    let out = BuildOutput {
                        name: zname,
                        dir: dir.to_string(),
                        runtime: runtime.clone(),
                        kind: kind.clone(),
                        zipfile: z.to_string_lossy().to_string(),
                    };
                    outs.push(out);
                }
            }
            Err(_e) => (),
        }
    }
    outs
}

pub fn just_build_out(dir: &str, name: &str, lang_str: &str) -> Vec<BuildOutput> {
    let runtime = LangRuntime::from_str(lang_str).expect("Failed to parse lang str");

    let zipfile = format!("{}/deps.zip", dir);
    let out = BuildOutput {
        name: name.to_owned(),
        dir: dir.to_string(),
        runtime: runtime,
        kind: BuildKind::Code,
        zipfile: zipfile,
    };
    vec![out]
}


pub async fn build(dir: &str, name: Option<String>, kind: Option<BuildKind>, trace: bool) -> Vec<BuildOutput> {
    let runtime = compiler::guess_runtime(dir);
    let lang = runtime.to_lang();
    let name = u::maybe_string(name, u::basedir(dir));
    let function = compiler::current_function(dir);

    if let Some(f) = function {

        let mut b = f.build;

        let kind = match kind {
            Some(k) => k,
            None => BuildKind::Code
        };

        println!("Building {} ({:?})", &name, &kind);
        b.kind = kind;


        sh("rm -f *.zip", dir);
        sh("rm -rf build", dir);
        let out = match lang {
            Lang::Ruby    => ruby::build(dir, runtime, &name, b, trace),
            Lang::Python  => python::build(dir, runtime,  &name, b, trace),
            Lang::Rust    => rust::build(dir, runtime, &name, b, trace),
            Lang::Clojure => clojure::build(dir, runtime, &name, b, trace),
            Lang::Go      => todo!()
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

pub async fn build_recursive(dirty: bool, kind: Option<BuildKind>, trace: bool) -> Vec<BuildOutput> {
    let mut outs: Vec<BuildOutput> = vec![];

    let knd = match kind {
        Some(k) => k,
        None => BuildKind::Code
    };

    match knd {

        BuildKind::Code => {
            let buildables = compiler::find_buildables(&u::pwd(), true);
            for b in buildables {
                let mut out = build(&b.dir, None, Some(BuildKind::Code), trace).await;
                outs.append(&mut out);
            }
        },

        BuildKind::Layer => {
            let layers = compiler::find_layers();
            for layer in layers.clone() {
                if should_build(&layer, dirty) {
                    let mut out = build(&layer.path, Some(layer.name), Some(BuildKind::Layer), trace).await;
                    outs.append(&mut out)
                }
            }
        },

        BuildKind::Inline => {
            println!("building inline")
        },

        _ => todo!()

    }


    outs
}

pub fn clean(dir: &str) {
    let lang = compiler::guess_lang(dir);

    match lang {
        Lang::Ruby    => ruby::clean(dir),
        Lang::Python  => python::clean(dir),
        Lang::Rust    => rust::clean(dir),
        Lang::Clojure => clojure::clean(dir),
        Lang::Go      => todo!()
    }
}

pub fn clean_recursive() {
    let layers = compiler::find_layers();
    for layer in layers {
        clean(&layer.path)
    }
}

pub fn write_manifest(builds: &Vec<BuildOutput>) {
    let s = serde_json::to_string(builds).unwrap();
    kit::write_str("build.json", &s);
}

pub fn read_manifest() -> Vec<BuildOutput> {
    let s = kit::slurp("build.json");
    let builds: Vec<BuildOutput> = serde_json::from_str(&s).expect("fail");
    builds
}
