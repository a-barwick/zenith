use std::path::{Path, PathBuf};
use std::process::Command;

pub const APEX_EXEC_M3_REVISION: &str = "1e4f1ca1938abfc996651ae447f227e0db680b6a";
pub const APEX_EXEC_M3_PROFILE: &str = "zenith-m3-apex-baseline";
pub const APEX_EXEC_M4_PROFILE: &str = "zenith-m4-safe-values";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationOutcome {
    Passed,
    Failed,
    Unsupported,
    InternalError,
}

impl VerificationOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Unsupported => "unsupported",
            Self::InternalError => "internal-error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationResult {
    pub outcome: VerificationOutcome,
    pub backend: String,
    pub revision: String,
    pub capability_profile: String,
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessVerifier {
    pub executable: PathBuf,
    pub backend: String,
    pub revision: String,
    pub capability_profile: String,
}

impl ProcessVerifier {
    pub fn apex_exec(executable: impl Into<PathBuf>, revision: impl Into<String>) -> Self {
        Self {
            executable: executable.into(),
            backend: "apex-exec".into(),
            revision: revision.into(),
            capability_profile: APEX_EXEC_M3_PROFILE.into(),
        }
    }

    pub fn with_capability_profile(mut self, capability_profile: impl Into<String>) -> Self {
        self.capability_profile = capability_profile.into();
        self
    }

    pub fn verify(&self, generated_classes: &Path) -> VerificationResult {
        if self.backend == "apex-exec" && self.capability_profile != APEX_EXEC_M3_PROFILE {
            return self.result(
                VerificationOutcome::Unsupported,
                None,
                String::new(),
                String::new(),
                format!(
                    "Apex Exec revision `{}` does not declare capability profile `{}`",
                    self.revision, self.capability_profile
                ),
            );
        }
        if self.backend == "apex-exec" && self.revision != APEX_EXEC_M3_REVISION {
            return self.result(
                VerificationOutcome::Unsupported,
                None,
                String::new(),
                String::new(),
                format!(
                    "Apex Exec revision `{}` is not the pinned M3 revision `{APEX_EXEC_M3_REVISION}`",
                    self.revision
                ),
            );
        }
        match Command::new(&self.executable)
            .arg("check")
            .arg(generated_classes)
            .output()
        {
            Ok(output) => self.result(
                if output.status.success() {
                    VerificationOutcome::Passed
                } else {
                    VerificationOutcome::Failed
                },
                output.status.code(),
                String::from_utf8_lossy(&output.stdout).into_owned(),
                String::from_utf8_lossy(&output.stderr).into_owned(),
                if output.status.success() {
                    "generated Apex compiler smoke check passed".into()
                } else {
                    "generated Apex compiler smoke check failed".into()
                },
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.result(
                VerificationOutcome::Unsupported,
                None,
                String::new(),
                String::new(),
                format!("backend executable is unavailable: {error}"),
            ),
            Err(error) => self.result(
                VerificationOutcome::InternalError,
                None,
                String::new(),
                String::new(),
                format!("could not launch backend reliably: {error}"),
            ),
        }
    }

    fn result(
        &self,
        outcome: VerificationOutcome,
        exit_status: Option<i32>,
        stdout: String,
        stderr: String,
        message: String,
    ) -> VerificationResult {
        VerificationResult {
            outcome,
            backend: self.backend.clone(),
            revision: self.revision.clone(),
            capability_profile: self.capability_profile.clone(),
            exit_status,
            stdout,
            stderr,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        APEX_EXEC_M3_REVISION, APEX_EXEC_M4_PROFILE, ProcessVerifier, VerificationOutcome,
    };
    use std::path::{Path, PathBuf};

    fn verifier(executable: PathBuf) -> ProcessVerifier {
        ProcessVerifier::apex_exec(executable, APEX_EXEC_M3_REVISION)
    }

    #[test]
    fn unsupported_profiles_and_missing_executables_are_distinct() {
        let unsupported_profile = verifier(PathBuf::from("/usr/bin/true"))
            .with_capability_profile(APEX_EXEC_M4_PROFILE)
            .verify(Path::new("."));
        assert_eq!(
            unsupported_profile.outcome,
            VerificationOutcome::Unsupported
        );
        let unsupported =
            ProcessVerifier::apex_exec("unused", "not-the-pinned-revision").verify(Path::new("."));
        assert_eq!(unsupported.outcome, VerificationOutcome::Unsupported);
        let missing =
            verifier(PathBuf::from("definitely-not-a-real-zenith-verifier")).verify(Path::new("."));
        assert_eq!(missing.outcome, VerificationOutcome::Unsupported);
    }

    #[cfg(unix)]
    #[test]
    fn process_exit_status_distinguishes_passed_and_failed() {
        let passed = verifier(PathBuf::from("/usr/bin/true")).verify(Path::new("."));
        let failed = verifier(PathBuf::from("/usr/bin/false")).verify(Path::new("."));
        assert_eq!(passed.outcome, VerificationOutcome::Passed);
        assert_eq!(failed.outcome, VerificationOutcome::Failed);
        assert_eq!(failed.exit_status, Some(1));
    }

    #[cfg(unix)]
    #[test]
    fn launch_errors_other_than_missing_executables_are_internal() {
        let result = verifier(PathBuf::from("/")).verify(Path::new("."));
        assert_eq!(result.outcome, VerificationOutcome::InternalError);
        assert!(
            result
                .message
                .starts_with("could not launch backend reliably:")
        );
    }
}
