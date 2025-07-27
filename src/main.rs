mod commands;
mod config;
mod error;

use clap::{Parser, Subcommand};
use error::AppResult;

/// ccode - Claude Code 环境切换工具
///
/// 一个用于快速切换不同API服务配置并启动claude程序的命令行工具
#[derive(Parser)]
#[command(name = "ccode")]
#[command(about = "Claude Code 环境切换工具", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出所有可用配置
    List,
    /// 添加新配置
    Add {
        /// 配置名称
        name: String,
    },
    /// 设置默认配置
    Use {
        /// 配置名称
        name: String,
    },
    /// 启动claude程序
    Run {
        /// 可选的配置名称，不指定则使用默认配置
        name: Option<String>,
    },
    /// 删除配置
    Remove {
        /// 配置名称
        name: String,
    },
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => commands::cmd_list(),
        Commands::Add { name } => commands::cmd_add(name),
        Commands::Use { name } => commands::cmd_use(name),
        Commands::Run { name } => commands::cmd_run(name),
        Commands::Remove { name } => commands::cmd_remove(name),
    }
}
