use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::ffi::OsStr;
use std::ops::{AddAssign, MulAssign};
use std::process::Command;

use serde::Serialize;
use structopt::StructOpt;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(
    Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumString, Display,
)]
enum Os {
    #[strum(serialize = "ubuntu")]
    Ubuntu,

    #[strum(serialize = "windows")]
    Windows,
}

#[derive(
    Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumString, Display,
)]
enum Toolchain {
    #[strum(serialize = "nightly")]
    Nightly,

    #[strum(serialize = "stable")]
    Stable,

    #[strum(serialize = "1.48")]
    Msrv,
}

#[derive(
    Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumString, Display,
)]
enum Dir {
    #[strum(serialize = "lib")]
    Lib,

    #[strum(serialize = "lib/macro/internal")]
    MacroInternal,

    #[strum(serialize = "lib/macro")]
    Macro,

    #[strum(serialize = "bin")]
    Bin,

    #[strum(serialize = "nostd")]
    Nostd,

    #[strum(serialize = "lib/fuzz")]
    Fuzz,

    #[strum(serialize = "cmp")]
    Cmp,

    #[strum(serialize = "www")]
    Www,
}

impl Dir {
    fn is_published(self) -> bool {
        matches!(self, Dir::Lib | Dir::MacroInternal | Dir::Macro | Dir::Bin)
    }
}

#[derive(
    Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumString, Display,
)]
enum Task {
    #[strum(serialize = "fmt")]
    Format,

    #[strum(serialize = "clippy")]
    Clippy,

    #[strum(serialize = "build")]
    Build,

    #[strum(serialize = "test")]
    Test,

    #[strum(serialize = "doc")]
    Doc,

    #[strum(serialize = "miri")]
    Miri,

    #[strum(serialize = "bench")]
    Bench,

    #[strum(serialize = "semver-checks")]
    SemverChecks,

    #[strum(serialize = "audit")]
    Audit,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Action {
    os: Os,
    toolchain: Toolchain,
    task: Task,
    dir: Dir,
}

impl Action {
    fn interpret(&self) -> Instructions {
        let mut instructions = Instructions::default();
        let default_env: &[(&str, &str)] = match self.task {
            Task::Doc => &[("RUSTDOCFLAGS", "--deny=warnings")],
            _ => &[],
        };
        let default_args: &[&str] = match (self.task, self.dir) {
            (Task::Format, _) => &["--", "--check"],
            (Task::Clippy, _) => &["--", "--deny=warnings"],
            (Task::Build, Dir::Nostd) => &["--release"],
            (Task::Miri, _) => &["test"],
            (Task::SemverChecks, _) => &["check-release"],
            (Task::Audit, _) => &["--deny=warnings"],
            _ => &[],
        };
        instructions += Instruction {
            executor: Executor::Cargo,
            env: default_env.iter().map(|(x, y)| (x.to_string(), y.to_string())).collect(),
            cmd: self.task.to_string(),
            args: default_args.iter().map(|x| x.to_string()).collect(),
        };
        if self.task == Task::Build {
            if self.dir.is_published() {
                instructions *= &[&["--release"]];
            }
            if self.dir == Dir::Lib {
                instructions *=
                    &[&["--no-default-features", "--features=alloc"], &["--no-default-features"]];
            }
        }
        if self.dir == Dir::Nostd && self.task == Task::Test {
            instructions = Instructions::default();
            instructions += Instruction {
                executor: Executor::Cargo,
                env: vec![],
                cmd: "run".to_string(),
                args: vec!["--release".to_string()],
            };
            instructions *= &[&["--features=alloc"]];
        }
        if self.dir == Dir::Bin && matches!(self.task, Task::Test | Task::Bench) {
            instructions = Instructions::default();
            instructions += Instruction {
                executor: Executor::Shell,
                env: vec![],
                cmd: format!("./{}.sh", self.task),
                args: vec![format!("+{}", self.toolchain)],
            };
        }
        if self.toolchain == Toolchain::Msrv {
            let lock = "Cargo.lock";
            let backup = "Cargo.lock.backup";
            let msrv = "Cargo.lock.msrv";
            // We assume a lock file already exists, which should often by the case.
            instructions += Instruction::shell("mv", &[lock, backup]);
            instructions += Instruction::shell("cp", &[msrv, lock]);
            instructions.0.rotate_right(2);
            // We have to remove first because Windows cannot move on an existing file.
            instructions += Instruction::shell("rm", &[lock]);
            instructions += Instruction::shell("mv", &[backup, lock]);
        }
        instructions
    }
}

#[derive(Copy, Clone, Debug)]
enum Executor {
    Cargo,
    Shell,
}

#[derive(Clone, Debug)]
struct Instruction {
    executor: Executor,
    env: Vec<(String, String)>,
    cmd: String,
    args: Vec<String>,
}

impl Instruction {
    fn shell(cmd: &str, args: &[&str]) -> Self {
        Instruction {
            executor: Executor::Shell,
            env: vec![],
            cmd: cmd.to_string(),
            args: args.iter().map(|x| x.to_string()).collect(),
        }
    }

    fn execute(&self, toolchain: Toolchain, dir: Dir) {
        let mut command = match self.executor {
            Executor::Cargo => {
                let mut command = Command::new("cargo");
                command.arg(format!("+{toolchain}"));
                command.arg(&self.cmd);
                command
            }
            Executor::Shell => Command::new(&self.cmd),
        };
        command.current_dir(format!("{dir}"));
        command.envs(self.env.iter().map(|(x, y)| (x, y)));
        command.args(&self.args);
        execute_command(command);
    }

    fn generate(&self, toolchain: Toolchain, dir: Dir) -> Vec<WorkflowStep> {
        let mut step = WorkflowStep::default();
        step.env = self.env.iter().cloned().collect();
        match self.executor {
            Executor::Cargo => {
                let mut run = format!("cargo +{toolchain} {}", self.cmd);
                for arg in &self.args {
                    run.push(' ');
                    run.push_str(&shell_escape::escape(arg.into()));
                }
                step.name = Some(format!("cd {dir} && {run}"));
                step.run = Some(run);
                step.working_directory = Some(dir.to_string());
            }
            Executor::Shell => {
                let mut run = format!("cd {} && ", shell_escape::escape(dir.to_string().into()));
                run.push_str(&shell_escape::escape(Cow::Borrowed(&self.cmd)));
                for arg in &self.args {
                    run.push(' ');
                    run.push_str(&shell_escape::escape(arg.into()));
                }
                step.run = Some(run);
            }
        }
        vec![step]
    }
}

#[derive(Clone, Default, Debug)]
struct Instructions(Vec<Instruction>);

impl AddAssign<Instruction> for Instructions {
    fn add_assign(&mut self, instruction: Instruction) {
        self.0.push(instruction);
    }
}

impl MulAssign<&[&[&str]]> for Instructions {
    fn mul_assign(&mut self, extra_args: &[&[&str]]) {
        let n = self.0.len();
        assert!(n > 0);
        for args in extra_args {
            for i in 0 .. n {
                let mut instruction = self.0[i].clone();
                instruction.args.extend(args.iter().map(|x| x.to_string()));
                self.0.push(instruction);
            }
        }
    }
}

fn execute_command(mut command: Command) {
    eprint!("\x1b[1;36m");
    if let Some(dir) = command.get_current_dir() {
        eprint!("cd {} && ", escape(dir.as_os_str()));
    }
    for (k, v) in command.get_envs() {
        match v {
            None => eprint!("--unset={} ", escape(k)),
            Some(v) => eprint!("{}={} ", escape(k), escape(v)),
        }
    }
    eprint!("{}", escape(command.get_program()));
    for arg in command.get_args() {
        eprint!(" {}", escape(arg));
    }
    eprintln!("\x1b[1m");
    let code = command.spawn().unwrap().wait().unwrap().code().unwrap();
    if code != 0 {
        std::process::exit(code);
    }
}

fn escape(x: &OsStr) -> Cow<str> {
    shell_escape::escape(x.to_str().unwrap().into())
}

#[derive(Debug, StructOpt)]
enum Flags {
    /// Generates the Github workflow file.
    Generate,

    /// Runs the integration tests.
    Test(Test),
}

#[derive(Debug, StructOpt)]
struct Test {
    /// Only run for those toolchains.
    #[structopt(long)]
    toolchain: Vec<Toolchain>,

    /// Only run for those directories.
    #[structopt(long)]
    dir: Vec<Dir>,

    /// Only run those tasks.
    #[structopt(long)]
    task: Vec<Task>,
}

#[derive(Serialize)]
struct Workflow {
    name: String,
    on: WorkflowOn,
    jobs: BTreeMap<String, WorkflowJob>,
}

#[derive(Serialize)]
struct WorkflowOn {
    push: WorkflowEvents,
    pull_request: WorkflowEvents,
    schedule: Vec<WorkflowSchedule>,
}

#[derive(Serialize)]
struct WorkflowEvents {
    branches: Vec<String>,
}

#[derive(Serialize)]
struct WorkflowSchedule {
    cron: String,
}

#[derive(Serialize)]
struct WorkflowJob {
    #[serde(rename = "runs-on")]
    runs_on: String,
    steps: Vec<WorkflowStep>,
}

#[derive(Default, Serialize)]
struct WorkflowStep {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uses: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    env: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "working-directory")]
    working_directory: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    with: BTreeMap<String, String>,
}

impl Flags {
    fn execute(self) {
        match self {
            Flags::Generate => {
                let actions = Actions::new().flatten();
                let mut ci = Workflow {
                    name: "Continuous Integration".to_owned(),
                    on: WorkflowOn {
                        push: WorkflowEvents { branches: vec!["main".to_owned()] },
                        pull_request: WorkflowEvents { branches: vec!["main".to_owned()] },
                        schedule: vec![WorkflowSchedule { cron: "38 11 * * 6".to_owned() }],
                    },
                    jobs: BTreeMap::new(),
                };
                for actions in actions.chunk_by(|x, y| x.os == y.os) {
                    let mut job =
                        WorkflowJob { runs_on: format!("{}-latest", actions[0].os), steps: vec![] };
                    job.steps.push(WorkflowStep {
                        uses: Some("actions/checkout@v4".to_owned()),
                        ..Default::default()
                    });
                    for actions in actions.chunk_by(|x, y| x.toolchain == y.toolchain) {
                        job.steps.push(WorkflowStep {
                            run: Some(format!("rustup install {}", actions[0].toolchain)),
                            ..Default::default()
                        });
                        let components: BTreeSet<_> = actions
                            .iter()
                            .filter_map(|x| match x.task {
                                Task::Format => Some("rustfmt"),
                                Task::Clippy => Some("clippy"),
                                Task::Miri => Some("miri"),
                                _ => None,
                            })
                            .collect();
                        if !components.is_empty() {
                            let mut run = format!(
                                "rustup component add --toolchain={}",
                                actions[0].toolchain
                            );
                            for component in components {
                                run.push(' ');
                                run.push_str(component);
                            }
                            job.steps.push(WorkflowStep { run: Some(run), ..Default::default() });
                        }
                        if matches!(
                            actions[0],
                            Action { os: Os::Ubuntu, toolchain: Toolchain::Nightly, .. }
                        ) {
                            job.steps.push(WorkflowStep {
                                uses: Some("actions/cache@v4".to_owned()),
                                with: [
                                    (
                                        "path".to_owned(),
                                        "~/.cargo/bin\n~/.cargo/.crates*\n".to_owned(),
                                    ),
                                    ("key".to_owned(), "cargo-home-${{ runner.os }}".to_owned()),
                                ]
                                .into_iter()
                                .collect(),
                                ..Default::default()
                            });
                        }
                        for task in [Task::Audit, Task::SemverChecks] {
                            if actions.iter().any(|x| x.task == task) {
                                job.steps.push(WorkflowStep {
                                    run: Some(format!(
                                        "cargo +{} install cargo-{task} --locked",
                                        actions[0].toolchain
                                    )),
                                    ..Default::default()
                                });
                            }
                        }
                        for action in actions {
                            for instruction in action.interpret().0 {
                                job.steps
                                    .extend(instruction.generate(action.toolchain, action.dir));
                            }
                        }
                    }
                    ci.jobs.insert(actions[0].os.to_string(), job);
                }
                let ci = serde_yaml::to_string(&ci).unwrap();
                std::fs::write(".github/workflows/ci.yml", ci).unwrap();
            }
            Flags::Test(test) => test.execute(),
        }
    }
}

impl Test {
    fn execute(self) {
        let mut actions = Actions::new();
        if !self.toolchain.is_empty() {
            let toolchains: HashSet<Toolchain> = self.toolchain.into_iter().collect();
            actions.0.retain(|x| toolchains.contains(&x.toolchain));
        }
        if !self.dir.is_empty() {
            let dirs: HashSet<Dir> = self.dir.into_iter().collect();
            actions.0.retain(|x| dirs.contains(&x.dir));
        }
        if !self.task.is_empty() {
            let tasks: HashSet<Task> = self.task.into_iter().collect();
            actions.0.retain(|x| tasks.contains(&x.task));
        }
        for action in actions.flatten() {
            for instruction in action.interpret().0 {
                instruction.execute(action.toolchain, action.dir);
            }
        }
    }
}

struct Actions(HashSet<Action>);

impl Actions {
    fn new() -> Actions {
        let mut actions = HashSet::new();
        // Check everything on ubuntu nightly.
        for task in Task::iter() {
            for dir in Dir::iter() {
                let os = Os::Ubuntu;
                let mut toolchain = Toolchain::Nightly;
                if task == Task::Doc && !matches!(dir, Dir::Lib) {
                    // Documentation only matters for the library.
                    continue;
                }
                if task == Task::Clippy && matches!(dir, Dir::Cmp | Dir::Www) {
                    // Clippy is currently broken on cmp and www.
                    continue;
                }
                if task == Task::Miri && !matches!(dir, Dir::Lib) {
                    // Miri is slow, so only run where it matters.
                    continue;
                }
                if task == Task::Bench && !matches!(dir, Dir::Lib | Dir::Bin) {
                    // Bench is only supported for lib and bin.
                    continue;
                }
                if task == Task::SemverChecks {
                    if !dir.is_published() || matches!(dir, Dir::Bin | Dir::MacroInternal) {
                        // SemverChecks only makes sense for published library crates (not binary
                        // and not proc-macro).
                        continue;
                    }
                    // SemverChecks only guarantees support for stable.
                    toolchain = Toolchain::Stable;
                }
                actions.insert(Action { os, toolchain, task, dir });
            }
        }
        // Build published crates on ubuntu and windows with all toolchains.
        for os in Os::iter() {
            for toolchain in Toolchain::iter() {
                for dir in Dir::iter().filter(|x| x.is_published()) {
                    if toolchain == Toolchain::Msrv && dir == Dir::Bin {
                        // Only the libraries need to compile with the MSRV.
                        continue;
                    }
                    let task = Task::Build;
                    actions.insert(Action { os, toolchain, task, dir });
                }
            }
        }
        Actions(actions)
    }

    fn flatten(self) -> Vec<Action> {
        let mut actions: Vec<_> = self.0.into_iter().collect();
        actions.sort();
        actions
    }
}

fn main() {
    Flags::from_args().execute();
}
