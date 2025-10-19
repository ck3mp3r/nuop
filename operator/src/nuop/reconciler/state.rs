use async_trait::async_trait;
use std::path::{Path, PathBuf};

use kube::{Client, api::ApiResource};

use super::config::Config;

// Command execution abstraction following DIP (Dependency Inversion Principle)
#[async_trait]
pub trait CommandExecutor: Clone + Send + Sync + 'static {
    async fn execute(
        &self,
        script: &Path,
        command: &str,
        input: &str,
    ) -> Result<CommandResult, anyhow::Error>;
}

#[derive(Debug)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

// Default implementation using actual process execution
#[derive(Clone, Debug, Default)]
pub struct ProcessExecutor;

#[async_trait]
impl CommandExecutor for ProcessExecutor {
    async fn execute(
        &self,
        script: &Path,
        command: &str,
        input: &str,
    ) -> Result<CommandResult, anyhow::Error> {
        use std::io::{BufRead, BufReader, Write};
        use std::process::{Command, Stdio};
        use tokio::task;

        let script = script.to_path_buf();
        let command = command.to_string();
        let input = input.to_string();

        task::spawn_blocking(move || {
            let mut child = Command::new("nu")
                .arg("--stdin")
                .arg(&script)
                .arg(&command)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input.as_bytes())?;
                stdin.flush()?;
                drop(stdin);
            }

            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

            let stdout_reader = BufReader::new(stdout);
            let stderr_reader = BufReader::new(stderr);

            let mut stdout_lines = Vec::new();
            let mut stderr_lines = Vec::new();

            for line in stdout_reader.lines() {
                stdout_lines.push(line?);
            }

            for line in stderr_reader.lines() {
                stderr_lines.push(line?);
            }

            let status = child.wait()?;
            let exit_code = status.code().unwrap_or(1);
            let stderr_output = stderr_lines.join("\n");

            // Check if this is a file not found error, which should be treated as infrastructure error
            if exit_code != 0 && stderr_output.contains("nu::shell::io::file_not_found") {
                return Err(anyhow::anyhow!("Script file not found: {}", stderr_output));
            }

            Ok(CommandResult {
                exit_code,
                stdout: stdout_lines.join("\n"),
                stderr: stderr_output,
            })
        })
        .await?
    }
}

// Zero-cost generic State with default executor (following guide pattern)
#[derive(Clone)]
pub struct State<E = ProcessExecutor>
where
    E: CommandExecutor,
{
    pub api_resource: ApiResource,
    pub client: Client,
    pub config: Config,
    pub script: PathBuf,
    pub executor: E,
}

impl<E> State<E>
where
    E: CommandExecutor,
{
    #[allow(dead_code)] // Used for testing with custom executors
    pub fn new(
        api_resource: ApiResource,
        client: Client,
        config: Config,
        script: PathBuf,
        executor: E,
    ) -> Self {
        State {
            api_resource,
            client,
            config,
            script,
            executor,
        }
    }
}

// Convenience constructor for default case (maintains backward compatibility)
impl State<ProcessExecutor> {
    pub fn new_default(
        api_resource: ApiResource,
        client: Client,
        config: Config,
        script: PathBuf,
    ) -> Self {
        State {
            api_resource,
            client,
            config,
            script,
            executor: ProcessExecutor,
        }
    }
}
