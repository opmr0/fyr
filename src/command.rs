use crate::log;
use colored::*;
use shell_words::split;
use std::process;


pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
}


pub fn parse_command(run: &str) -> ParsedCommand {
    let parts = split(run).unwrap_or_else(|e| {
        eprintln!("{} failed to parse command: {}", "Error:".red(), e);
        process::exit(1);
    });
    if parts.is_empty() {
        eprintln!("{} empty command", "Error:".red());
        process::exit(1);
    }
    ParsedCommand {
        cmd: parts[0].clone(),
        args: parts[1..].to_vec(),
    }
}

pub fn validate_command(command: &ParsedCommand, quiet: bool) {
    use crate::FYR;
    log!(quiet, "{} checking command...", FYR.yellow());
    if which::which(&command.cmd).is_err() {
        eprintln!("{} command '{}' not found", "Error:".red(), command.cmd);
        process::exit(1);
    }
    log!(quiet, "  '{}' {}", command.cmd, "found".green());
}
