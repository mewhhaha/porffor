use super::ParsedTest262Args;

pub(crate) struct AdmittedTest262Command {
    command: Test262Command,
    parsed: ParsedTest262Args,
}

impl AdmittedTest262Command {
    pub(crate) fn into_parts(self) -> (Test262Command, ParsedTest262Args) {
        (self.command, self.parsed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Test262Command {
    Sync,
    List,
    ListAll,
    Run,
    RunList,
    Shard,
    Report,
    ReportAll,
    PublishStatus,
    ProgressStatus,
    TriageStatus,
    FailureDetails,
    GenerateBacklog,
    CompareSnapshots,
}

impl Test262Command {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "sync" => Ok(Self::Sync),
            "list" => Ok(Self::List),
            "list-all" => Ok(Self::ListAll),
            "run" => Ok(Self::Run),
            "run-list" => Ok(Self::RunList),
            "shard" => Ok(Self::Shard),
            "report" => Ok(Self::Report),
            "report-all" => Ok(Self::ReportAll),
            "publish-status" => Ok(Self::PublishStatus),
            "progress-status" => Ok(Self::ProgressStatus),
            "triage-status" => Ok(Self::TriageStatus),
            "failure-details" => Ok(Self::FailureDetails),
            "generate-backlog" => Ok(Self::GenerateBacklog),
            "compare-snapshots" => Ok(Self::CompareSnapshots),
            _ => Err(format!("unknown test262 subcommand: {value}")),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Sync => "sync",
            Self::List => "list",
            Self::ListAll => "list-all",
            Self::Run => "run",
            Self::RunList => "run-list",
            Self::Shard => "shard",
            Self::Report => "report",
            Self::ReportAll => "report-all",
            Self::PublishStatus => "publish-status",
            Self::ProgressStatus => "progress-status",
            Self::TriageStatus => "triage-status",
            Self::FailureDetails => "failure-details",
            Self::GenerateBacklog => "generate-backlog",
            Self::CompareSnapshots => "compare-snapshots",
        }
    }

    // An exhaustive command match owns admission. Run-only options cannot be
    // silently ignored by an inventory or stored-report command.
    fn options(self) -> &'static [&'static str] {
        match self {
            Self::Sync => &["--suite-root", "--execution-backend"],
            Self::List | Self::ListAll => &["--suite-root"],
            Self::Run | Self::Report => &[
                "--suite-root",
                "--snapshot-dir",
                "--snapshot-name",
                "--execution-backend",
                "--threads",
                "--timeout-ms",
                "--reuse-case-workers",
                "--resume",
                "--matrix-node",
            ],
            Self::RunList => &[
                "--suite-root",
                "--snapshot-dir",
                "--snapshot-name",
                "--execution-backend",
                "--threads",
                "--timeout-ms",
                "--reuse-case-workers",
                "--resume",
            ],
            Self::Shard => &[
                "--suite-root",
                "--snapshot-dir",
                "--snapshot-name",
                "--execution-backend",
                "--threads",
                "--timeout-ms",
                "--reuse-case-workers",
                "--resume",
                "--shard",
            ],
            Self::ReportAll => &[
                "--suite-root",
                "--snapshot-dir",
                "--snapshot-name",
                "--execution-backend",
                "--threads",
                "--timeout-ms",
                "--reuse-case-workers",
                "--resume",
                "--max-matrix-nodes",
            ],
            Self::PublishStatus => &[
                "--suite-root",
                "--snapshot-dir",
                "--snapshot-name",
                "--execution-backend",
                "--threads",
                "--timeout-ms",
                "--reuse-case-workers",
                "--resume",
                "--readme-path",
            ],
            Self::ProgressStatus
            | Self::TriageStatus
            | Self::FailureDetails
            | Self::GenerateBacklog
            | Self::CompareSnapshots => &[
                "--suite-root",
                "--snapshot-dir",
                "--snapshot-name",
                "--execution-backend",
            ],
        }
    }

    pub(crate) fn admit(self, parsed: ParsedTest262Args) -> Result<AdmittedTest262Command, String> {
        for option in &parsed.provided_options {
            if !self.options().contains(&option.as_str()) {
                return Err(format!("test262 {} does not allow {option}", self.name()));
            }
        }
        match self {
            Self::Sync
            | Self::ReportAll
            | Self::PublishStatus
            | Self::ProgressStatus
            | Self::TriageStatus
            | Self::GenerateBacklog
                if parsed.filter.is_some() =>
            {
                return Err(format!(
                    "test262 {} does not take a positional filter",
                    self.name()
                ));
            }
            Self::RunList | Self::FailureDetails | Self::CompareSnapshots
                if parsed.filter.is_none() =>
            {
                return Err(format!(
                    "test262 {} requires a positional input",
                    self.name()
                ));
            }
            Self::Shard if !parsed.provided_options.contains("--shard") => {
                return Err("test262 shard requires --shard INDEX/TOTAL".into());
            }
            _ => {}
        }
        Ok(AdmittedTest262Command {
            command: self,
            parsed,
        })
    }
}
