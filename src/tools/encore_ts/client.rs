use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
// use log::{info, error};

#[derive(Serialize, Debug)]
pub struct PrepareInput {
    pub app_root: PathBuf,
    pub runtime_version: String,
    pub local_runtime_override: Option<PathBuf>,
}

#[derive(Serialize, Debug)]
pub struct ParseInput {
    pub app_root: PathBuf,
    pub platform_id: Option<String>,
    pub local_id: String,
    pub parse_tests: bool,
}

#[derive(Serialize, Debug)]
pub struct CompileInput {
    pub debug: DebugMode,
    pub nodejs_runtime: NodeJSRuntime,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DebugMode {
    Off,
    Full,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum NodeJSRuntime {
    Node,
    Bun,
}

#[derive(Serialize, Debug)]
pub struct GenUserFacingInput {}

pub struct TsParserClient {
    _child: Child,
    stdin: std::process::ChildStdin,
    stdout: std::io::BufReader<std::process::ChildStdout>,
}

impl TsParserClient {
    pub fn new(binary_path: &Path, cwd: &Path) -> Result<Self> {
        let mut child = Command::new(binary_path)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn tsparser-encore")?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("No stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("No stdout"))?;

        Ok(Self {
            _child: child,
            stdin,
            stdout: std::io::BufReader::new(stdout),
        })
    }

    fn write_command<T: Serialize>(&mut self, cmd: &str, input: &T) -> Result<()> {
        // Line-delimited command
        writeln!(self.stdin, "{}", cmd)?;
        // JSON input
        serde_json::to_writer(&mut self.stdin, input)?;
        // Optional newline to ensure flush/separation if needed, though deserializer typically stops at value end.
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<Vec<u8>> {
        // Protocol: [len: u32 le] [status: u8] [data]
        let mut len_buf = [0u8; 4];
        self.stdout.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut status_buf = [0u8; 1];
        self.stdout.read_exact(&mut status_buf)?;
        let is_err = status_buf[0] != 0;

        let mut data = vec![0u8; len];
        self.stdout.read_exact(&mut data)?;

        if is_err {
            let msg = String::from_utf8_lossy(&data);
            Err(anyhow!("TsParser error: {}", msg))
        } else {
            Ok(data)
        }
    }

    pub fn prepare(&mut self, input: PrepareInput) -> Result<serde_json::Value> {
        self.write_command("prepare", &input)?;
        let data = self.read_response()?;
        let v: serde_json::Value = serde_json::from_slice(&data)?;
        Ok(v)
    }

    pub fn parse(&mut self, input: ParseInput) -> Result<Vec<u8>> {
        self.write_command("parse", &input)?;
        let data = self.read_response()?;
        Ok(data) // Protobuf bytes
    }

    pub fn gen_user_facing(&mut self) -> Result<()> {
        self.write_command("gen-user-facing", &GenUserFacingInput {})?;
        let _ = self.read_response()?; // Expect empty data on success
        Ok(())
    }

    pub fn compile(&mut self, input: CompileInput) -> Result<serde_json::Value> {
        self.write_command("compile", &input)?;
        let data = self.read_response()?;
        let v: serde_json::Value = serde_json::from_slice(&data)?;
        Ok(v)
    }
}
