mod commands;
mod config;
mod error;
mod toml_config;

use clap::{Parser, Subcommand};
use error::AppResult;

/// ccode - Claude Code 环境切换工具
///
/// 一个用于快速切换不同API服务配置并启动claude程序的命令行工具
#[derive(Parser)]
#[command(name = "ccode")]
#[command(about = "Claude Code 环境切换工具", long_about = None)]
#[command(version = "0.2.0")]
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
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();

    match cli.command {
        // 统一接口命令（支持--group参数）
        Commands::List { group } => commands::cmd_list_with_group(group),
        Commands::Add { name, group } => commands::cmd_add_with_group(name, group),
        Commands::Use { name, group } => commands::cmd_use_with_group(name, group),
        Commands::Run {
            name,
            group,
            claude_args,
        } => commands::cmd_run_with_group(name, group, claude_args),
        Commands::Remove { name, group } => commands::cmd_remove_with_group(name, group),
    }
}
