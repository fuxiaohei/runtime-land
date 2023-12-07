use anyhow::Result;
use std::process::Command;

pub fn build_command(cmd_str: &str) -> Result<()> {
    let args = cmd_str.split_whitespace().collect::<Vec<&str>>();
    let mut cmd = Command::new(args[0]);
    let child = cmd.args(&args[1..]).spawn()?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow::anyhow!(err));
    }
    Ok(())
}
