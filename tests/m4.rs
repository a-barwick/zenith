use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use zenith::{APEX_EXEC_M3_PROFILE, APEX_EXEC_M4_PROFILE, compile_project, render_artifacts};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(sources: &[(&str, &str)]) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("zenith-m4-{}-{id}", std::process::id()));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("zenith.toml"),
            "salesforce-api-version = \"65.0\"\n",
        )
        .unwrap();
        for (path, source) in sources {
            let path = root.join("src").join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn codes(project: &TempProject) -> Vec<String> {
    compile_project(project.path())
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn assert_rejected(sources: &[(&str, &str)], expected: &str) {
    let project = TempProject::new(sources);
    let compilation = compile_project(project.path());
    assert!(
        compilation
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == expected),
        "expected {expected}, found {:#?}",
        compilation.diagnostics
    );
    assert!(
        compilation.artifacts.is_empty(),
        "invalid Zenith must not emit partial Apex"
    );
}

#[test]
fn acceptance_fixture_compiles_every_m4_feature_through_typed_hir() {
    let compilation = compile_project(&fixture("examples/m4-safe-values"));
    assert!(
        compilation.diagnostics.is_empty(),
        "{:#?}",
        compilation.diagnostics
    );
    assert_eq!(compilation.class_count(), 4);
    assert_eq!(compilation.artifacts.len(), 14);

    let hir = format!("{:#?}", compilation.hir.unwrap());
    for expected in [
        "Record",
        "SealedResult",
        "Id(\n",
        "Nullable(",
        "immutable: true",
        "Match {",
        "variant_name: \"Found\"",
        "safe: true",
        "New {",
    ] {
        assert!(
            hir.contains(expected),
            "missing `{expected}` in HIR:\n{hir}"
        );
    }
}

#[test]
fn complete_m4_emission_is_golden_and_byte_deterministic() {
    let first = compile_project(&fixture("examples/m4-safe-values"));
    let second = compile_project(&fixture("examples/m4-safe-values"));
    assert_eq!(first.artifacts, second.artifacts);
    assert_eq!(
        render_artifacts(&first.artifacts),
        fs::read_to_string(fixture("tests/golden/m4-safe-values.emit")).unwrap()
    );
}

#[test]
fn generated_apex_erases_static_types_and_exposes_readable_helpers() {
    let compilation = compile_project(&fixture("examples/m4-safe-values"));
    let rendered = render_artifacts(&compilation.artifacts);
    for expected in [
        "public final Id accountId;",
        "public AccountSummary(Id accountId, OwnerContact owner)",
        "public static LookupResult Found(AccountSummary summary)",
        "public Integer ZenithGenerated_tag { get; private set; }",
        "LookupResult ZenithGenerated_match_5_358 = result;",
        "String email = summary.owner?.email ?? 'unassigned';",
    ] {
        assert!(
            rendered.contains(expected),
            "missing `{expected}`:\n{rendered}"
        );
    }
    for erased in ["Id<Account>", "Id<Contact>", "OwnerContact?"] {
        assert!(
            !rendered.contains(erased),
            "static-only type `{erased}` leaked into Apex"
        );
    }
}

#[test]
fn manifest_and_maps_cover_non_emitting_domains_and_generated_helpers() {
    let compilation = compile_project(&fixture("examples/m4-safe-values"));
    let manifest = compilation
        .artifacts
        .iter()
        .find(|artifact| artifact.path == "build.json")
        .unwrap()
        .text()
        .unwrap();
    assert!(manifest.contains("\"src/Account.zen\""));
    assert!(manifest.contains("\"src/Contact.zen\""));
    assert!(!manifest.contains("\"name\": \"Account\""));
    assert!(!manifest.contains("\"name\": \"Contact\""));

    for (path, generated_name) in [
        ("maps/LookupResult.cls.map.json", "LookupResult"),
        (
            "maps/SafeValueService.cls.map.json",
            "ZenithGenerated_match",
        ),
    ] {
        let map = compilation
            .artifacts
            .iter()
            .find(|artifact| artifact.path == path)
            .unwrap()
            .text()
            .unwrap();
        let apex_path = path
            .replace("maps/", "generated/main/default/classes/")
            .replace(".map.json", "");
        let apex = compilation
            .artifacts
            .iter()
            .find(|artifact| artifact.path == apex_path)
            .unwrap()
            .text()
            .unwrap();
        assert!(apex.contains(generated_name));
        assert!(
            map.lines()
                .filter(|line| line.contains("\"source\": ["))
                .count()
                > 8
        );
    }
}

#[test]
fn non_null_defaults_and_nullable_operations_are_checked() {
    let cases = [
        (
            "public class Main { public static String run() { String value = null; return value; } }",
            "type.incompatible-value",
        ),
        (
            "public class Main { public static String run(String? value) { return value; } }",
            "type.incompatible-value",
        ),
        (
            "public class Main { public static String run(Box? value) { return value.text; } }\n",
            "type.nullable-dereference",
        ),
        (
            "public class Main { public static String? run(Box value) { return value?.text; } }\n",
            "type.invalid-safe-navigation",
        ),
        (
            "public class Main { public static String run(String value) { return value ?? 'x'; } }",
            "type.invalid-null-coalescing",
        ),
        (
            "public class Main { public static Integer run(String? value) { return value ?? 1; } }",
            "type.invalid-null-coalescing",
        ),
        (
            "public class Main { private String value; public static void run() {} }",
            "type.uninitialized-non-null",
        ),
        (
            "public class Main { public String Value { get; private set; } }",
            "type.uninitialized-non-null",
        ),
        (
            "public class Main { public static void run() { Integer value; } }",
            "type.uninitialized-non-null",
        ),
        (
            "public class Main { public static void? run() {} }",
            "type.invalid-nullable-type",
        ),
        (
            "public class Main { private String?? value = null; }",
            "type.invalid-nullable-type",
        ),
        (
            "public class Main {
                public static String run(Boolean choose) {
                    let value = choose ? null : null;
                    return 'unreachable';
                }
            }",
            "type.cannot-infer-let",
        ),
    ];
    for (source, expected) in cases {
        let mut sources = vec![("Main.zen", source)];
        if source.contains("Box") {
            sources.push(("Box.zen", "public record Box(String text);"));
        }
        assert_rejected(&sources, expected);
    }

    let object_erasure = TempProject::new(&[(
        "Main.zen",
        "public class Main {
            public static Object nullableBoundary(Boolean choose) {
                Object value = null;
                return choose ? value : null;
            }
        }",
    )]);
    assert!(
        codes(&object_erasure).is_empty(),
        "{:#?}",
        codes(&object_erasure)
    );
}

#[test]
fn flow_narrowing_handles_both_branches_guards_and_assignment_invalidation() {
    let positive = TempProject::new(&[
        ("Box.zen", "public record Box(String text);"),
        (
            "Main.zen",
            "public class Main {
                public static String branch(Box? value, Boolean choose) {
                    if (value != null) { return value.text; }
                    return 'none';
                }
                public static String inverse(Box? value) {
                    if (value == null) { return 'none'; }
                    return value.text;
                }
                public static String compound(Box? value) {
                    if (value != null && value.text == 'ready') {
                        return value.text;
                    }
                    return 'none';
                }
                public static String conditional(Box? value) {
                    return value != null ? value.text : 'none';
                }
                public static String returningThen(Box? value) {
                    if (value == null) { return 'none'; } else { value = value; }
                    return value.text;
                }
                public static String returningElse(Box? value) {
                    if (value != null) { value = value; } else { return 'none'; }
                    return value.text;
                }
                public static Box? optional(Boolean present, Box value) {
                    return present ? value : null;
                }
            }",
        ),
    ]);
    assert!(codes(&positive).is_empty(), "{:?}", codes(&positive));

    assert_rejected(
        &[
            ("Box.zen", "public record Box(String text);"),
            (
                "Main.zen",
                "public class Main {
                    public static String run(Box? value) {
                        if (value != null) { value.text; }
                        return value.text;
                    }
                }",
            ),
        ],
        "type.nullable-dereference",
    );
    assert_rejected(
        &[
            ("Box.zen", "public record Box(String text);"),
            (
                "Main.zen",
                "public class Main {
                    public static String run(Box? value) {
                        if (value != null) {
                            value = null;
                            return value.text;
                        }
                        return 'none';
                    }
                }",
            ),
        ],
        "type.nullable-dereference",
    );
}

#[test]
fn fields_are_not_unsafely_flow_narrowed() {
    assert_rejected(
        &[
            ("Box.zen", "public record Box(String text);"),
            (
                "Main.zen",
                "public class Main {
                    private Box? value;
                    public String run() {
                        if (value != null) { return value.text; }
                        return 'none';
                    }
                }",
            ),
        ],
        "type.nullable-dereference",
    );
}

#[test]
fn let_inference_and_binding_immutability_are_enforced() {
    for source in [
        "public class Main {
            public static String run() {
                let value = 'first';
                value = 'second';
                return value;
            }
        }",
        "public class Main {
            public static String run() {
                let value = 'first';
                (value) = 'second';
                return value;
            }
        }",
        "public class Main {
            public static Integer run() {
                let value = 1;
                ++value;
                return value;
            }
        }",
        "public class Main {
            public static Integer run() {
                let value = 1;
                value++;
                return value;
            }
        }",
    ] {
        assert_rejected(&[("Main.zen", source)], "type.immutable-assignment");
    }
    for source in [
        "public class Main { public static void run() { let value = null; } }",
        "public class Main { public static void run() { let value = Main; } }",
        "public class Main {
            public static void run() { let value = consume(); }
            private static void consume() {}
        }",
    ] {
        assert_rejected(&[("Main.zen", source)], "type.cannot-infer-let");
    }
}

#[test]
fn record_construction_components_and_collisions_are_checked() {
    let cases = [
        (
            vec![
                (
                    "Pair.zen",
                    "public record Pair(String left, Integer right);",
                ),
                (
                    "Main.zen",
                    "public class Main { public static Pair run() { return new Pair('x'); } }",
                ),
            ],
            "type.not-constructible",
        ),
        (
            vec![
                (
                    "Pair.zen",
                    "public record Pair(String left, Integer right);",
                ),
                (
                    "Main.zen",
                    "public class Main { public static Pair run() { return new Pair(1, 2); } }",
                ),
            ],
            "type.incompatible-value",
        ),
        (
            vec![
                (
                    "Pair.zen",
                    "public record Pair(String left, Integer right);",
                ),
                (
                    "Main.zen",
                    "public class Main { public static void run(Pair pair) { pair.left = 'x'; } }",
                ),
            ],
            "type.invalid-assignment-target",
        ),
        (
            vec![(
                "Pair.zen",
                "public record Pair(String value, Integer VALUE);",
            )],
            "resolve.duplicate-record-component",
        ),
        (
            vec![(
                "Main.zen",
                "public class Main { public static Main run() { return new Main(); } }",
            )],
            "type.not-constructible",
        ),
        (
            vec![(
                "Pair.zen",
                "public record Pair(String ZenithGenerated_value);",
            )],
            "resolve.reserved-generated-name",
        ),
    ];
    for (sources, expected) in cases {
        assert_rejected(&sources, expected);
    }
}

#[test]
fn generated_record_and_result_members_preserve_global_accessibility() {
    let project = TempProject::new(&[
        ("GlobalValue.zen", "global record GlobalValue(String text);"),
        (
            "GlobalResult.zen",
            "global sealed result GlobalResult { case Found(GlobalValue value); case Missing; }",
        ),
    ]);
    let compilation = compile_project(project.path());
    assert!(
        compilation.diagnostics.is_empty(),
        "{:#?}",
        compilation.diagnostics
    );
    let rendered = render_artifacts(&compilation.artifacts);
    for expected in [
        "global final String text;",
        "global GlobalValue(String text)",
        "global Integer ZenithGenerated_tag",
        "global static GlobalResult Found(GlobalValue value)",
    ] {
        assert!(
            rendered.contains(expected),
            "missing `{expected}`:\n{rendered}"
        );
    }
}

#[test]
fn typed_ids_are_nominal_sobject_only_and_zero_cost() {
    let domains = [
        ("Account.zen", "public sobject Account;"),
        ("Contact.zen", "public sobject Contact;"),
    ];
    assert_rejected(
        &[
            domains[0],
            domains[1],
            (
                "Main.zen",
                "public class Main {
                    public static Id<Account> run(Id<Contact> value) { return value; }
                }",
            ),
        ],
        "type.incompatible-value",
    );
    assert_rejected(
        &[
            ("Domain.zen", "public record Domain(String value);"),
            (
                "Main.zen",
                "public class Main { public static Id<Domain> run(Id<Domain> value) { return value; } }",
            ),
        ],
        "type.invalid-id-domain",
    );
    assert_rejected(
        &[
            domains[0],
            (
                "Main.zen",
                "public class Main { public static Id<Account> run(Id value) { return value; } }",
            ),
        ],
        "type.incompatible-value",
    );
    assert_rejected(
        &[
            domains[0],
            (
                "Main.zen",
                "public class Main {
                    public static Id<Account> run(Id<Account?> value) { return value; }
                }",
            ),
        ],
        "type.invalid-id-domain",
    );
    assert_rejected(
        &[
            domains[0],
            domains[1],
            (
                "Main.zen",
                "public class Main {
                    public static Id<Account> run(Id<Account, Contact> value) { return value; }
                }",
            ),
        ],
        "type.invalid-id-domain",
    );

    let erased = TempProject::new(&[
        domains[0],
        (
            "Main.zen",
            "public class Main {
                public static Id erase(Id<Account> value) { return value; }
                public static Id<Account>? optional(Boolean present, Id<Account> value) {
                    return present ? value : null;
                }
            }",
        ),
    ]);
    assert!(codes(&erased).is_empty(), "{:?}", codes(&erased));
}

#[test]
fn sealed_results_require_valid_unique_exhaustive_matches() {
    let result = (
        "Choice.zen",
        "public sealed result Choice { case Yes(String value); case No; }",
    );
    let cases = [
        (
            "public class Main {
                public static String run(Choice value) {
                    match (value) { when Yes(text) { return text; } }
                }
            }",
            "type.non-exhaustive-match",
        ),
        (
            "public class Main {
                public static String run(Choice value) {
                    match (value) {
                        when Yes(text) { return text; }
                        when Yes(other) { return other; }
                        when No { return 'no'; }
                    }
                }
            }",
            "type.duplicate-match-arm",
        ),
        (
            "public class Main {
                public static String run(Choice value) {
                    match (value) {
                        when Maybe { return 'maybe'; }
                        when Yes(text) { return text; }
                        when No { return 'no'; }
                    }
                }
            }",
            "resolve.unknown-result-variant",
        ),
        (
            "public class Main {
                public static String run(Choice value) {
                    match (value) {
                        when Yes { return 'yes'; }
                        when No { return 'no'; }
                    }
                }
            }",
            "type.match-binding-count",
        ),
        (
            "public class Main {
                public static String run(Choice? value) {
                    match (value) {
                        when Yes(text) { return text; }
                        when No { return 'no'; }
                    }
                }
            }",
            "type.invalid-match-subject",
        ),
        (
            "public class Main {
                public static String run(Main value) {
                    match (value) {}
                    return 'none';
                }
            }",
            "type.invalid-match-subject",
        ),
    ];
    for (main, expected) in cases {
        assert_rejected(&[result, ("Main.zen", main)], expected);
    }
}

#[test]
fn sealed_result_declaration_and_arm_bindings_reject_collisions_and_mutation() {
    assert_rejected(
        &[(
            "Choice.zen",
            "public sealed result Choice { case Yes; case YES; }",
        )],
        "resolve.duplicate-result-variant",
    );
    assert_rejected(
        &[(
            "Choice.zen",
            "public sealed result Choice { case Yes(String value, Integer VALUE); }",
        )],
        "resolve.duplicate-record-component",
    );
    assert_rejected(
        &[
            (
                "Choice.zen",
                "public sealed result Choice { case Yes(String value); }",
            ),
            (
                "Main.zen",
                "public class Main {
                    public static String run(Choice choice) {
                        match (choice) {
                            when Yes(value) { value = 'changed'; return value; }
                        }
                    }
                }",
            ),
        ],
        "type.immutable-assignment",
    );
    assert_rejected(
        &[
            (
                "Choice.zen",
                "public sealed result Choice { case Yes(String left, String right); }",
            ),
            (
                "Main.zen",
                "public class Main {
                    public static String run(Choice choice) {
                        match (choice) {
                            when Yes(value, VALUE) { return value; }
                        }
                    }
                }",
            ),
        ],
        "resolve.duplicate-local",
    );
    assert_rejected(
        &[
            (
                "Choice.zen",
                "public sealed result Choice { case Yes(String value); }",
            ),
            (
                "Main.zen",
                "public class Main {
                    public static String run(Choice choice) {
                        match (choice) {
                            when Yes(ZenithGenerated_payload) {
                                return ZenithGenerated_payload;
                            }
                        }
                    }
                }",
            ),
        ],
        "resolve.reserved-generated-name",
    );
    assert_rejected(
        &[(
            "Main.zen",
            "public class Main {
                public static String run() {
                    let ZenithGenerated_match_0_0 = 'collision';
                    return ZenithGenerated_match_0_0;
                }
            }",
        )],
        "resolve.reserved-generated-name",
    );
}

#[test]
fn safe_navigation_checks_methods_as_well_as_fields() {
    let project = TempProject::new(&[
        (
            "Helper.zen",
            "public class Helper { public String label() { return 'ok'; } }",
        ),
        (
            "Main.zen",
            "public class Main {
                public static String? run(Helper? helper) {
                    return helper?.label();
                }
            }",
        ),
    ]);
    let compilation = compile_project(project.path());
    assert!(
        compilation.diagnostics.is_empty(),
        "{:#?}",
        compilation.diagnostics
    );
    assert!(render_artifacts(&compilation.artifacts).contains("return helper?.label();"));
}

#[test]
fn cli_checks_and_emits_the_m4_fixture() {
    let checked = Command::new(env!("CARGO_BIN_EXE_zenith"))
        .args(["check", "examples/m4-safe-values"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(checked.status.success(), "{checked:?}");
    assert_eq!(
        String::from_utf8(checked.stdout).unwrap(),
        "Checked 4 classes.\n"
    );

    let emitted = Command::new(env!("CARGO_BIN_EXE_zenith"))
        .args(["emit", "examples/m4-safe-values"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(emitted.status.success(), "{emitted:?}");
    assert_eq!(
        String::from_utf8(emitted.stdout).unwrap(),
        fs::read_to_string(fixture("tests/golden/m4-safe-values.emit")).unwrap()
    );
}

#[cfg(unix)]
#[test]
fn m4_build_writes_complete_output_and_capability_gates_the_m3_verifier() {
    let project = TempProject::new(&[
        ("Account.zen", "public sobject Account;"),
        (
            "Summary.zen",
            "public record Summary(Id<Account> accountId, String? label);",
        ),
        (
            "Main.zen",
            "public class Main {
                public static String label(Summary? summary) {
                    return summary?.label ?? 'none';
                }
            }",
        ),
    ]);
    let output = Command::new(env!("CARGO_BIN_EXE_zenith"))
        .arg("build")
        .arg(project.path())
        .arg("--verify-apex-exec")
        .arg("/usr/bin/true")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with(
        "Apex verification: unsupported (apex-exec, revision \
         1e4f1ca1938abfc996651ae447f227e0db680b6a, profile zenith-m4-safe-values).\n"
    ));
    assert!(stdout.contains("Built 2 classes to "));
    assert!(output.stderr.is_empty());

    let output_root = project.path().join(".zenith");
    assert!(
        output_root
            .join("generated/main/default/classes/Summary.cls")
            .is_file()
    );
    assert!(output_root.join("maps/Main.cls.map.json").is_file());
    let manifest = fs::read_to_string(output_root.join("build.json")).unwrap();
    assert!(manifest.contains("\"src/Account.zen\""));
    assert!(manifest.contains("\"outcome\": \"unsupported\""));
    assert!(manifest.contains("\"capabilityProfile\": \"zenith-m4-safe-values\""));
}

#[test]
fn verifier_profiles_follow_the_emitted_surface_not_erased_source_types() {
    let erased = TempProject::new(&[
        ("Account.zen", "public sobject Account;"),
        (
            "Main.zen",
            "public class Main {
                public static Id<Account>? copy(Id<Account>? value) {
                    let result = value;
                    return result;
                }
            }",
        ),
    ]);
    let erased = compile_project(erased.path());
    assert!(erased.diagnostics.is_empty(), "{:#?}", erased.diagnostics);
    assert_eq!(erased.apex_exec_profile(), APEX_EXEC_M3_PROFILE);

    let safe_navigation = TempProject::new(&[
        (
            "Helper.zen",
            "public class Helper { public String label() { return 'ok'; } }",
        ),
        (
            "Main.zen",
            "public class Main {
                public static String? label(Helper? helper) {
                    return helper?.label();
                }
            }",
        ),
    ]);
    let safe_navigation = compile_project(safe_navigation.path());
    assert!(
        safe_navigation.diagnostics.is_empty(),
        "{:#?}",
        safe_navigation.diagnostics
    );
    assert_eq!(safe_navigation.apex_exec_profile(), APEX_EXEC_M4_PROFILE);

    let coalescing = TempProject::new(&[(
        "Main.zen",
        "public class Main {
            public static String label(String? value) { return value ?? 'none'; }
        }",
    )]);
    let coalescing = compile_project(coalescing.path());
    assert!(
        coalescing.diagnostics.is_empty(),
        "{:#?}",
        coalescing.diagnostics
    );
    assert_eq!(coalescing.apex_exec_profile(), APEX_EXEC_M4_PROFILE);
}
