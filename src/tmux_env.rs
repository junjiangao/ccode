use crate::error::{AppError, AppResult};
use clap::ValueEnum;
use std::collections::HashSet;
use std::io;
use std::process::{Command, Output};

pub const TMUX_SYNC_ENV_VARS: &[&str] = &[
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_MODEL",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_SMALL_FAST_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
    "CLAUDE_CODE_SUBAGENT_MODEL",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TmuxEnvMode {
    Auto,
    Always,
    Never,
}

pub struct TmuxUpdateEnvGuard {
    old_value: String,
}

pub struct TmxGlobalEnvGuard {
    _old_update_env: TmuxUpdateEnvGuard,
    vars_to_clear: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TmuxClearReport {
    pub had_server: bool,
    pub session_count: usize,
}

pub fn try_patch_tmux_update_environment(
    mode: TmuxEnvMode,
    claude_args: &[String],
    quiet: bool,
) -> Option<TmuxUpdateEnvGuard> {
    if !should_patch_tmux_env(mode, claude_args) {
        return None;
    }

    match tmux_server_running() {
        Ok(true) => {}
        Ok(false) => return None,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            if !quiet {
                eprintln!("⚠️  未检测到 tmux，跳过 tmux 环境同步: {e}");
            }
            return None;
        }
        Err(e) => {
            if !quiet {
                eprintln!("⚠️  检测 tmux 状态失败，跳过 tmux 环境同步: {e}");
            }
            return None;
        }
    }

    let old_value = match tmux_show_update_environment() {
        Ok(v) => v,
        Err(e) => {
            if !quiet {
                eprintln!("⚠️  读取 tmux update-environment 失败，跳过 tmux 环境同步: {e}");
            }
            return None;
        }
    };

    let merged = merge_update_environment(&old_value, TMUX_SYNC_ENV_VARS);
    if merged == old_value {
        return None;
    }

    if let Err(e) = tmux_set_update_environment(&merged) {
        if !quiet {
            eprintln!("⚠️  更新 tmux update-environment 失败，跳过 tmux 环境同步: {e}");
        }
        return None;
    }

    if !quiet {
        println!("🧷 已为 tmux 临时启用 Claude 环境变量同步");
    }
    Some(TmuxUpdateEnvGuard { old_value })
}

/// 设置 tmux 全局环境变量（使新 pane/window 能继承）
pub fn try_set_tmux_global_env(env_vars: &[(String, String)]) -> Option<TmxGlobalEnvGuard> {
    if env_vars.is_empty() {
        return None;
    }

    // 检查 tmux server 是否运行
    match tmux_server_running() {
        Ok(true) => {}
        Ok(false) => return None, // tmux server 未运行，静默跳过
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // tmux 未安装，静默跳过
            return None;
        }
        Err(e) => {
            // 其他错误，记录警告但不阻断流程
            eprintln!("⚠️  检测 tmux 状态失败，跳过设置全局环境变量: {e}");
            return None;
        }
    }

    let mut vars_to_clear: Vec<String> = Vec::new();
    for (key, value) in env_vars {
        if let Err(e) = tmux_set_global_env(key, value) {
            eprintln!("⚠️  设置 tmux 全局环境变量 {key} 失败: {e}");
            // 清理已设置的变量
            for var in &vars_to_clear {
                let _ = tmux_unset_global(var);
            }
            return None;
        }
        vars_to_clear.push(key.clone());
    }

    Some(TmxGlobalEnvGuard {
        _old_update_env: TmuxUpdateEnvGuard {
            old_value: String::new(),
        },
        vars_to_clear,
    })
}

pub fn clear_tmux_env_vars() -> AppResult<TmuxClearReport> {
    if !tmux_server_running().map_err(tmux_io_to_app_error)? {
        return Ok(TmuxClearReport {
            had_server: false,
            session_count: 0,
        });
    }

    let sessions = list_tmux_sessions()?;
    for var in TMUX_SYNC_ENV_VARS {
        tmux_unset_global(var)?;
        for session in &sessions {
            if let Err(e) = tmux_unset_session(session, var) {
                // 会话可能在清理过程中结束，忽略该类竞态错误
                if !is_session_not_found_message(&e) {
                    return Err(AppError::CommandExecution(format!(
                        "清理 tmux 会话环境变量失败: {e}"
                    )));
                }
            }
        }
    }

    Ok(TmuxClearReport {
        had_server: true,
        session_count: sessions.len(),
    })
}

pub fn claude_args_hint_tmux_or_worktree(args: &[String]) -> bool {
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--tmux" || arg.starts_with("--tmux=") {
            return true;
        }
        if arg == "--worktree" || arg.starts_with("--worktree=") {
            return true;
        }
        if arg == "-w" {
            return true;
        }
        if arg.starts_with("-w") && arg.len() > 2 {
            return true;
        }
        i += 1;
    }
    false
}

pub fn merge_update_environment(old: &str, add: &[&str]) -> String {
    let mut merged = Vec::new();
    let mut seen = HashSet::new();

    for item in old.split_whitespace() {
        if seen.insert(item.to_string()) {
            merged.push(item.to_string());
        }
    }

    for item in add {
        if seen.insert((*item).to_string()) {
            merged.push((*item).to_string());
        }
    }

    merged.join(" ")
}

impl Drop for TmuxUpdateEnvGuard {
    fn drop(&mut self) {
        if let Err(e) = tmux_set_update_environment(&self.old_value) {
            eprintln!("⚠️  恢复 tmux update-environment 失败，请手动检查: {e}");
        }
    }
}

impl Drop for TmxGlobalEnvGuard {
    fn drop(&mut self) {
        for var in &self.vars_to_clear {
            if let Err(e) = tmux_unset_global(var) {
                eprintln!("⚠️  清理 tmux 全局环境变量 {var} 失败: {e}");
            }
        }
    }
}

fn should_patch_tmux_env(mode: TmuxEnvMode, claude_args: &[String]) -> bool {
    match mode {
        TmuxEnvMode::Never => false,
        TmuxEnvMode::Always => true,
        TmuxEnvMode::Auto => {
            claude_args_hint_tmux_or_worktree(claude_args) || std::env::var_os("TMUX").is_some()
        }
    }
}

fn list_tmux_sessions() -> AppResult<Vec<String>> {
    let output = run_tmux(&["list-sessions", "-F", "#S"]).map_err(tmux_io_to_app_error)?;
    if !output.status.success() {
        return Err(AppError::CommandExecution(format!(
            "获取 tmux 会话列表失败: {}",
            stderr_string(&output)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn tmux_unset_global(var: &str) -> AppResult<()> {
    let output = run_tmux(&["set-environment", "-g", "-u", var]).map_err(tmux_io_to_app_error)?;
    if output.status.success() {
        return Ok(());
    }

    Err(AppError::CommandExecution(format!(
        "清理 tmux 全局环境变量失败({var}): {}",
        stderr_string(&output)
    )))
}

fn tmux_unset_session(session: &str, var: &str) -> Result<(), String> {
    let args = ["set-environment", "-t", session, "-u", var];
    let output = run_tmux(&args).map_err(|e| e.to_string())?;
    if output.status.success() {
        return Ok(());
    }

    Err(stderr_string(&output))
}

fn tmux_set_global_env(var: &str, value: &str) -> AppResult<()> {
    let output = run_tmux(&["set-environment", "-g", var, value]).map_err(tmux_io_to_app_error)?;
    if output.status.success() {
        return Ok(());
    }
    Err(AppError::CommandExecution(format!(
        "设置 tmux 全局环境变量失败({var}): {}",
        stderr_string(&output)
    )))
}

fn tmux_server_running() -> io::Result<bool> {
    let output = run_tmux(&["ls"])?;
    if output.status.success() {
        return Ok(true);
    }

    let stderr = stderr_string(&output);
    if is_no_server_message(&stderr) {
        return Ok(false);
    }

    Err(io::Error::other(format!("tmux ls 失败: {stderr}")))
}

fn tmux_show_update_environment() -> AppResult<String> {
    let output =
        run_tmux(&["show-options", "-gqv", "update-environment"]).map_err(tmux_io_to_app_error)?;
    if !output.status.success() {
        return Err(AppError::CommandExecution(format!(
            "读取 tmux update-environment 失败: {}",
            stderr_string(&output)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn tmux_set_update_environment(value: &str) -> AppResult<()> {
    let output = run_tmux(&["set-option", "-g", "update-environment", value])
        .map_err(tmux_io_to_app_error)?;
    if output.status.success() {
        return Ok(());
    }

    Err(AppError::CommandExecution(format!(
        "写入 tmux update-environment 失败: {}",
        stderr_string(&output)
    )))
}

fn run_tmux(args: &[&str]) -> io::Result<Output> {
    Command::new("tmux").args(args).output()
}

fn tmux_io_to_app_error(e: io::Error) -> AppError {
    if e.kind() == io::ErrorKind::NotFound {
        return AppError::CommandExecution("找不到 'tmux' 程序，请先安装 tmux".to_string());
    }
    AppError::CommandExecution(format!("执行 tmux 命令失败: {e}"))
}

fn stderr_string(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

fn is_no_server_message(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("no server running")
        || lower.contains("failed to connect to server")
        || (lower.contains("error connecting to")
            && (lower.contains("no such file or directory")
                || lower.contains("connection refused")))
}

fn is_session_not_found_message(msg: &str) -> bool {
    msg.to_ascii_lowercase().contains("can't find session")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_update_environment_appends_missing_without_dup() {
        let old = "DISPLAY PATH ANTHROPIC_BASE_URL DISPLAY";
        let merged =
            merge_update_environment(old, &["ANTHROPIC_BASE_URL", "ANTHROPIC_AUTH_TOKEN", "PATH"]);
        assert_eq!(
            merged,
            "DISPLAY PATH ANTHROPIC_BASE_URL ANTHROPIC_AUTH_TOKEN"
        );
    }

    #[test]
    fn merge_update_environment_from_empty() {
        let merged = merge_update_environment("", &["A", "B", "C"]);
        assert_eq!(merged, "A B C");
    }

    #[test]
    fn detect_tmux_or_worktree_args() {
        assert!(claude_args_hint_tmux_or_worktree(&[
            "--tmux".to_string(),
            "--worktree".to_string(),
        ]));
        assert!(claude_args_hint_tmux_or_worktree(&[
            "--tmux=classic".to_string(),
            "code".to_string(),
        ]));
        assert!(claude_args_hint_tmux_or_worktree(&[
            "--worktree=test".to_string(),
            "code".to_string(),
        ]));
        assert!(claude_args_hint_tmux_or_worktree(&[
            "-w".to_string(),
            "branch-a".to_string(),
        ]));
        assert!(claude_args_hint_tmux_or_worktree(&[
            "-wbranch-b".to_string(),
            "code".to_string(),
        ]));
        assert!(!claude_args_hint_tmux_or_worktree(&[
            "--version".to_string(),
            "code".to_string(),
        ]));
    }

    #[test]
    fn treat_missing_tmux_socket_as_no_server() {
        assert!(is_no_server_message(
            "error connecting to /tmp/tmux-14397/default (No such file or directory)"
        ));
    }
}
