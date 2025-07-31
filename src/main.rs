mod ccr_config;
mod ccr_manager;
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
    List {
        /// 指定配置组 (direct|ccr)
        #[arg(long)]
        group: Option<String>,
    },
    /// 添加新配置
    Add {
        /// 配置名称
        name: String,
        /// 指定配置组 (direct|ccr)
        #[arg(long)]
        group: Option<String>,
    },
    /// 设置默认配置
    Use {
        /// 配置名称
        name: String,
        /// 指定配置组 (direct|ccr)
        #[arg(long)]
        group: Option<String>,
    },
    /// 启动claude程序
    Run {
        /// 可选的配置名称，不指定则使用默认配置
        name: Option<String>,
        /// 指定配置组 (direct|ccr)
        #[arg(long)]
        group: Option<String>,
    },
    /// 删除配置
    Remove {
        /// 配置名称
        name: String,
        /// 指定配置组 (direct|ccr)
        #[arg(long)]
        group: Option<String>,
    },

    // CCR快捷命令
    /// 添加CCR配置
    #[command(name = "add-ccr")]
    AddCcr {
        /// 配置名称
        name: String,
    },
    /// 启动CCR配置
    #[command(name = "run-ccr")]
    RunCcr {
        /// 可选的配置名称，不指定则使用默认CCR配置
        name: Option<String>,
    },
    /// 列出CCR配置
    #[command(name = "list-ccr")]
    ListCcr,
    /// 设置默认CCR配置
    #[command(name = "use-ccr")]
    UseCcr {
        /// 配置名称
        name: String,
    },
    /// 删除CCR配置
    #[command(name = "remove-ccr")]
    RemoveCcr {
        /// 配置名称
        name: String,
    },

    // CCR服务管理
    /// CCR服务管理
    Ccr {
        #[command(subcommand)]
        ccr_cmd: CcrCommands,
    },

    // Provider管理
    /// Provider管理
    Provider {
        #[command(subcommand)]
        provider_cmd: ProviderCommands,
    },
}

#[derive(Subcommand)]
enum CcrCommands {
    /// 启动CCR服务
    Start,
    /// 停止CCR服务
    Stop,
    /// 重启CCR服务
    Restart,
    /// 查看CCR服务状态
    Status,
    /// 查看CCR服务日志
    Logs,
}

#[derive(Subcommand)]
enum ProviderCommands {
    /// 列出所有Providers
    List,
    /// 添加新Provider
    Add {
        /// Provider名称
        name: String,
    },
    /// 删除Provider
    Remove {
        /// Provider名称
        name: String,
    },
    /// 显示Provider详情
    Show {
        /// Provider名称
        name: String,
    },
    /// 编辑Provider
    Edit {
        /// Provider名称
        name: String,
    },
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();

    match cli.command {
        // 统一接口命令（支持--group参数）
        Commands::List { group } => commands::cmd_list_with_group(group),
        Commands::Add { name, group } => commands::cmd_add_with_group(name, group),
        Commands::Use { name, group } => commands::cmd_use_with_group(name, group),
        Commands::Run { name, group } => commands::cmd_run_with_group(name, group),
        Commands::Remove { name, group } => commands::cmd_remove_with_group(name, group),

        // CCR快捷命令
        Commands::AddCcr { name } => commands::cmd_add_ccr(name),
        Commands::RunCcr { name } => commands::cmd_run_ccr(name),
        Commands::ListCcr => commands::cmd_list_ccr(),
        Commands::UseCcr { name } => commands::cmd_use_ccr(name),
        Commands::RemoveCcr { name } => commands::cmd_remove_ccr(name),

        // CCR服务管理
        Commands::Ccr { ccr_cmd } => match ccr_cmd {
            CcrCommands::Start => commands::cmd_ccr_start(),
            CcrCommands::Stop => commands::cmd_ccr_stop(),
            CcrCommands::Restart => commands::cmd_ccr_restart(),
            CcrCommands::Status => commands::cmd_ccr_status(),
            CcrCommands::Logs => commands::cmd_ccr_logs(),
        },

        // Provider管理
        Commands::Provider { provider_cmd } => match provider_cmd {
            ProviderCommands::List => commands::cmd_provider_list(),
            ProviderCommands::Add { name } => commands::cmd_provider_add(name),
            ProviderCommands::Remove { name } => commands::cmd_provider_remove(name),
            ProviderCommands::Show { name } => commands::cmd_provider_show(name),
            ProviderCommands::Edit { name } => commands::cmd_provider_edit(name),
        },
    }
}
