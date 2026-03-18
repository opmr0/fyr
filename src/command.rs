use shell_words::split;
use anyhow::{Context, Result, anyhow};
pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
}

pub fn parse_command(run: &str) -> Result<ParsedCommand> {
    let parts = split(run).context("failed to parse command")?;
    if parts.is_empty() {
        return Err(anyhow!("empty command"));
    }
    Ok(ParsedCommand {
        cmd: parts[0].clone(),
        args: parts[1..].to_vec(),
    })
}

pub fn validate_command(command: &ParsedCommand, quiet: bool) -> Result<()> {
    log!(quiet, "checking command...");
    if which::which(&command.cmd).is_err() {
        return Err(anyhow!("command '{}' not found", command.cmd));
    }
    log!(quiet, "  '{}' {}", command.cmd, "found".green());
    Ok(())
}
