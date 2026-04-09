mod commands;
mod config;
mod error;
mod migrate;
mod tmux_env;
mod toml_config;

use clap::{Parser, Subcommand};
use error::AppResult;
use tmux_env::TmuxEnvMode;

const PROFILE_SUBCOMMAND: &str = "profile";

/// 已废弃的旧命令列表，检测到时给出提示
/// 已废弃的旧命令 → 对应新命令的映射
const DEPRECATED_COMMANDS: &[(&str, &str)] = &[
    ("list", "ccode profile list"),
    ("add", "ccode profile add"),
    ("use", "ccode profile use"),
    ("remove", "ccode profile remove"),
    ("run", "ccode profile run"),
    ("config", "（JSON→TOML 迁移现在自动执行，无需手动操作）"),
    ("tmux", "ccode profile run clear-env"),
];

/// ccode profile 管理子命令
#[derive(Parser)]
#[command(name = "ccode")]
#[command(bin_name = "ccode profile")]
#[command(about = "Claude Code 环境切换工具", long_about = None)]
#[command(version)]
struct ProfileCli {
    #[command(subcommand)]
    command: ProfileCommands,
}

#[derive(Subcommand)]
enum ProfileCommands {
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
    /// 删除配置
    Remove {
        /// 配置名称
        name: String,
    },
    /// 启动claude程序
    Run {
        /// 可选的配置名称，不指定则使用默认配置
        name: Option<String>,
        /// tmux 环境同步策略：auto(默认)/always/never
        #[arg(long, value_enum, default_value_t = TmuxEnvMode::Auto)]
        tmux_env: TmuxEnvMode,
        /// 透传给claude的参数 (例如: profile run myprofile --version 或 profile run myprofile -- --help)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        claude_args: Vec<String>,
    },
    /// 清理 tmux 中 ccode 相关环境变量
    ClearEnv,
}

/// 打印 ccode 简短版本信息
fn print_ccode_version() {
    println!("ccode v{}", env!("CARGO_PKG_VERSION"));
}

/// 打印 ccode 简短使用帮助
fn print_ccode_help() {
    println!(
        "ccode v{} — Claude Code 环境切换工具",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("用法: ccode [claude 参数...]        直接启动 claude（使用默认 profile）");
    println!("      ccode profile <子命令>        管理配置");
    println!();
    println!("子命令: profile list|add|use|remove|run");
    println!("运行 'ccode profile --help' 查看详细帮助");
    println!();
    println!("以下为 claude code 的帮助信息:");
}

fn main() -> AppResult<()> {
    // 开机自动迁移：仅在存在 JSON 且缺少 TOML 时执行
    if let Err(e) = crate::migrate::auto_migrate_if_needed() {
        eprintln!("⚠️ 自动迁移失败: {e}");
    }

    let args: Vec<String> = std::env::args().collect();

    // 无参数 → 直接启动默认 profile
    if args.len() == 1 {
        return commands::cmd_run(None, TmuxEnvMode::Auto, vec![], true);
    }

    let first = args.get(1).map(|s| s.as_str()).unwrap_or("");

    // 首参数为 profile → 交给 clap 解析子命令
    if first == PROFILE_SUBCOMMAND {
        // 构造新参数：["ccode profile", "list", ...] 替换程序名以匹配 bin_name
        let mut profile_args = vec!["ccode profile".to_string()];
        profile_args.extend_from_slice(&args[2..]);
        let cli = ProfileCli::parse_from(profile_args);
        match cli.command {
            ProfileCommands::List => commands::cmd_list(),
            ProfileCommands::Add { name } => commands::cmd_add(name),
            ProfileCommands::Use { name } => commands::cmd_use(name),
            ProfileCommands::Remove { name } => commands::cmd_remove(name),
            ProfileCommands::Run {
                name,
                tmux_env,
                claude_args,
            } => commands::cmd_run(name, tmux_env, claude_args, false),
            ProfileCommands::ClearEnv => commands::cmd_tmux_clear_env(),
        }
    } else {
        // 检测废弃命令并给出提示
        if let Some((_, replacement)) = DEPRECATED_COMMANDS.iter().find(|(cmd, _)| *cmd == first) {
            if replacement.starts_with("ccode") {
                eprintln!("⚠️ 'ccode {first}' 已废弃，请使用 '{replacement}'");
            } else {
                eprintln!("⚠️ 'ccode {first}' {replacement}");
            }
            std::process::exit(1);
        }

        // 先检查 --help/-h/--version/-v/-V，输出 ccode 信息后再透传给 claude
        if ["--help", "-h"].contains(&first) {
            print_ccode_help();
        } else if ["--version", "-v", "-V"].contains(&first) {
            print_ccode_version();
        }
        let claude_args = args[1..].to_vec();
        commands::cmd_run(None, TmuxEnvMode::Auto, claude_args, true)
    }
}
