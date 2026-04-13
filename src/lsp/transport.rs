use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

pub struct Transport {
    // Transport is handled via stdin/stdout in client.rs
    // This module can be extended for other transport methods
}

impl Transport {
    pub fn create_process(cmd: &mut Command) -> Result<()> {
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        Ok(())
    }
}
