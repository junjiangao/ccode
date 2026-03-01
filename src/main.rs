mod commands;
mod config;
mod error;
mod migrate;
mod tmux_env;
mod toml_config;

use clap::{Parser, Subcommand};
use error::AppResult;
use tmux_env::TmuxEnvMode;

/// ccode - Claude Code 环境切换工具
///
/// 一个用于快速切换不同API服务配置并启动claude程序的命令行工具
#[derive(Parser)]
#[command(name = "ccode")]
#[command(about = "Claude Code 环境切换工具", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出所有可用配置
    List {
        /// 指定配置组 (仅 direct)
        #[arg(long)]
        group: Option<String>,
    },
    /// 添加新配置
    Add {
        /// 配置名称
        name: String,
        /// 指定配置组 (仅 direct)
        #[arg(long)]
        group: Option<String>,
    },
    /// 设置默认配置
    Use {
        /// 配置名称
        name: String,
        /// 指定配置组 (仅 direct)
        #[arg(long)]
        group: Option<String>,
    },
    /// 启动claude程序
    Run {
        /// 可选的配置名称，不指定则使用默认配置
        name: Option<String>,
        /// 指定配置组 (仅 direct)
        #[arg(long)]
        group: Option<String>,
        /// tmux 环境同步策略：auto(默认)/always/never
        #[arg(long, value_enum, default_value_t = TmuxEnvMode::Auto)]
        tmux_env: TmuxEnvMode,
        /// 透传给claude的参数 (仅Direct模式支持，例如: run myprofile --version 或 run myprofile -- --help)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        claude_args: Vec<String>,
    },
    /// 删除配置
    Remove {
        /// 配置名称
        name: String,
        /// 指定配置组 (仅 direct)
        #[arg(long)]
        group: Option<String>,
    },
    /// 配置相关操作
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// tmux 相关操作
    Tmux {
        #[command(subcommand)]
        action: TmuxAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// 将旧版 config.json 迁移/合并到 config.toml
    Merge {},
}

#[derive(Subcommand)]
enum TmuxAction {
    /// 清理 tmux 中 ccode 相关环境变量
    ClearEnv {},
}

fn main() -> AppResult<()> {
    // 开机自动迁移：仅在存在 JSON 且缺少 TOML 时执行
    if let Err(e) = crate::migrate::auto_migrate_if_needed() {
        eprintln!("⚠️ 自动迁移失败: {e}");
    }
    let cli = Cli::parse();

    match cli.command {
        // 统一接口命令（支持--group参数）
        Commands::List { group } => commands::cmd_list_with_group(group),
        Commands::Add { name, group } => commands::cmd_add_with_group(name, group),
        Commands::Use { name, group } => commands::cmd_use_with_group(name, group),
        Commands::Run {
            name,
            group,
            tmux_env,
            claude_args,
        } => commands::cmd_run_with_group(name, group, tmux_env, claude_args),
        Commands::Remove { name, group } => commands::cmd_remove_with_group(name, group),
        Commands::Config { action } => match action {
            ConfigAction::Merge {} => commands::cmd_config_merge(),
        },
        Commands::Tmux { action } => match action {
            TmuxAction::ClearEnv {} => commands::cmd_tmux_clear_env(),
        },
    }
}
