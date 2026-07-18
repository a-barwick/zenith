use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use zenith::{
    VerificationOutcome, VerificationResult, compile_project, record_verification,
    render_artifacts, write_artifacts,
};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(config: &str, sources: &[(&str, &str)], boundary: Option<&str>) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("zenith-m3-{}-{id}", std::process::id()));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("zenith.toml"), config).unwrap();
        for (path, text) in sources {
            let path = root.join("src").join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        if let Some(boundary) = boundary {
            fs::write(root.join("apex-boundary.api"), boundary).unwrap();
        }
        Self { root }
    }

    fn standard(sources: &[(&str, &str)]) -> Self {
        Self::new("salesforce-api-version = \"65.0\"\n", sources, None)
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

#[test]
fn acceptance_project_selects_cross_file_collection_and_boundary_calls() {
    let compilation = compile_project(&fixture("examples/m3-service"));
    assert!(
        compilation.diagnostics.is_empty(),
        "{:#?}",
        compilation.diagnostics
    );
    assert_eq!(compilation.class_count(), 2);
    assert_eq!(compilation.artifacts.len(), 8);

    let hir = format!("{:#?}", compilation.hir.unwrap());
    assert!(hir.contains("CallTarget::Collection") || hir.contains("Collection {"));
    assert!(hir.contains("External {"));
    assert!(hir.contains("effects_unknown: true"));
    assert!(hir.contains("Project {"));
    assert!(hir.contains("owner: \"MessageFormatter\""));
}

#[test]
fn complete_emission_matches_the_m3_golden_and_is_byte_deterministic() {
    let first = compile_project(&fixture("examples/m3-service"));
    let second = compile_project(&fixture("examples/m3-service"));
    assert_eq!(first.artifacts, second.artifacts);
    assert_eq!(
        render_artifacts(&first.artifacts),
        fs::read_to_string(fixture("tests/golden/m3-service.emit")).unwrap()
    );
}

#[test]
fn build_writes_the_complete_tree_and_removes_stale_generated_files() {
    let project = TempProject::standard(&[(
        "Main.zen",
        "public class Main { public static String value() { return 'ok'; } }",
    )]);
    let compilation = compile_project(project.path());
    assert!(
        compilation.diagnostics.is_empty(),
        "{:#?}",
        compilation.diagnostics
    );
    let output = project.path().join(".zenith");
    fs::create_dir_all(output.join("generated")).unwrap();
    fs::write(output.join("generated/stale.cls"), "stale").unwrap();

    write_artifacts(&output, &compilation.artifacts).unwrap();

    assert!(!output.join("generated/stale.cls").exists());
    for artifact in &compilation.artifacts {
        assert_eq!(
            fs::read(output.join(&artifact.path)).unwrap(),
            artifact.bytes
        );
    }
}

#[test]
fn source_maps_have_sorted_non_overlapping_generated_ranges() {
    let compilation = compile_project(&fixture("examples/m3-service"));
    for artifact in compilation
        .artifacts
        .iter()
        .filter(|artifact| artifact.path.ends_with(".map.json"))
    {
        let text = artifact.text().unwrap();
        assert!(text.starts_with("{\n  \"version\": 1,\n"));
        let mut previous_end = 0;
        let mut count = 0;
        for line in text
            .lines()
            .filter(|line| line.contains("\"generated\": ["))
        {
            let range = line
                .split("\"generated\": [")
                .nth(1)
                .unwrap()
                .split(']')
                .next()
                .unwrap();
            let mut values = range
                .split(',')
                .map(|value| value.trim().parse::<usize>().unwrap());
            let start = values.next().unwrap();
            let end = values.next().unwrap();
            assert!(start >= previous_end);
            assert!(end > start);
            previous_end = end;
            count += 1;
        }
        assert!(count > 5, "{}", artifact.path);
    }
}

#[test]
fn resolution_is_case_insensitive_but_emission_preserves_declaration_spelling() {
    let project = TempProject::standard(&[
        (
            "Helper.zen",
            "public class Helper { public static String Value() { return 'ok'; } }",
        ),
        (
            "Main.zen",
            "public class Main { public static String run() { return hELPer.vALue(); } }",
        ),
    ]);
    let compilation = compile_project(project.path());
    assert!(compilation.diagnostics.is_empty());
    let apex = compilation
        .artifacts
        .iter()
        .find(|artifact| artifact.path.ends_with("Main.cls"))
        .unwrap()
        .text()
        .unwrap();
    assert!(apex.contains("return Helper.Value();"));
}

#[test]
fn complete_checked_surface_lowers_and_emits_readable_apex() {
    let project = TempProject::standard(&[(
        "Surface.zen",
        r#"
public class Surface {
    private Long longValue;
    private Decimal decimalValue;
    private Double doubleValue;
    private Object objectValue;
    private String backingText;
    public String State { get; private set; }

    public static Integer exercise(
        List<Integer> values,
        Set<String> labels,
        Map<String, Integer> totals,
        Boolean enabled,
        Integer threshold
    ) {
        Integer total = 0;
        Integer spare;
        String mode = enabled ? 'enabled' : 'disabled';
        Boolean contained = labels.contains(mode);
        Boolean inserted = labels.add(mode);
        values.add(total);
        spare = +total;
        while (total < threshold && !values.isEmpty()) {
            total += values[0];
            if (total > 100) {
                break;
            } else {
                total = total + 1;
            }
        }
        for (Integer index = 0; index < threshold; index += 1) {
            if (index == 2) {
                continue;
            }
            total = total + index % 2;
        }
        for (String label : labels) {
            if (labels.contains(label)) {
                totals.put(label, total);
            }
        }
        {
            Integer nested = totals.size();
            total += nested;
        }
        ;
        Integer selected = totals.get(mode);
        Boolean empty = totals.isEmpty() || labels.isEmpty();
        return totals.containsKey(mode) || contained || inserted || empty
            ? selected
            : spare;
    }

    public static Integer minimum() {
        return -2147483648;
    }

    public String setState(String value) {
        this.backingText = value;
        State = value;
        return this.backingText;
    }

    public static Long keepLong(Long value) { return value; }
    public static Decimal keepDecimal(Decimal value) { return value; }
    public static Double keepDouble(Double value) { return value; }
    public static Object keepObject(Object value) { return value; }
    public static void touch(List<Integer> values) {
        values.add(1);
        return;
    }
}
"#,
    )]);
    let compilation = compile_project(project.path());
    assert!(
        compilation.diagnostics.is_empty(),
        "{:#?}",
        compilation.diagnostics
    );
    let apex = compilation
        .artifacts
        .iter()
        .find(|artifact| artifact.path.ends_with("Surface.cls"))
        .unwrap()
        .text()
        .unwrap();
    for expected in [
        "while (total < threshold && !values.isEmpty())",
        "for (Integer index = 0; index < threshold; index += 1)",
        "for (String label : labels)",
        "totals.put(label, total);",
        "return -2147483648;",
        "this.backingText = value;",
        "values.add(1);",
    ] {
        assert!(apex.contains(expected), "missing `{expected}`:\n{apex}");
    }
}

#[test]
fn project_configuration_and_discovery_fail_with_stable_diagnostics() {
    let cases = [
        ("source-root = \"src\"\n", "project.missing-api-version"),
        (
            "salesforce-api-version = \"65\"\n",
            "project.invalid-api-version",
        ),
        (
            "salesforce-api-version = \"65.0\"\nsource-root = \"../src\"\n",
            "project.invalid-config-path",
        ),
        (
            "salesforce-api-version = \"65.0\"\nunknown = \"x\"\n",
            "project.unknown-config-key",
        ),
        (
            "salesforce-api-version = \"65.0\"\nsalesforce-api-version = \"65.0\"\n",
            "project.duplicate-config-key",
        ),
        (
            "salesforce-api-version \"65.0\"\n",
            "project.invalid-config",
        ),
        ("salesforce-api-version = 65.0\n", "project.invalid-config"),
        (
            "salesforce-api-version = \"65.0\"\nsource-root = \"missing\"\n",
            "project.source-root-unavailable",
        ),
        (
            "salesforce-api-version = \"65.0\"\napex-source-root = \"missing\"\n",
            "project.apex-source-root-unavailable",
        ),
    ];
    for (config, expected) in cases {
        let project = TempProject::new(config, &[], None);
        let compilation = compile_project(project.path());
        assert!(
            compilation
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == expected),
            "expected {expected}: {:#?}",
            compilation.diagnostics
        );
    }

    let no_sources = TempProject::standard(&[]);
    assert!(
        compile_project(no_sources.path())
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "project.no-sources")
    );

    let missing_boundary = TempProject::new(
        "salesforce-api-version = \"65.0\"\napex-boundary = \"missing.api\"\n",
        &[("Main.zen", "public class Main {}")],
        None,
    );
    assert!(
        compile_project(missing_boundary.path())
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "project.read-failed")
    );
}

#[test]
fn semantic_failures_are_explicit_and_never_emit_partial_apex() {
    let cases = [
        (
            "Main.zen",
            "public class Main { public static String run() { return Missing; } }",
            "resolve.unknown-name",
        ),
        (
            "Main.zen",
            "public class Main { public static String run() { return 1; } }",
            "type.incompatible-value",
        ),
        (
            "Main.zen",
            "public class Main { public static String run(Boolean ok) { if (ok) { return 'x'; } } }",
            "type.missing-return",
        ),
        (
            "Main.zen",
            "public class Main { List<String, String> values; }",
            "type.wrong-generic-arity",
        ),
        (
            "Main.zen",
            "public class Main { public static Main make() { return new Main(); } }",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "public class Main { public Main() {} }",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "public class Main { public static void run() { break; } }",
            "type.break-outside-loop",
        ),
        (
            "Main.zen",
            "public class Main { public static void run(List<String> values) { values.missing(); } }",
            "resolve.no-matching-method",
        ),
        (
            "Main.zen",
            "public class Main { public static void run(String value) { if (value) {} } }",
            "type.incompatible-value",
        ),
        (
            "Main.zen",
            "public class Main { public static Main run() { return Main; } }",
            "type.class-name-as-value",
        ),
        (
            "Other.zen",
            "public class Main {}",
            "project.class-file-mismatch",
        ),
        (
            "Main.zen",
            "public class Main { public static String run(Map<String, String> values) { return values['x']; } }",
            "type.not-indexable",
        ),
        (
            "Main.zen",
            "public class Main { public static Boolean run(List<String> values) { return values.add('x'); } }",
            "type.incompatible-value",
        ),
        (
            "Main.zen",
            "public class Main { public static String run(Boolean ok) { if (ok) String hidden = 'x'; return hidden; } }",
            "resolve.unknown-name",
        ),
        (
            "Main.zen",
            "public class Main { private static final String VALUE = 'x'; public static void run() { VALUE = 'y'; } }",
            "type.invalid-assignment-target",
        ),
        (
            "Main.zen",
            "public class Main { public String Value { get; } public void run() { Value = 'x'; } }",
            "type.invalid-assignment-target",
        ),
        (
            "Main.zen",
            "public class Main { private final String value; }",
            "type.uninitialized-final-field",
        ),
        (
            "ZenithGenerated_Main.zen",
            "public class ZenithGenerated_Main {}",
            "resolve.reserved-generated-name",
        ),
        (
            "Main.zen",
            "public class Main { private String ZenithGenerated_value; }",
            "resolve.reserved-generated-name",
        ),
        (
            "Main.zen",
            "public class Main { public void ZenithGenerated_run() {} }",
            "resolve.reserved-generated-name",
        ),
        (
            "Main.zen",
            "public final virtual class Main {}",
            "resolve.conflicting-modifiers",
        ),
        (
            "Main.zen",
            "public class Main { public override void run() {} }",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "public class Main { public static Integer run() { return -2147483649; } }",
            "type.integer-out-of-range",
        ),
        (
            "Main.zen",
            "public class Main { public static Integer run() { return 2147483648; } }",
            "type.integer-out-of-range",
        ),
        (
            "Main.zen",
            "public class Main { public static void run(String value, String VALUE) {} }",
            "resolve.duplicate-local",
        ),
        (
            "Main.zen",
            "public class Main { private String value; public String VALUE { get; set; } }",
            "resolve.duplicate-member",
        ),
        (
            "Main.zen",
            "public class Main { public void run(Int value) {} public void RUN(Integer other) {} }",
            "resolve.duplicate-method",
        ),
        (
            "Main.zen",
            "public class Main { public String Value { get; GET; } }",
            "resolve.duplicate-accessor",
        ),
        (
            "Main.zen",
            "public class Main { private String value; public static String run() { return value; } }",
            "type.instance-member-in-static-context",
        ),
        (
            "Main.zen",
            "public class Main { public static void run() { continue; } }",
            "type.continue-outside-loop",
        ),
        (
            "Main.zen",
            "public class Main { public static void run(Map<String, String> values) { for (String value : values) {} } }",
            "type.not-iterable",
        ),
        (
            "Main.zen",
            "public class Main { public static void run(List<Integer> values) { for (String value : values) {} } }",
            "type.incompatible-value",
        ),
        (
            "Main.zen",
            "public class Main { public static void run() { void value; } }",
            "type.void-value",
        ),
        (
            "Main.zen",
            "public class Main { Missing value; }",
            "resolve.unknown-type",
        ),
        (
            "Main.zen",
            "public class Main { public static void run() { return 1; } }",
            "type.unexpected-return-value",
        ),
        (
            "Main.zen",
            "public class Main { public static String run() { return; } }",
            "type.missing-return-value",
        ),
        (
            "Main.zen",
            "public class Main { public static void run() { do {} while (true); } }",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "public class Main { public static void run() { Integer value = 0; value++; } }",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "private class Main {}",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "public class Main { public final String Value { get; set; } }",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "public class Main { public String Value { set; } }",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "public class Main { public String Value { public get; set; } }",
            "type.unsupported-syntax",
        ),
        (
            "Main.zen",
            "public class Main { public virtual void run() {} }",
            "resolve.invalid-modifier-context",
        ),
    ];

    for (path, source, expected) in cases {
        let project = TempProject::standard(&[(path, source)]);
        let compilation = compile_project(project.path());
        assert!(
            compilation
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == expected),
            "expected {expected} for {source}: {:#?}",
            compilation.diagnostics
        );
        assert!(compilation.artifacts.is_empty());
    }
}

#[test]
fn cross_class_access_respects_member_and_setter_visibility() {
    let project = TempProject::standard(&[
        (
            "Owner.zen",
            "public class Owner { private String Secret; public String Name { get; private set; } private void hidden() {} }",
        ),
        (
            "Consumer.zen",
            "public class Consumer { public static void run(Owner value) { value.Secret; value.hidden(); value.Name = 'changed'; } }",
        ),
    ]);
    let compilation = compile_project(project.path());
    let codes: Vec<_> = compilation
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();
    assert!(codes.contains(&"resolve.inaccessible-member"), "{codes:?}");
    assert!(codes.contains(&"resolve.inaccessible-method"), "{codes:?}");
    assert!(
        codes.contains(&"type.invalid-assignment-target"),
        "{codes:?}"
    );
    assert!(compilation.artifacts.is_empty());
}

#[test]
fn verification_evidence_is_complete_and_json_escaped() {
    let project = TempProject::standard(&[("Main.zen", "public class Main {}")]);
    let mut compilation = compile_project(project.path());
    let result = VerificationResult {
        outcome: VerificationOutcome::Failed,
        backend: "apex-exec".into(),
        revision: "abc123".into(),
        capability_profile: "profile".into(),
        exit_status: Some(7),
        stdout: "line one\n\"quoted\"".into(),
        stderr: "path\\failure".into(),
        message: "generated Apex compiler smoke check failed".into(),
    };
    record_verification(&mut compilation.artifacts, &result);
    let manifest = compilation
        .artifacts
        .iter()
        .find(|artifact| artifact.path == "build.json")
        .unwrap()
        .text()
        .unwrap();
    for expected in [
        "\"backend\": \"apex-exec\"",
        "\"revision\": \"abc123\"",
        "\"capabilityProfile\": \"profile\"",
        "\"outcome\": \"failed\"",
        "\"exitStatus\": 7",
        "\"stdout\": \"line one\\n\\\"quoted\\\"\"",
        "\"stderr\": \"path\\\\failure\"",
    ] {
        assert!(
            manifest.contains(expected),
            "missing `{expected}`:\n{manifest}"
        );
    }
}

#[test]
fn duplicate_case_insensitive_names_and_invalid_boundary_summaries_are_rejected() {
    let duplicate = TempProject::standard(&[
        ("A.zen", "public class Thing {}"),
        ("B.zen", "public class thing {}"),
    ]);
    assert!(
        compile_project(duplicate.path())
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "resolve.duplicate-class")
    );

    let boundary = TempProject::new(
        "salesforce-api-version = \"65.0\"\napex-boundary = \"apex-boundary.api\"\n",
        &[("Main.zen", "public class Main {}")],
        Some("class Bad { static void run(String value) }"),
    );
    assert!(
        compile_project(boundary.path())
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "project.invalid-apex-boundary")
    );
}

#[test]
fn non_block_if_bodies_are_braced_to_preserve_nested_control_flow() {
    let project = TempProject::standard(&[(
        "Main.zen",
        "public class Main { public static String choose(Boolean first, Boolean second) { if (first) if (second) return 'both'; return 'none'; } }",
    )]);
    let compilation = compile_project(project.path());
    assert!(
        compilation.diagnostics.is_empty(),
        "{:#?}",
        compilation.diagnostics
    );
    let apex = compilation
        .artifacts
        .iter()
        .find(|artifact| artifact.path.ends_with("Main.cls"))
        .unwrap()
        .text()
        .unwrap();
    assert!(apex.contains("if (first) {\n            if (second) {"));
    assert!(apex.contains("                return 'both';\n            }\n        }"));
}
