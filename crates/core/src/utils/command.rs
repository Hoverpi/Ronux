use std::process::{Command, Stdio};
use anyhow::{Context, Result};

pub enum RunMode {
    Stream,
    Capture
}

pub struct CommandRunner {
    program: String,
    args: Vec<String>,
    sudo: bool,
    mode: RunMode,
}

impl CommandRunner {
    /// Start a new command
    pub fn new<S: Into<String>>(program: S) -> Self {
        Self { 
            program: program.into(), 
            args: vec![], 
            sudo: false, 
            mode: RunMode::Stream 
        }
    }

    /// Add arguments to the new command
    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Enable sudo
    pub fn sudo(mut self) -> Self {
        self.sudo = true;
        self
    }

    /// Switch to capture mode
    pub fn capture(mut self) -> Self {
        self.mode = RunMode::Capture;
        self
    }

    /// Run the command
    pub fn run(self) -> Result<Option<String>> {
        // Build program + args
        let mut cmd = if self.sudo {
            let mut c = Command::new("sudo");
            c.arg(&self.program);
            c
        } else {
            Command::new(&self.program)
        };

        for a in &self.args {
            cmd.arg(a);
        }

        // Configure stdout/stderr 
        match self.mode {
            RunMode::Stream => {
                cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
                let status = cmd
                    .status()
                    .with_context(|| format!("Failed to execute {}", self.program))?;
                if !status.success() {
                    anyhow::bail!("{} exited with {}", self.program, status);
                }
                Ok(None)
            }
            RunMode::Capture => {
                let output = cmd
                    .output()
                    .with_context(|| format!("failed to execute {}", self.program))?;
                if !output.status.success() {
                    anyhow::bail!(
                        "{} exited with {}: {}",
                        self.program,
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                let stdout = String::from_utf8(output.stdout)
                    .context("failed to parse stdout as UTF-8")?;
                Ok(Some(stdout))
            }

        }
    }
}