use anyhow::{
    Result,
    anyhow,
};
use kit as u;
use crate::spec::mutation::MutationSpec;
use crate::spec::mutation;
use serde_yaml::{
    Mapping,
    Value,
    value::{
        Tag,
        TaggedValue,
    },
};
use std::{
    collections::HashSet,
    fmt,
    fs::{
        File,
        canonicalize,
    },
    path::PathBuf,
};

fn load_yaml(file_path: PathBuf) -> Result<Value> {
    let file_reader = File::open(file_path).expect("Unable to open file");
    let data: Value = serde_yaml::from_reader(file_reader)?;

    Ok(data)
}

#[derive(Debug, Clone)]
pub struct Transformer {
    error_on_circular: bool,
    root_path: PathBuf,
    dir: String,
    seen_paths: HashSet<PathBuf>, // for circular reference detection
}

impl Transformer {
    pub fn new(root_path: PathBuf, strict: bool, dir: &str) -> Result<Self> {
        Self::new_node(root_path, strict, dir, None)
    }

    pub fn parse(&self) -> Value {
        let file_path = self.root_path.clone();
        let input = load_yaml(file_path).unwrap();

        self.clone().recursive_process(input)
    }

    fn new_node(
        root_path: PathBuf,
        strict: bool,
        dir: &str,
        seen_paths_option: Option<HashSet<PathBuf>>,
    ) -> Result<Self> {
        let mut seen_paths = match seen_paths_option {
            Some(set) => set,
            None => HashSet::new(),
        };

        let normalized_path = canonicalize(&root_path).unwrap();

        // Circular reference guard
        if seen_paths.contains(&normalized_path) {
            return Err(anyhow!(
                "circular reference: {}",
                &normalized_path.display()
            ));
        }

        seen_paths.insert(normalized_path);

        Ok(Transformer {
            error_on_circular: strict,
            dir: dir.to_string(),
            root_path,
            seen_paths,
        })
    }

    fn recursive_process(self, input: Value) -> Value {
        match input {
            Value::Sequence(seq) => seq
                .iter()
                .map(|v| self.clone().recursive_process(v.clone()))
                .collect(),
            Value::Mapping(map) => Value::Mapping(Mapping::from_iter(
                map.iter()
                    .map(|(k, v)| (k.clone(), self.clone().recursive_process(v.clone()))),
            )),
            Value::Tagged(tagged_value) => match tagged_value.tag.to_string().as_str() {
                "!include" => {
                    let value = tagged_value.value.as_str().unwrap();
                    let file_path = PathBuf::from(value);

                    self.handle_include_extension(file_path)
                }
                "!read" => {
                    let value = tagged_value.value.as_str().unwrap();
                    let paths: Vec<&str> = value.split(" !read ").collect();

                    let mut s: String = String::from("");
                    for path in paths {
                        let p = u::absolutize(&self.dir, &path);
                        let c = u::slurp(&p);
                        s.push_str(&c);
                    }
                    u::write_str("/tmp/tc-read-tmp.yml", &s);
                    let file_path = PathBuf::from("/tmp/tc-read-tmp.yml");
                    self.handle_include_extension(file_path)
                }
                "!mutations" => {
                    let value = tagged_value.value.as_str().unwrap();
                    let paths: Vec<&str> = value.split(" !mutations ").collect();
                    let mut specs: Vec<MutationSpec> = vec![];

                    for path in paths {
                        let data: String = u::slurp(&path);
                        let spec: MutationSpec = serde_yaml::from_str(&data).unwrap();
                        specs.push(spec);
                    }
                    let merged = mutation::merge_specs(&specs);
                    let s = serde_yaml::to_string(&merged).unwrap();
                    u::write_str("/tmp/tc-mutations.yml", &s);
                    let file_path = PathBuf::from("/tmp/tc-mutations.yml");
                    self.handle_include_extension(file_path)
                }

                _ => Value::Tagged(tagged_value),
            },
            // default no transform
            _ => input,
        }
    }

    fn handle_include_extension(&self, file_path: PathBuf) -> Value {
        let normalized_file_path = self.process_path(&file_path);

        match Transformer::new_node(
            normalized_file_path,
            self.error_on_circular,
            &self.dir,
            Some(self.seen_paths.clone()),
        ) {
            Ok(transformer) => transformer.parse(),
            Err(e) => {
                if self.error_on_circular {
                    // TODO: probably something better to do than panic ?
                    panic!("{:?}", e);
                }

                return Value::Tagged(
                    TaggedValue {
                        tag: Tag::new("circular"),
                        value: Value::String(file_path.display().to_string()),
                    }
                    .into(),
                );
            }
        }
    }

    fn process_path(&self, file_path: &PathBuf) -> PathBuf {
        if file_path.is_absolute() {
            return file_path.clone();
        }
        let joined = self.root_path.parent().unwrap().join(file_path);

        if !joined.is_file() {
            panic!("{:?} not found", joined);
        }

        canonicalize(joined).unwrap()
    }
}

impl fmt::Display for Transformer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_yaml::to_string(&self.clone().parse()).unwrap()
        )
    }
}
