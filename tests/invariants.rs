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

    /// Spec fields that map directly to a Topology field (possibly Option<T> -> T).
    /// Format: (spec_field, topology_field)
    const DIRECT_MAPPINGS: &[(&str, &str)] = &[
        ("kind", "kind"),
        ("concurrent", "concurrent"),
        ("infra", "infra"),
        ("hyphenated_names", "hyphenated_names"),
        ("dir", "dir"),
        ("events", "events"),
        ("routes", "routes"),
        ("functions", "functions"),
        ("queues", "queues"),
        ("channels", "channels"),
        ("pages", "pages"),
        ("tests", "tests"),
        ("sequences", "sequences"),
        ("flow", "flow"),
    ];

    /// Spec fields that map to a differently-named or transformed Topology field.
    /// Format: (spec_field, topology_field, description)
    const TRANSFORMED_MAPPINGS: &[(&str, &str, &str)] = &[
        ("name", "namespace", "renamed: name -> namespace"),
        ("mutations", "mutations", "MutationSpec -> HashMap<String, Mutation>"),
        ("triggers", "pools", "triggers + pools vec -> HashMap<String, Pool>"),
        ("pools", "pools", "pools vec feeds into Pool map with triggers"),
        ("states", "flow", "states is alternate input to flow"),
        ("version", "version", "UNUSED: spec.version ignored; topology.version from semver"),
        ("config", "config", "UNUSED: spec.config ignored; topology.config is Config::new()"),
    ];

    /// Spec fields used only for compilation control (not on Topology).
    const CONTROL_ONLY_FIELDS: &[&str] = &[
        "root", "recursive", "auto", "mode", "nodes",
    ];

    /// Topology fields with NO counterpart on TopologySpec (derived during composition).
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

    fn all_mapped_spec_fields() -> Vec<&'static str> {
        let mut fields: Vec<&str> = Vec::new();
        fields.extend(DIRECT_MAPPINGS.iter().map(|(s, _)| *s));
        fields.extend(TRANSFORMED_MAPPINGS.iter().map(|(s, _, _)| *s));
        fields.extend(CONTROL_ONLY_FIELDS.iter().copied());
        fields
    }

    fn all_mapped_topology_fields() -> Vec<&'static str> {
        let mut fields: Vec<&str> = Vec::new();
        fields.extend(DIRECT_MAPPINGS.iter().map(|(_, t)| *t));
        fields.extend(TRANSFORMED_MAPPINGS.iter().map(|(_, t, _)| *t));
        fields
    }

    #[test]
    fn all_spec_fields_are_mapped() {
        let spec_fields: Vec<&str> = vec![
            "name", "root", "recursive", "auto", "concurrent", "dir", "kind",
            "version", "infra", "config", "mode", "hyphenated_names", "pools",
            "nodes", "functions", "events", "routes", "mutations", "queues",
            "channels", "triggers", "pages", "tests", "states", "flow", "sequences",
        ];

        let mapped = all_mapped_spec_fields();
        let unmapped: Vec<&&str> = spec_fields.iter()
            .filter(|f| !mapped.contains(*f))
            .collect();

        assert!(
            unmapped.is_empty(),
            "TopologySpec fields not accounted for in mapping: {:?}\n\
             Every spec field must be explicitly mapped (even if control-only).",
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

        let mapped = all_mapped_topology_fields();
        let unaccounted: Vec<&&str> = topology_fields.iter()
            .filter(|f| !mapped.contains(*f) && !TOPOLOGY_ONLY_FIELDS.contains(*f))
            .collect();

        assert!(
            unaccounted.is_empty(),
            "Topology fields not accounted for in either mappings or TOPOLOGY_ONLY_FIELDS: {:?}\n\
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

        let phantom: Vec<&&str> = TOPOLOGY_ONLY_FIELDS.iter()
            .filter(|f| !topology_fields.contains(*f))
            .collect();

        assert!(
            phantom.is_empty(),
            "TOPOLOGY_ONLY_FIELDS references fields not on Topology: {:?}\n\
             Remove stale entries.",
            phantom
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

mod tier1_composition_matrix {
    //! Invariant: Not all entities are composable with each other.
    //!
    //! The Entity Composition Matrix (https://tc-functors.org/reference/composition/)
    //! defines which entity types can target which other entity types. This is the
    //! source of truth for valid compositions. Provider-specific extensions must be
    //! explicitly declared.
    //!
    //! The matrix is encoded here and validated against the actual spec structs to
    //! ensure they don't offer composition paths that the design forbids.

    use std::collections::HashMap;

    /// Entity types that participate in composition.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Entity {
        Function,
        Event,
        Route,
        Queue,
        Channel,
        Mutation,
        Page,
    }

    /// The canonical Entity Composition Matrix.
    /// Key = source entity, Value = entities it is allowed to target.
    ///
    /// Reference: https://tc-functors.org/reference/composition/#entity-composition-matrix
    ///
    /// NOTE: "State" (Step Functions / Flow) is intentionally excluded from this
    /// matrix — it is an orchestration mechanism, not a composable entity in the
    /// same sense. Events and Routes can implicitly fall back to State targets
    /// when no explicit target is specified; that is handled by the orchestrator
    /// invariants separately.
    fn composition_matrix() -> HashMap<Entity, Vec<Entity>> {
        let mut m = HashMap::new();
        m.insert(Entity::Function, vec![Entity::Function, Entity::Event, Entity::Queue]);
        m.insert(Entity::Event,    vec![Entity::Function, Entity::Event, Entity::Queue]);
        m.insert(Entity::Route,    vec![Entity::Function, Entity::Event]);
        m.insert(Entity::Queue,    vec![Entity::Function]);
        m.insert(Entity::Channel,  vec![Entity::Function]);
        m.insert(Entity::Mutation, vec![Entity::Function, Entity::Event]);
        m.insert(Entity::Page,     vec![Entity::Function]);
        m
    }

    /// What the spec structs actually allow (based on their fields).
    /// Each entry is (source entity, field name, target entity it references).
    const SPEC_STRUCT_COMPOSITIONS: &[(Entity, &str, Entity)] = &[
        // EventSpec targets
        (Entity::Event, "function", Entity::Function),
        (Entity::Event, "functions", Entity::Function),
        (Entity::Event, "mutation", Entity::Mutation),
        (Entity::Event, "channel", Entity::Channel),
        // Event -> state is orchestration, not composition (excluded from matrix)

        // RouteSpec targets
        (Entity::Route, "function", Entity::Function),
        (Entity::Route, "event", Entity::Event),
        (Entity::Route, "queue", Entity::Queue),
        // Route -> state is orchestration fallback

        // QueueSpec targets
        (Entity::Queue, "function", Entity::Function),

        // ChannelSpec targets
        (Entity::Channel, "function", Entity::Function),
        // HandlerSpec also has function + event, but handler is Channel-internal

        // MutationSpec/ResolverSpec targets
        (Entity::Mutation, "function", Entity::Function),
        (Entity::Mutation, "event", Entity::Event),

        // InlineFunctionSpec targets (Function composing with others)
        (Entity::Function, "function", Entity::Function),
        (Entity::Function, "event", Entity::Event),
        (Entity::Function, "queue", Entity::Queue),
        (Entity::Function, "mutation", Entity::Mutation),
        (Entity::Function, "channel", Entity::Channel),

        // PageSpec targets: none (pages are static assets, no entity references)
    ];

    #[test]
    fn spec_structs_only_allow_compositions_in_matrix() {
        let matrix = composition_matrix();
        let mut violations = Vec::new();

        for (source, field, target) in SPEC_STRUCT_COMPOSITIONS {
            if let Some(allowed) = matrix.get(source) {
                if !allowed.contains(target) {
                    violations.push(format!(
                        "{:?} spec has field '{}' targeting {:?}, but matrix does not allow {:?} -> {:?}",
                        source, field, target, source, target
                    ));
                }
            }
        }

        if !violations.is_empty() {
            panic!(
                "Spec structs allow compositions not in the Entity Composition Matrix ({} violations):\n\n{}\n\n\
                 Either update the matrix (if this is a valid new composition) or \
                 remove the field from the spec struct.",
                violations.len(),
                violations.join("\n")
            );
        }
    }

    #[test]
    fn matrix_is_complete_for_all_spec_compositions() {
        let matrix = composition_matrix();

        // Every entity that appears as a source in SPEC_STRUCT_COMPOSITIONS
        // must have an entry in the matrix
        let mut missing_sources = Vec::new();
        for (source, _, _) in SPEC_STRUCT_COMPOSITIONS {
            if !matrix.contains_key(source) {
                missing_sources.push(format!("{:?}", source));
            }
        }

        missing_sources.sort();
        missing_sources.dedup();

        assert!(
            missing_sources.is_empty(),
            "Entities used in spec composition but missing from matrix: {:?}",
            missing_sources
        );
    }

    #[test]
    fn function_make_targets_matches_matrix() {
        // InlineFunctionSpec::make_targets() produces TargetSpecs for:
        // function, mutation, event, channel (but NOT queue, even though
        // InlineFunctionSpec has a queue field).
        //
        // This test documents that discrepancy. The queue field on
        // InlineFunctionSpec is used for step-function wiring, not for
        // transducer targets.
        let matrix = composition_matrix();
        let function_allowed = matrix.get(&Entity::Function).unwrap();

        // What make_targets actually promotes to TargetSpec
        let make_targets_entities = vec![
            Entity::Function,
            Entity::Mutation,
            Entity::Event,
            Entity::Channel,
        ];

        let mut out_of_matrix = Vec::new();
        for entity in &make_targets_entities {
            if !function_allowed.contains(entity) {
                out_of_matrix.push(format!("{:?}", entity));
            }
        }

        if !out_of_matrix.is_empty() {
            panic!(
                "InlineFunctionSpec::make_targets() promotes targets not in the composition matrix:\n\
                 Targets: {:?}\n\
                 Matrix allows Function -> {:?}\n\n\
                 Either the matrix needs updating or make_targets is too permissive.",
                out_of_matrix, function_allowed
            );
        }
    }

    #[test]
    fn event_spec_targets_match_matrix() {
        // EventSpec can target: function(s), mutation, channel, state
        // The matrix says Event -> [Function, Event, Queue]
        //
        // This means EventSpec allows compositions OUTSIDE the published matrix:
        //   - Event -> Mutation (not in matrix)
        //   - Event -> Channel (not in matrix)
        //
        // This test documents these extensions. If they are intentional
        // provider-specific extensions, they should be declared as such.
        let matrix = composition_matrix();
        let event_allowed = matrix.get(&Entity::Event).unwrap();

        let event_actual_targets = vec![
            ("function", Entity::Function),
            ("mutation", Entity::Mutation),
            ("channel", Entity::Channel),
        ];

        let mut extensions = Vec::new();
        for (field, entity) in &event_actual_targets {
            if !event_allowed.contains(entity) {
                extensions.push(format!("EventSpec.{} -> {:?}", field, entity));
            }
        }

        // This test FAILS to document that these exist as known extensions.
        // When the team decides these are valid, move them into the matrix
        // or into a PROVIDER_EXTENSIONS list.
        assert!(
            extensions.is_empty(),
            "EventSpec allows compositions outside the published matrix:\n  {}\n\n\
             These may be valid provider-specific extensions. If so, add them to \
             the matrix or declare them as provider extensions.",
            extensions.join("\n  ")
        );
    }

    #[test]
    fn route_spec_targets_match_matrix() {
        // RouteSpec can target: function, event, queue, state (fallback)
        // The published matrix says Route -> [Function, Event]
        //
        // Route -> Queue is present in the code but NOT in the published matrix.
        let matrix = composition_matrix();
        let route_allowed = matrix.get(&Entity::Route).unwrap();

        let route_actual_targets = vec![
            ("function", Entity::Function),
            ("event", Entity::Event),
            ("queue", Entity::Queue),
        ];

        let mut extensions = Vec::new();
        for (field, entity) in &route_actual_targets {
            if !route_allowed.contains(entity) {
                extensions.push(format!("RouteSpec.{} -> {:?}", field, entity));
            }
        }

        assert!(
            extensions.is_empty(),
            "RouteSpec allows compositions outside the published matrix:\n  {}\n\n\
             These may be valid extensions. If so, update the matrix at \
             https://tc-functors.org/reference/composition/",
            extensions.join("\n  ")
        );
    }
}

mod tier1_orchestrator_separation {
    //! Invariant: tc provides two kinds of orchestrators:
    //!   1. Flow (AWS Step Functions) — driven by TopologySpec.flow/states/auto
    //!   2. Transducer (inbuilt) — driven by Function targets
    //!
    //! These are fundamentally different mechanisms and must remain cleanly
    //! separated. Their types, construction paths, and semantics should not
    //! leak into each other.

    use std::process::Command;

    #[test]
    fn flow_and_transducer_are_separate_modules() {
        // Flow logic should live in composer/src/aws/flow*
        // Transducer logic should live in composer/src/aws/transducer*
        // Neither should import from the other.
        let output = Command::new("rg")
            .args([
                "--no-heading",
                "-n",
                r"use.*transducer",
                "lib/composer/src/aws/flow.rs",
                "lib/composer/src/aws/flow/",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let flow_imports_transducer: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .collect();

        let output2 = Command::new("rg")
            .args([
                "--no-heading",
                "-n",
                r"use.*(flow|sfn)",
                "lib/composer/src/aws/transducer.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout2 = String::from_utf8_lossy(&output2.stdout);
        let transducer_imports_flow: Vec<&str> = stdout2.lines()
            .filter(|l| !l.is_empty())
            .collect();

        let mut violations = Vec::new();
        if !flow_imports_transducer.is_empty() {
            violations.push(format!(
                "Flow module imports from transducer:\n  {}",
                flow_imports_transducer.join("\n  ")
            ));
        }
        if !transducer_imports_flow.is_empty() {
            violations.push(format!(
                "Transducer module imports from flow:\n  {}",
                transducer_imports_flow.join("\n  ")
            ));
        }

        assert!(
            violations.is_empty(),
            "Orchestrator types must remain independent:\n\n{}\n\n\
             Flow (Step Functions) and Transducer are separate mechanisms. \
             Shared logic should be extracted to a common module, not cross-imported.",
            violations.join("\n")
        );
    }

    #[test]
    fn topology_kind_step_function_implies_flow() {
        // When TopologyKind is StepFunction, it should be because flow is present.
        // The find_kind() function infers StepFunction when flow.is_some().
        // This test verifies that relationship is documented and consistent:
        // find_kind should not return StepFunction based on transducer presence.

        let output = Command::new("rg")
            .args([
                "--no-heading",
                "-n",
                "transducer",
                "lib/composer/src/topology.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let find_kind_section: Vec<&str> = stdout.lines()
            .filter(|l| l.contains("find_kind") || {
                // Check lines near find_kind (613-635 range)
                if let Some(num_str) = l.split(':').nth(1) {
                    if let Ok(num) = num_str.parse::<u32>() {
                        return num >= 613 && num <= 640;
                    }
                }
                false
            })
            .collect();

        // If transducer appears in the find_kind function, that's a violation
        assert!(
            find_kind_section.is_empty(),
            "find_kind() references transducer — TopologyKind should be determined \
             by Flow presence, not Transducer:\n  {}",
            find_kind_section.join("\n  ")
        );
    }

    #[test]
    fn transducer_does_not_depend_on_topology_kind() {
        // Transducer::new() should not check or depend on TopologyKind.
        // It is purely derived from function targets, independent of whether
        // the topology is a step-function, evented, routed, etc.
        let output = Command::new("rg")
            .args([
                "--no-heading",
                "-n",
                "TopologyKind",
                "lib/composer/src/aws/transducer.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let references: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .collect();

        assert!(
            references.is_empty(),
            "Transducer module references TopologyKind (should be independent):\n  {}\n\n\
             Transducer is derived from function targets only, not topology kind.",
            references.join("\n  ")
        );
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

mod tier2_coding_style {
    //! Coding style invariants for tc (from icy's 11 points + earlier 3).
    //!
    //! 1. Avoid unwrap()
    //! 2. Avoid stack allocation (&str) in map/sequence fns (not easily lintable)
    //! 3. Avoid clone()
    //! 4. Run cargo fmt before pushing
    //! 5. Don't over-comment; don't comment function signatures
    //! 6. Top-level functions shouldn't return Result
    //! 7. Generic utils belong in lib/kit
    //! 8. Avoid Rust generics
    //! 9. Avoid closures stored in variables (trait-bound closures in iterators OK)
    //! 10. Use destructuring (not easily lintable)
    //! 11. Pass &str not String in fn params (unless async/mpsc)
    //!
    //! Plus the earlier 3:
    //! - Avoid explicit lifetimes unless borrow checker requires them
    //! - Avoid custom error types
    //! - Avoid custom trait definitions

    use std::process::Command;

    // ── Point 1: Avoid unwrap() ──────────────────────────────────────────────

    #[test]
    fn unwrap_usage_does_not_increase() {
        // Baseline: 609 unwrap() calls as of 2026-05-07.
        // This should only go DOWN over time, never up.
        let output = Command::new("rg")
            .args(["--count", r"\.unwrap\(\)", "lib/", "--glob", "*.rs"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let total: usize = stdout.lines()
            .filter_map(|l| l.rsplit(':').next())
            .filter_map(|n| n.parse::<usize>().ok())
            .sum();

        let baseline = 609;
        assert!(
            total <= baseline,
            "unwrap() count increased from {} to {}.\n\
             Design rule: avoid unwrap() like a plague. Use if-let, match, \
             unwrap_or, or propagate with ?.",
            baseline, total
        );
    }

    // ── Point 3: Avoid clone() ───────────────────────────────────────────────

    #[test]
    fn clone_usage_does_not_increase() {
        // Baseline: 624 clone() calls as of 2026-05-07.
        // This should only go DOWN over time, never up.
        let output = Command::new("rg")
            .args(["--count", r"\.clone\(\)", "lib/", "--glob", "*.rs"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let total: usize = stdout.lines()
            .filter_map(|l| l.rsplit(':').next())
            .filter_map(|n| n.parse::<usize>().ok())
            .sum();

        let baseline = 624;
        assert!(
            total <= baseline,
            "clone() count increased from {} to {}.\n\
             Design rule: avoid clone(). Prefer passing references or \
             restructuring to avoid the need for cloning.",
            baseline, total
        );
    }

    // ── Point 4: Run cargo fmt ───────────────────────────────────────────────

    #[test]
    fn code_is_formatted() {
        // Uses nightly fmt (cargo +nightly fmt) which matches the project's
        // rustfmt.toml settings. Falls back to stable if nightly unavailable.
        let output = Command::new("cargo")
            .args(["+nightly", "fmt", "--check"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => {
                // Nightly not available, try stable
                Command::new("cargo")
                    .args(["fmt", "--check"])
                    .current_dir(env!("CARGO_MANIFEST_DIR"))
                    .output()
                    .expect("cargo fmt must be available")
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let diffs: Vec<&str> = stdout.lines()
            .filter(|l| l.starts_with("Diff in"))
            .collect();

        assert!(
            diffs.is_empty(),
            "Code is not formatted ({} files have diffs). Run `make fmt`:\n\n{}",
            diffs.len(),
            diffs.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
        );
    }

    // ── Point 6: Top-level functions shouldn't return Result ─────────────────

    #[test]
    fn no_result_returns_in_top_level_api() {
        // Top-level pub functions (in lib/*/src/lib.rs) should not return
        // Result types, as it adds unnecessary unwrapping in callers.
        let output = Command::new("rg")
            .args([
                "--no-heading", "-n",
                r"pub (async )?fn \w+.*-> .*Result",
                "lib/", "--glob", "*/src/lib.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let violations: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .collect();

        assert!(
            violations.is_empty(),
            "Top-level functions return Result ({} violations):\n\n{}\n\n\
             Design rule: top-level functions shouldn't return Result types.\n\
             Handle errors internally or use simpler return types.",
            violations.len(),
            violations.join("\n")
        );
    }

    // ── Point 8: Avoid Rust generics ─────────────────────────────────────────

    #[test]
    fn no_unnecessary_generics() {
        // Generic type parameters should be avoided. Exceptions:
        //   - lib/kit: utility functions (print_table<T>, json helpers) are OK
        //   - lisp/expr.rs: serialize/deserialize wrappers
        //   - Trait impl blocks (Hash, From, etc.)
        let output = Command::new("rg")
            .args([
                "--no-heading", "-n",
                r"(pub fn|fn) \w+<\w",
                "lib/", "--glob", "*.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);

        let allowed = &[
            "kit/",           // utility crate, generics are expected
            "lisp/parser.rs", // nom combinators require lifetime generics
            "lisp/expr.rs",   // serialize/deserialize helpers
        ];

        let violations: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .filter(|l| !allowed.iter().any(|a| l.contains(a)))
            .filter(|l| !l.contains("fn from(") && !l.contains("fn hash("))
            .collect();

        assert!(
            violations.is_empty(),
            "Generic type parameters found outside kit ({} violations):\n\n{}\n\n\
             Design rule: avoid Rust generics. Simple functions are easier to read.\n\
             If needed for utility functions, put them in lib/kit.",
            violations.len(),
            violations.join("\n")
        );
    }

    // ── Point 9: Avoid closures stored in variables ──────────────────────────

    #[test]
    fn no_stored_closures() {
        // Closures passed to iterators (.map(|x| ...), .filter(|x| ...)) are fine.
        // Closures stored in variables (let f = |x| ...) should be avoided;
        // use named functions instead.
        let output = Command::new("rg")
            .args([
                "--no-heading", "-n",
                r"let \w+ = \|",
                "lib/", "--glob", "*.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);

        let allowed = &[
            "lisp",  // lisp interpreter uses closures as values by design
        ];

        let violations: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .filter(|l| !allowed.iter().any(|a| l.contains(a)))
            .collect();

        assert!(
            violations.is_empty(),
            "Stored closures found ({} violations):\n\n{}\n\n\
             Design rule: avoid closures stored in variables. Use named functions \
             instead. Closures passed inline to iterators are fine.",
            violations.len(),
            violations.join("\n")
        );
    }

    // ── Point 11: Pass &str not String in fn params ──────────────────────────

    #[test]
    fn prefer_str_ref_over_owned_string_params() {
        // Function parameters should take &str, not String, unless the function
        // needs ownership (async tasks, mpsc channels, etc.).
        let output = Command::new("rg")
            .args([
                "--no-heading", "-n",
                r"fn \w+\(.*\w: String",
                "lib/", "--glob", "*.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);

        let allowed = &[
            "async fn",      // async functions may need owned data
            "fn from(",      // From trait impls take owned values
            "lisp",          // lisp interpreter manipulates owned strings
        ];

        let violations: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .filter(|l| !allowed.iter().any(|a| l.contains(a)))
            .collect();

        assert!(
            violations.is_empty(),
            "Functions taking owned String parameters ({} violations):\n\n{}\n\n\
             Design rule: pass &str instead of String in function parameters \
             unless it is an async/mpsc thread that needs ownership.",
            violations.len(),
            violations.join("\n")
        );
    }

    // ── Earlier point: Avoid custom error types ──────────────────────────────

    #[test]
    fn no_custom_error_types() {
        let output = Command::new("rg")
            .args([
                "--no-heading", "-n",
                r"(enum|struct)\s+\w*Error",
                "lib/", "--glob", "*.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);

        let allowed = &[
            "ParseError",  // entity.rs — simple unit struct, grandfathered
        ];

        let violations: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .filter(|l| !allowed.iter().any(|a| l.contains(a)))
            .filter(|l| !l.contains("#[cfg(test)]"))
            .collect();

        assert!(
            violations.is_empty(),
            "Custom error types found ({} violations):\n\n{}\n\n\
             Design preference: avoid wrapping errors in custom types.\n\
             Use Result<T, String>, anyhow::Result, or simple error values.",
            violations.len(),
            violations.join("\n")
        );
    }

    // ── Earlier point: Avoid custom traits ───────────────────────────────────

    #[test]
    fn no_custom_trait_definitions() {
        let output = Command::new("rg")
            .args([
                "--no-heading", "-n",
                r"^pub trait |^trait ",
                "lib/", "--glob", "*.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let violations: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .filter(|l| !l.contains("#[cfg(test)]"))
            .collect();

        assert!(
            violations.is_empty(),
            "Custom trait definitions found ({} violations):\n\n{}\n\n\
             Design preference: use plain functions instead of traits.\n\
             Built-in trait impls (Display, From, Serialize, etc.) are fine.",
            violations.len(),
            violations.join("\n")
        );
    }

    // ── Earlier point: Avoid lifetimes ───────────────────────────────────────

    #[test]
    fn no_unnecessary_lifetime_annotations() {
        let output = Command::new("rg")
            .args([
                "--no-heading", "-n",
                r"(struct|enum|fn)\s+\w+<'",
                "lib/", "--glob", "*.rs",
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("rg must be installed");

        let stdout = String::from_utf8_lossy(&output.stdout);

        let allowed_files = &[
            "lisp/parser.rs",     // nom requires lifetime params
            "differ/src/lib.rs",  // borrows from topology tree
            "kit/src/core.rs",    // ties output lifetime to input
        ];

        let violations: Vec<&str> = stdout.lines()
            .filter(|l| !l.is_empty())
            .filter(|l| !l.contains("#[cfg(test)]"))
            .filter(|l| !allowed_files.iter().any(|f| l.contains(f)))
            .collect();

        if !violations.is_empty() {
            panic!(
                "Explicit lifetime annotations found ({} instances):\n\n{}\n\n\
                 Design preference: avoid lifetimes unless the borrow checker requires them.\n\
                 Prefer owned types (String over &str, Vec over &[T]) in struct fields.\n\
                 If the borrow checker requires it, add to the allowed_files list.",
                violations.len(),
                violations.join("\n")
            );
        }
    }
}
