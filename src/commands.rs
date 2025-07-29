use crate::config::{Config, Profile};
use crate::error::{AppError, AppResult};
use chrono::Utc;
use std::io::{self, Write};
use std::process::Command;

/// 列出所有配置
pub fn cmd_list() -> AppResult<()> {
    let config = match Config::load() {
        Ok(config) => config,
        Err(AppError::ConfigNotFound) => {
            println!("📋 暂无配置，请使用 'ccode add <name>' 添加配置");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let profiles = config.list_profiles();

    if profiles.is_empty() {
        println!("📋 暂无配置，请使用 'ccode add <name>' 添加配置");
        return Ok(());
    }

    println!("📋 可用配置：");
    println!();

    for (name, profile, is_default) in profiles {
        let default_marker = if is_default { " (默认)" } else { "" };
        println!("🔧 {name}{default_marker}");
        println!("   📍 URL: {}", profile.anthropic_base_url);
        println!(
            "   🔑 Token: {}...{}",
            &profile.anthropic_auth_token[..7],
            &profile.anthropic_auth_token[profile.anthropic_auth_token.len() - 4..]
        );

        if let Some(desc) = &profile.description {
            println!("   📝 描述: {desc}");
        }

        if let Some(created) = &profile.created_at {
            println!("   📅 创建: {created}");
        }
        println!();
    }

    Ok(())
}

/// 交互式添加配置
pub fn cmd_add(name: String) -> AppResult<()> {
    let mut config = Config::load().unwrap_or_default();

    // 检查配置是否已存在
    if config.profiles.contains_key(&name) {
        return Err(AppError::Config(format!("配置 '{name}' 已存在")));
    }

    println!("🔧 添加新配置: {name}");
    println!();

    // 获取认证令牌
    print!("🔑 请输入 ANTHROPIC_AUTH_TOKEN (支持各种第三方API格式): ");
    io::stdout().flush().unwrap();
    let mut token = String::new();
    io::stdin().read_line(&mut token)?;
    let token = token.trim().to_string();

    // 获取基础URL
    print!("📍 请输入 ANTHROPIC_BASE_URL (如: https://api.anthropic.com): ");
    io::stdout().flush().unwrap();
    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    let url = url.trim().to_string();

    // 获取描述（可选）
    print!("📝 请输入描述 (可选，直接回车跳过): ");
    io::stdout().flush().unwrap();
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim();
    let description = if description.is_empty() {
        None
    } else {
        Some(description.to_string())
    };

    // 创建配置
    let profile = Profile {
        anthropic_auth_token: token,
        anthropic_base_url: url,
        description,
        created_at: Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()),
    };

    // 添加并保存配置
    config.add_profile(name.clone(), profile)?;
    config.save()?;

    println!();
    println!("✅ 配置 '{name}' 添加成功！");

    if config.profiles.len() == 1 {
        println!("🎯 已自动设为默认配置");
    }

    Ok(())
}

/// 设置默认配置
pub fn cmd_use(name: String) -> AppResult<()> {
    let mut config = Config::load()?;

    config.set_default(&name)?;
    config.save()?;

    println!("✅ 已将 '{name}' 设为默认配置");
    Ok(())
}

/// 启动claude程序
pub fn cmd_run(name: Option<String>) -> AppResult<()> {
    let config = Config::load()?;

    let (profile_name, profile) = match name {
        Some(name) => {
            let profile = config.get_profile(&name)?;
            (name, profile)
        }
        None => {
            let (default_name, profile) = config.get_default_profile()?;
            (default_name.clone(), profile)
        }
    };

    println!("🚀 使用配置 '{profile_name}' 启动 claude...");
    println!("📍 API URL: {}", profile.anthropic_base_url);
    println!();

    // 设置环境变量并启动claude
    let mut cmd = Command::new("claude");
    cmd.env("ANTHROPIC_AUTH_TOKEN", &profile.anthropic_auth_token);
    cmd.env("ANTHROPIC_BASE_URL", &profile.anthropic_base_url);

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                println!("✅ claude 程序正常退出");
            } else {
                println!("⚠️  claude 程序异常退出，退出码: {:?}", status.code());
            }
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                return Err(AppError::CommandExecution(
                    "找不到 'claude' 程序，请确保 claude 已安装并在 PATH 中".to_string(),
                ));
            } else {
                return Err(AppError::CommandExecution(format!("执行 claude 失败: {e}")));
            }
        }
    }

    Ok(())
}

/// 删除配置
pub fn cmd_remove(name: String) -> AppResult<()> {
    let mut config = Config::load()?;

    // 确认删除
    print!("⚠️  确定要删除配置 '{name}' 吗？(y/N): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("❌ 取消删除");
        return Ok(());
    }

    config.remove_profile(&name)?;
    config.save()?;

    println!("✅ 配置 '{name}' 已删除");

    // 如果还有其他配置，显示当前默认配置
    if !config.profiles.is_empty() {
        if let Some(default) = &config.default {
            println!("🎯 当前默认配置: {default}");
        }
    } else {
        println!("📋 暂无配置，请使用 'ccode add <name>' 添加配置");
    }

    Ok(())
}
