//! Formal validation of tc design invariants.
//!
//! These tests encode the intended architectural contracts of the tc codebase.
//! They serve as living documentation: if a test fails, either the code has
//! drifted from the design intent (fix the code) or the intent has evolved
//! (update the test with justification).

mod tier1_struct_symmetry {
    //! Invariant: TopologySpec is symmetric to Topology.
    //!
    //! Every domain-relevant field on TopologySpec should have a corresponding
    //! field on Topology (possibly with a different type, e.g. Option<T> -> T).
    //! Fields that exist only for compilation control (filesystem discovery,
    //! recursion flags) are explicitly documented as "control-only".

    /// Canonical mapping between TopologySpec fields and Topology fields.
    /// This is the source of truth for the symmetry invariant.
    struct FieldMapping {
        spec_field: &'static str,
        topology_field: Option<&'static str>,
        relationship: FieldRelationship,
    }

    enum FieldRelationship {
        /// Spec field maps directly to Topology field (possibly Option<T> -> T)
        Direct,
        /// Spec field is used during composition but does not appear on Topology
        /// (control/discovery flags)
        ControlOnly,
        /// Spec field maps to a differently-named Topology field
        Renamed(&'static str),
        /// Spec field is transformed into a different structure on Topology
        Transformed(&'static str),
    }

    /// The authoritative field mapping. If you add a field to TopologySpec or
    /// Topology, you MUST update this table — the tests below will fail otherwise.
    const SPEC_TO_TOPOLOGY: &[FieldMapping] = &[
        // --- Direct mappings (Option<T> on spec -> T on topology) ---
        FieldMapping { spec_field: "kind", topology_field: Some("kind"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "concurrent", topology_field: Some("concurrent"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "infra", topology_field: Some("infra"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "hyphenated_names", topology_field: Some("hyphenated_names"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "dir", topology_field: Some("dir"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "events", topology_field: Some("events"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "routes", topology_field: Some("routes"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "functions", topology_field: Some("functions"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "queues", topology_field: Some("queues"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "channels", topology_field: Some("channels"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "pages", topology_field: Some("pages"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "tests", topology_field: Some("tests"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "sequences", topology_field: Some("sequences"), relationship: FieldRelationship::Direct },
        FieldMapping { spec_field: "flow", topology_field: Some("flow"), relationship: FieldRelationship::Direct },

        // --- Renamed mappings ---
        FieldMapping { spec_field: "name", topology_field: Some("namespace"), relationship: FieldRelationship::Renamed("name -> namespace") },

        // --- Transformed mappings ---
        FieldMapping { spec_field: "mutations", topology_field: Some("mutations"), relationship: FieldRelationship::Transformed("MutationSpec -> HashMap<String, Mutation>") },
        FieldMapping { spec_field: "triggers", topology_field: Some("pools"), relationship: FieldRelationship::Transformed("triggers + pools vec -> HashMap<String, Pool>") },
        FieldMapping { spec_field: "pools", topology_field: Some("pools"), relationship: FieldRelationship::Transformed("pools vec feeds into Pool map with triggers") },
        FieldMapping { spec_field: "states", topology_field: Some("flow"), relationship: FieldRelationship::Transformed("states is alternate input to flow") },

        // --- Control-only fields (used for discovery/compilation, not on Topology) ---
        FieldMapping { spec_field: "root", topology_field: None, relationship: FieldRelationship::ControlOnly },
        FieldMapping { spec_field: "recursive", topology_field: None, relationship: FieldRelationship::ControlOnly },
        FieldMapping { spec_field: "auto", topology_field: None, relationship: FieldRelationship::ControlOnly },
        FieldMapping { spec_field: "mode", topology_field: None, relationship: FieldRelationship::ControlOnly },
        FieldMapping { spec_field: "nodes", topology_field: None, relationship: FieldRelationship::ControlOnly },

        // --- Fields that exist on spec but are NOT used in topology construction ---
        // These are potential bugs or dead code:
        FieldMapping { spec_field: "version", topology_field: Some("version"), relationship: FieldRelationship::Transformed("UNUSED: spec.version is ignored; topology.version comes from semver lookup") },
        FieldMapping { spec_field: "config", topology_field: Some("config"), relationship: FieldRelationship::Transformed("UNUSED: spec.config path is ignored; topology.config is always Config::new()") },
    ];

    /// Fields on Topology that have NO counterpart on TopologySpec.
    /// These are derived/computed during composition.
    const TOPOLOGY_ONLY_FIELDS: &[&str] = &[
        "env",           // from environment/CLI context
        "fqn",           // computed from namespace + env
        "sandbox",       // derived from infra path conventions
        "version",       // from git semver, not spec.version
        "tags",          // derived from topology metadata
        "config",        // Config::new(), not spec.config
        "roles",         // derived from functions + step-function needs
        "base_roles",    // derived from functions
        "schedules",     // from infra path scanning, not spec
        "transducer",    // optional, from flow compilation
        "nodes",         // recursive child topologies from filesystem discovery
    ];

    #[test]
    fn all_spec_fields_are_mapped() {
        let spec_fields: Vec<&str> = vec![
            "name", "root", "recursive", "auto", "concurrent", "dir", "kind",
            "version", "infra", "config", "mode", "hyphenated_names", "pools",
            "nodes", "functions", "events", "routes", "mutations", "queues",
            "channels", "triggers", "pages", "tests", "states", "flow", "sequences",
        ];

        let mapped_spec_fields: Vec<&str> = SPEC_TO_TOPOLOGY
            .iter()
            .map(|m| m.spec_field)
            .collect();

        let mut unmapped = Vec::new();
        for field in &spec_fields {
            if !mapped_spec_fields.contains(field) {
                unmapped.push(*field);
            }
        }

        assert!(
            unmapped.is_empty(),
            "TopologySpec fields not accounted for in SPEC_TO_TOPOLOGY mapping: {:?}\n\
             Every spec field must be explicitly mapped (even if ControlOnly).",
            unmapped
        );
    }

    #[test]
    fn all_topology_fields_are_accounted_for() {
        let topology_fields: Vec<&str> = vec![
            "namespace", "env", "fqn", "concurrent", "kind", "infra", "dir",
            "sandbox", "hyphenated_names", "version", "nodes", "events", "routes",
            "functions", "mutations", "schedules", "queues", "channels", "pools",
            "pages", "tags", "flow", "config", "roles", "base_roles", "tests",
            "transducer", "sequences",
        ];

        let mapped_topology_fields: Vec<&str> = SPEC_TO_TOPOLOGY
            .iter()
            .filter_map(|m| m.topology_field)
            .collect();

        let mut unaccounted = Vec::new();
        for field in &topology_fields {
            if !mapped_topology_fields.contains(field)
                && !TOPOLOGY_ONLY_FIELDS.contains(field)
            {
                unaccounted.push(*field);
            }
        }

        assert!(
            unaccounted.is_empty(),
            "Topology fields not accounted for in either SPEC_TO_TOPOLOGY or TOPOLOGY_ONLY_FIELDS: {:?}\n\
             Every topology field must be documented as either mapped-from-spec or derived.",
            unaccounted
        );
    }

    #[test]
    fn no_undocumented_topology_only_fields() {
        let topology_fields: Vec<&str> = vec![
            "namespace", "env", "fqn", "concurrent", "kind", "infra", "dir",
            "sandbox", "hyphenated_names", "version", "nodes", "events", "routes",
            "functions", "mutations", "schedules", "queues", "channels", "pools",
            "pages", "tags", "flow", "config", "roles", "base_roles", "tests",
            "transducer", "sequences",
        ];

        let mut phantom_fields = Vec::new();
        for field in TOPOLOGY_ONLY_FIELDS {
            if !topology_fields.contains(field) {
                phantom_fields.push(*field);
            }
        }

        assert!(
            phantom_fields.is_empty(),
            "TOPOLOGY_ONLY_FIELDS references fields not on Topology: {:?}\n\
             Remove stale entries.",
            phantom_fields
        );
    }
}

mod tier1_inference_no_duplication {
    //! Invariant: Inference logic must not be duplicated across crates.
    //!
    //! Specifically, `infer_lang` (and similar heuristic functions) should exist
    //! in exactly one place. If compiler needs it, it should re-export from
    //! composer (or a shared location), not duplicate.

    use std::process::Command;

    #[test]
    fn infer_lang_defined_in_single_crate() {
        let output = Command::new("rg")
            .args(["--count", r"pub fn infer_lang", "lib/"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let definitions: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

        assert!(
            definitions.len() <= 1,
            "infer_lang is defined in multiple crates (violates inference locality):\n{}",
            definitions.join("\n")
        );
    }

    #[test]
    fn guess_runtime_defined_in_single_crate() {
        let output = Command::new("rg")
            .args(["--count", r"pub fn guess_runtime", "lib/"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let definitions: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

        assert!(
            definitions.len() <= 1,
            "guess_runtime is defined in multiple crates (violates inference locality):\n{}",
            definitions.join("\n")
        );
    }
}

mod tier2_filesystem_boundary {
    //! Invariant: All filesystem logic belongs in compiler (or kit as the
    //! low-level abstraction layer).
    //!
    //! Crates other than `compiler` and `kit` should not perform direct
    //! filesystem operations. They should receive data from the compiler layer.

    use std::process::Command;

    const ALLOWED_FS_CRATES: &[&str] = &["lib/compiler", "lib/kit"];

    const FS_PATTERNS: &[&str] = &[
        r"std::fs::",
        r"File::open",
        r"File::create",
        r"read_to_string",
        r"create_dir_all",
        r"fs::metadata",
        r"fs::read_dir",
        r"WalkDir",
    ];

    fn find_fs_violations() -> Vec<String> {
        let mut violations = Vec::new();

        for pattern in FS_PATTERNS {
            let output = Command::new("rg")
                .args([
                    "--no-heading",
                    "-l",
                    pattern,
                    "lib/",
                    "--glob", "*.rs",
                ])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .output()
                .expect("rg must be installed");

            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.is_empty() { continue; }
                let is_allowed = ALLOWED_FS_CRATES.iter().any(|c| line.starts_with(c));
                if !is_allowed {
                    violations.push(format!("{}: {}", line, pattern));
                }
            }
        }

        violations.sort();
        violations.dedup();
        violations
    }

    #[test]
    fn no_filesystem_operations_outside_compiler_and_kit() {
        let violations = find_fs_violations();

        if !violations.is_empty() {
            panic!(
                "Filesystem operations found outside compiler/kit ({} violations):\n\n{}\n\n\
                 Design intent: all filesystem logic belongs in compiler.\n\
                 Either move this logic to compiler, or if intentional, add to ALLOWED_FS_CRATES.",
                violations.len(),
                violations.join("\n")
            );
        }
    }
}

mod tier2_inference_boundary {
    //! Invariant: Inference can be provider-specific but must be localized
    //! to the composer.
    //!
    //! No inference heuristics (language detection, kind inference, implicit
    //! resource discovery) should exist outside `lib/composer/`.

    use std::process::Command;

    const INFERENCE_PATTERNS: &[&str] = &[
        r"fn infer_",
        r"fn guess_",
        r"fn discover_",
        r"fn find_implicit_",
        r"fn is_inferred_",
    ];

    const ALLOWED_INFERENCE_CRATES: &[&str] = &["lib/composer"];

    fn find_inference_violations() -> Vec<String> {
        let mut violations = Vec::new();

        for pattern in INFERENCE_PATTERNS {
            let output = Command::new("rg")
                .args([
                    "--no-heading",
                    "-n",
                    pattern,
                    "lib/",
                    "--glob", "*.rs",
                ])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .output()
                .expect("rg must be installed");

            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.is_empty() { continue; }
                let is_allowed = ALLOWED_INFERENCE_CRATES.iter().any(|c| line.starts_with(c));
                let is_test = line.contains("#[cfg(test)]") || line.contains("mod tests");
                if !is_allowed && !is_test {
                    violations.push(line.to_string());
                }
            }
        }

        violations.sort();
        violations.dedup();
        violations
    }

    #[test]
    fn no_inference_logic_outside_composer() {
        let violations = find_inference_violations();

        if !violations.is_empty() {
            panic!(
                "Inference logic found outside composer ({} violations):\n\n{}\n\n\
                 Design intent: inference is provider-specific but localized to composer.\n\
                 Move this logic to composer, or delegate to composer from caller.",
                violations.len(),
                violations.join("\n")
            );
        }
    }
}

mod tier3_dependency_direction {
    //! Invariant: The dependency graph must follow a strict layering:
    //!
    //!   kit (lowest) -> compiler -> composer -> resolver (highest core)
    //!
    //! No crate may depend on a crate at a higher layer than itself.
    //! Additionally, core pipeline crates should not have circular or
    //! backwards dependencies.

    use std::collections::HashMap;
    use std::process::Command;

    /// Allowed dependency directions. Key depends on values.
    /// This encodes the intended architecture.
    fn allowed_deps() -> HashMap<&'static str, Vec<&'static str>> {
        let mut m = HashMap::new();
        // kit: leaf crate, no internal deps
        m.insert("kit", vec![]);
        // compiler: only depends on kit
        m.insert("compiler", vec!["kit"]);
        // configurator: only depends on kit
        m.insert("configurator", vec!["kit"]);
        // composer: depends on compiler, configurator, kit
        m.insert("composer", vec!["kit", "compiler", "configurator"]);
        // provider: depends on kit, configurator
        m.insert("provider", vec!["kit", "configurator"]);
        // differ: depends on kit, composer, tagger
        m.insert("differ", vec!["kit", "composer", "tagger"]);
        // resolver: depends on kit, compiler, composer, differ, provider, configurator, snapshotter
        m.insert("resolver", vec!["kit", "compiler", "composer", "differ", "provider", "configurator", "snapshotter"]);
        // builder: depends on kit, compiler, composer, configurator, provider
        m.insert("builder", vec!["kit", "compiler", "composer", "configurator", "provider"]);
        // deployer: depends on kit, compiler, composer, builder, provider, configurator
        m.insert("deployer", vec!["kit", "compiler", "composer", "builder", "provider", "configurator"]);
        // tagger: depends on kit, notifier
        m.insert("tagger", vec!["kit", "notifier"]);
        // notifier: depends on kit, configurator
        m.insert("notifier", vec!["kit", "configurator"]);
        // invoker: depends on kit, provider, composer, compiler, configurator
        m.insert("invoker", vec!["kit", "provider", "composer", "compiler", "configurator"]);
        // tester: depends on kit, composer, compiler, invoker, provider
        m.insert("tester", vec!["kit", "composer", "compiler", "invoker", "provider"]);
        // snapshotter: depends on kit, provider, configurator, compiler, composer, tagger
        m.insert("snapshotter", vec!["kit", "provider", "configurator", "compiler", "composer", "tagger"]);
        // scaffolder: depends on kit, compiler, composer, visualizer, provider
        m.insert("scaffolder", vec!["kit", "compiler", "composer", "visualizer", "provider"]);
        // emulator: depends on kit, provider, composer, compiler, configurator
        m.insert("emulator", vec!["kit", "provider", "composer", "compiler", "configurator"]);
        // executor: depends on kit
        m.insert("executor", vec!["kit"]);
        // router: depends on kit, configurator, provider
        m.insert("router", vec!["kit", "configurator", "provider"]);
        // visualizer: depends on kit, compiler, composer
        m.insert("visualizer", vec!["kit", "compiler", "composer"]);
        m
    }

    fn parse_actual_deps() -> HashMap<String, Vec<String>> {
        let output = Command::new("rg")
            .args([
                "--no-heading",
                "-n",
                r#"path = "\.\./([^"]+)""#,
                "lib/",
                "--glob", "Cargo.toml",
                "--only-matching",
                "-r", "$1",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            // Format with --only-matching -r: lib/composer/Cargo.toml:NN:kit
            let parts: Vec<&str> = line.splitn(3, ':').collect();
            if parts.len() < 3 { continue; }
            let path = parts[0];
            let dep = parts[2].trim();

            if let Some(crate_name) = path
                .strip_prefix("lib/")
                .and_then(|s| s.strip_suffix("/Cargo.toml"))
            {
                deps.entry(crate_name.to_string())
                    .or_default()
                    .push(dep.to_string());
            }
        }

        deps
    }

    #[test]
    fn dependency_graph_matches_intended_architecture() {
        let allowed = allowed_deps();
        let actual = parse_actual_deps();
        let mut violations = Vec::new();

        for (crate_name, deps) in &actual {
            if let Some(allowed_list) = allowed.get(crate_name.as_str()) {
                for dep in deps {
                    if !allowed_list.contains(&dep.as_str()) {
                        violations.push(format!(
                            "{} depends on {} (not in allowed list: {:?})",
                            crate_name, dep, allowed_list
                        ));
                    }
                }
            } else {
                violations.push(format!(
                    "Crate '{}' not in allowed_deps() map — add it with its intended dependencies",
                    crate_name
                ));
            }
        }

        if !violations.is_empty() {
            panic!(
                "Dependency direction violations ({}):\n\n{}\n\n\
                 Update allowed_deps() if this is intentional architectural change.",
                violations.len(),
                violations.join("\n")
            );
        }
    }

    #[test]
    fn core_pipeline_has_no_backwards_deps() {
        let actual = parse_actual_deps();

        // The core pipeline flows: compiler -> composer -> resolver
        // None of these should depend on crates "above" them in the pipeline.
        let backwards_rules: Vec<(&str, &[&str])> = vec![
            ("compiler", &["composer", "resolver", "builder", "deployer", "differ"]),
            ("composer", &["resolver", "builder", "deployer"]),
        ];

        let mut violations = Vec::new();
        for (crate_name, forbidden) in &backwards_rules {
            if let Some(deps) = actual.get(*crate_name) {
                for dep in deps {
                    if forbidden.contains(&dep.as_str()) {
                        violations.push(format!(
                            "{} depends on {} (backwards dependency in core pipeline)",
                            crate_name, dep
                        ));
                    }
                }
            }
        }

        assert!(
            violations.is_empty(),
            "Backwards dependencies in core pipeline:\n\n{}\n\n\
             The pipeline must flow: kit -> compiler -> composer -> resolver",
            violations.join("\n")
        );
    }
}
