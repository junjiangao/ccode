use crate::ccr_manager::CcrManager;
use crate::config::{CcrProfile, CcrProvider, CcrRouter, Config, Profile};
use crate::error::{AppError, AppResult};
use chrono::Utc;
use serde_json::json;
use std::io::{self, Write};
use std::process::Command;

/// 列出所有配置
#[allow(dead_code)]
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

    // 检查配置是否已存在（检查direct组）
    if config.groups.direct.contains_key(&name) {
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
    config.add_direct_profile(name.clone(), profile)?;
    config.save()?;

    println!();
    println!("✅ 配置 '{name}' 添加成功！");

    if config.groups.direct.len() == 1 {
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

    config.remove_profile(&name)?; // 这个方法会自动检测组类型
    config.save()?;

    println!("✅ 配置 '{name}' 已删除");

    // 如果还有其他配置，显示当前默认配置
    if !config.groups.direct.is_empty() || !config.groups.ccr.is_empty() {
        if let Some(default_profile) = &config.default_profile {
            if let Some(direct) = &default_profile.direct {
                println!("🎯 当前默认Direct配置: {direct}");
            }
            if let Some(ccr) = &default_profile.ccr {
                println!("🎯 当前默认CCR配置: {ccr}");
            }
        }
    } else {
        println!("📋 暂无配置，请使用 'ccode add <name>' 添加配置");
    }

    Ok(())
}

// ==================== 统一接口命令（支持--group参数） ====================

/// 列出配置（统一接口）
pub fn cmd_list_with_group(group: Option<String>) -> AppResult<()> {
    match group.as_deref() {
        Some("direct") => cmd_list_direct(),
        Some("ccr") => cmd_list_ccr(),
        Some(g) => Err(AppError::Config(format!("未知的配置组: {g}"))),
        None => cmd_list_all(),
    }
}

/// 添加配置（统一接口）
pub fn cmd_add_with_group(name: String, group: Option<String>) -> AppResult<()> {
    match group.as_deref() {
        Some("direct") => cmd_add_direct(name),
        Some("ccr") => cmd_add_ccr(name),
        Some(g) => Err(AppError::Config(format!("未知的配置组: {g}"))),
        None => cmd_add_direct(name), // 默认使用direct组
    }
}

/// 设置默认配置（统一接口）
pub fn cmd_use_with_group(name: String, group: Option<String>) -> AppResult<()> {
    match group.as_deref() {
        Some("direct") => cmd_use_direct(name),
        Some("ccr") => cmd_use_ccr(name),
        Some(g) => Err(AppError::Config(format!("未知的配置组: {g}"))),
        None => cmd_use(name), // 向后兼容
    }
}

/// 运行配置（统一接口）
pub fn cmd_run_with_group(name: Option<String>, group: Option<String>) -> AppResult<()> {
    match group.as_deref() {
        Some("direct") => cmd_run_direct(name),
        Some("ccr") => cmd_run_ccr(name),
        Some(g) => Err(AppError::Config(format!("未知的配置组: {g}"))),
        None => cmd_run(name), // 向后兼容
    }
}

/// 删除配置（统一接口）
pub fn cmd_remove_with_group(name: String, group: Option<String>) -> AppResult<()> {
    match group.as_deref() {
        Some("direct") => cmd_remove_direct(name),
        Some("ccr") => cmd_remove_ccr(name),
        Some(g) => Err(AppError::Config(format!("未知的配置组: {g}"))),
        None => cmd_remove(name), // 向后兼容
    }
}

// ==================== Direct组专用命令 ====================

/// 列出所有配置（显示所有组）
pub fn cmd_list_all() -> AppResult<()> {
    let config = match Config::load() {
        Ok(config) => config,
        Err(AppError::ConfigNotFound) => {
            println!("📋 暂无配置，请使用 'ccode add <name>' 添加配置");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let direct_profiles = config.list_direct_profiles();
    let ccr_profiles = config.list_ccr_profiles();

    if direct_profiles.is_empty() && ccr_profiles.is_empty() {
        println!("📋 暂无配置，请使用 'ccode add <name>' 添加配置");
        return Ok(());
    }

    println!("📋 所有配置：");
    println!();

    // 显示Direct组配置
    if !direct_profiles.is_empty() {
        println!("🔗 Direct组配置：");
        for (name, profile, is_default) in direct_profiles {
            let default_marker = if is_default { " (默认)" } else { "" };
            println!("  🔧 {name}{default_marker}");
            println!("     📍 URL: {}", profile.anthropic_base_url);
            println!(
                "     🔑 Token: {}...{}",
                &profile.anthropic_auth_token[..7.min(profile.anthropic_auth_token.len())],
                &profile.anthropic_auth_token
                    [profile.anthropic_auth_token.len().saturating_sub(4)..]
            );
            if let Some(desc) = &profile.description {
                println!("     📝 描述: {desc}");
            }
            if let Some(created) = &profile.created_at {
                println!("     📅 创建: {created}");
            }
            println!();
        }
    }

    // 显示CCR组配置
    if !ccr_profiles.is_empty() {
        println!("🚀 CCR组配置：");
        for (name, profile, is_default) in ccr_profiles {
            let default_marker = if is_default { " (默认)" } else { "" };
            println!("  🔧 {name}{default_marker}");
            println!("     🔗 提供商数量: {}", profile.providers.len());
            println!("     🎯 默认路由: {}", profile.router.default);
            if let Some(desc) = &profile.description {
                println!("     📝 描述: {desc}");
            }
            if let Some(created) = &profile.created_at {
                println!("     📅 创建: {created}");
            }
            println!();
        }
    }

    Ok(())
}

/// 列出Direct组配置
pub fn cmd_list_direct() -> AppResult<()> {
    let config = match Config::load() {
        Ok(config) => config,
        Err(AppError::ConfigNotFound) => {
            println!("📋 暂无Direct配置，请使用 'ccode add --group direct <name>' 添加配置");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let profiles = config.list_direct_profiles();

    if profiles.is_empty() {
        println!("📋 暂无Direct配置，请使用 'ccode add --group direct <name>' 添加配置");
        return Ok(());
    }

    println!("📋 Direct组配置：");
    println!();

    for (name, profile, is_default) in profiles {
        let default_marker = if is_default { " (默认)" } else { "" };
        println!("🔧 {name}{default_marker}");
        println!("   📍 URL: {}", profile.anthropic_base_url);
        println!(
            "   🔑 Token: {}...{}",
            &profile.anthropic_auth_token[..7.min(profile.anthropic_auth_token.len())],
            &profile.anthropic_auth_token[profile.anthropic_auth_token.len().saturating_sub(4)..]
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

/// 添加Direct配置
pub fn cmd_add_direct(name: String) -> AppResult<()> {
    cmd_add(name) // 复用现有的逻辑
}

/// 设置默认Direct配置
pub fn cmd_use_direct(name: String) -> AppResult<()> {
    let mut config = Config::load()?;
    config.set_default_direct(&name)?;
    config.save()?;
    println!("✅ 已将 '{name}' 设为默认Direct配置");
    Ok(())
}

/// 运行Direct配置
pub fn cmd_run_direct(name: Option<String>) -> AppResult<()> {
    cmd_run(name) // 复用现有的逻辑
}

/// 删除Direct配置
pub fn cmd_remove_direct(name: String) -> AppResult<()> {
    let mut config = Config::load()?;

    // 确认删除
    print!("⚠️  确定要删除Direct配置 '{name}' 吗？(y/N): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("❌ 取消删除");
        return Ok(());
    }

    config.remove_direct_profile(&name)?;
    config.save()?;

    println!("✅ Direct配置 '{name}' 已删除");

    // 显示当前默认配置
    if !config.groups.direct.is_empty() {
        if let Some(default_profile) = &config.default_profile {
            if let Some(direct) = &default_profile.direct {
                println!("🎯 当前默认Direct配置: {direct}");
            }
        }
    } else {
        println!("📋 暂无Direct配置，请使用 'ccode add --group direct <name>' 添加配置");
    }

    Ok(())
}

// ==================== CCR组专用命令 ====================

/// 列出CCR配置
pub fn cmd_list_ccr() -> AppResult<()> {
    let config = match Config::load() {
        Ok(config) => config,
        Err(AppError::ConfigNotFound) => {
            println!("📋 暂无CCR配置，请使用 'ccode add-ccr <name>' 添加配置");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let profiles = config.list_ccr_profiles();

    if profiles.is_empty() {
        println!("📋 暂无CCR配置，请使用 'ccode add-ccr <name>' 添加配置");
        return Ok(());
    }

    println!("📋 CCR组配置：");
    println!();

    for (name, profile, is_default) in profiles {
        let default_marker = if is_default { " (默认)" } else { "" };
        println!("🚀 {name}{default_marker}");
        println!("   🔗 提供商数量: {}", profile.providers.len());
        println!("   🎯 默认路由: {}", profile.router.default);

        if !profile.providers.is_empty() {
            println!("   📊 提供商:");
            for provider in &profile.providers {
                println!("     • {}: {} 个模型", provider.name, provider.models.len());
            }
        }

        if let Some(timeout) = profile.api_timeout_ms {
            println!("   ⏱️  超时: {}ms", timeout);
        }

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

/// 添加CCR配置（交互式）
pub fn cmd_add_ccr(name: String) -> AppResult<()> {
    let mut config = Config::load().unwrap_or_default();

    // 检查配置是否已存在
    if config.groups.ccr.contains_key(&name) {
        return Err(AppError::Config(format!("CCR配置 '{name}' 已存在")));
    }

    println!("🚀 添加新CCR配置: {name}");
    println!();

    // 简化的CCR配置创建 - 提供几个常用模板
    println!("📋 选择CCR配置模板:");
    println!("  1) DeepSeek (推荐)");
    println!("  2) OpenRouter");
    println!("  3) 自定义配置");

    print!("请选择 [1-3]: ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    let ccr_profile = match choice {
        "1" => create_deepseek_template()?,
        "2" => create_openrouter_template()?,
        "3" => create_custom_ccr_profile()?,
        _ => {
            println!("❌ 无效选择，默认使用DeepSeek模板");
            create_deepseek_template()?
        }
    };

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

    let mut final_profile = ccr_profile;
    final_profile.description = description;
    final_profile.created_at = Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());

    // 添加并保存配置
    config.add_ccr_profile(name.clone(), final_profile)?;
    config.save()?;

    println!();
    println!("✅ CCR配置 '{name}' 添加成功！");

    if config.groups.ccr.len() == 1 {
        println!("🎯 已自动设为默认CCR配置");
    }

    Ok(())
}

/// 设置默认CCR配置
pub fn cmd_use_ccr(name: String) -> AppResult<()> {
    let mut config = Config::load()?;
    config.set_default_ccr(&name)?;
    config.save()?;
    println!("✅ 已将 '{name}' 设为默认CCR配置");
    Ok(())
}

/// 运行CCR配置
pub fn cmd_run_ccr(name: Option<String>) -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let config = Config::load()?;

        let (profile_name, profile) = match name {
            Some(name) => {
                let profile = config.get_ccr_profile(&name)?;
                (name, profile)
            }
            None => {
                let (default_name, profile) = config.get_default_ccr_profile()?;
                (default_name.clone(), profile)
            }
        };

        println!("🚀 使用CCR配置 '{profile_name}' 启动 claude...");
        println!("🔗 提供商数量: {}", profile.providers.len());
        println!("🎯 默认路由: {}", profile.router.default);
        println!();

        // 创建CCR管理器
        let mut manager = CcrManager::new()?;

        // 生成CCR配置文件
        println!("📄 生成CCR配置文件...");
        manager.generate_ccr_config(profile)?;

        // 检查并启动CCR服务
        println!("📡 检查CCR服务状态...");
        if !manager.is_service_running().await? {
            println!("🚀 启动CCR服务...");
            manager.start_service().await?;
        } else {
            println!("✅ CCR服务已在运行");
        }

        // 启动claude程序，通过CCR代理
        println!("🎯 启动claude程序...");
        let mut cmd = Command::new("claude");
        cmd.env("ANTHROPIC_BASE_URL", "http://localhost:3456");
        cmd.env("ANTHROPIC_AUTH_TOKEN", "any-string-is-ok"); // CCR会处理认证

        match cmd.status() {
            Ok(status) => {
                if status.success() {
                    println!("✅ claude 程序正常退出");
                } else {
                    println!("⚠️  claude 程序异常退出，退出码: {:?}", status.code());
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(AppError::CommandExecution(
                        "找不到 'claude' 程序，请确保 claude 已安装并在 PATH 中".to_string(),
                    ));
                } else {
                    return Err(AppError::CommandExecution(format!("执行 claude 失败: {e}")));
                }
            }
        }

        Ok::<(), crate::error::AppError>(())
    })?;

    Ok(())
}

/// 删除CCR配置
pub fn cmd_remove_ccr(name: String) -> AppResult<()> {
    let mut config = Config::load()?;

    // 确认删除
    print!("⚠️  确定要删除CCR配置 '{name}' 吗？(y/N): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("❌ 取消删除");
        return Ok(());
    }

    config.remove_ccr_profile(&name)?;
    config.save()?;

    println!("✅ CCR配置 '{name}' 已删除");

    // 显示当前默认配置
    if !config.groups.ccr.is_empty() {
        if let Some(default_profile) = &config.default_profile {
            if let Some(ccr) = &default_profile.ccr {
                println!("🎯 当前默认CCR配置: {ccr}");
            }
        }
    } else {
        println!("📋 暂无CCR配置，请使用 'ccode add-ccr <name>' 添加配置");
    }

    Ok(())
}

// ==================== CCR服务管理命令 ====================

/// 启动CCR服务
pub fn cmd_ccr_start() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut manager = CcrManager::new()?;
        manager.start_service().await
    })
}

/// 停止CCR服务
pub fn cmd_ccr_stop() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut manager = CcrManager::new()?;
        manager.stop_service().await
    })
}

/// 重启CCR服务
pub fn cmd_ccr_restart() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut manager = CcrManager::new()?;
        manager.restart_service().await
    })
}

/// 查看CCR服务状态
pub fn cmd_ccr_status() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let manager = CcrManager::new()?;
        let status = manager.get_service_status().await?;

        println!("📊 CCR服务状态:");
        print!("{}", status.format_status());

        Ok::<(), crate::error::AppError>(())
    })?;
    Ok(())
}

/// 查看CCR服务日志
pub fn cmd_ccr_logs() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let manager = CcrManager::new()?;
        let logs = manager.get_service_logs().await?;

        println!("📋 CCR服务日志:");
        println!("{}", logs);

        Ok::<(), crate::error::AppError>(())
    })?;
    Ok(())
}

// ==================== CCR配置模板 ====================

/// 创建DeepSeek模板
fn create_deepseek_template() -> AppResult<CcrProfile> {
    print!("🔑 请输入 DeepSeek API Key: ");
    io::stdout().flush().unwrap();
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    let profile = CcrProfile {
        providers: vec![CcrProvider {
            name: "deepseek".to_string(),
            api_base_url: "https://api.deepseek.com/chat/completions".to_string(),
            api_key,
            models: vec!["deepseek-chat".to_string(), "deepseek-reasoner".to_string()],
            transformer: Some(json!({"use": ["deepseek"]})),
        }],
        router: CcrRouter {
            default: "deepseek,deepseek-chat".to_string(),
            background: Some("deepseek,deepseek-chat".to_string()),
            think: Some("deepseek,deepseek-reasoner".to_string()),
            long_context: None,
            long_context_threshold: Some(60000),
            web_search: None,
        },
        api_timeout_ms: Some(600000),
        proxy_url: None,
        log: Some(true),
        api_key: None,
        host: None,
        description: None,
        created_at: None,
    };

    Ok(profile)
}

/// 创建OpenRouter模板
fn create_openrouter_template() -> AppResult<CcrProfile> {
    print!("🔑 请输入 OpenRouter API Key: ");
    io::stdout().flush().unwrap();
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    let profile = CcrProfile {
        providers: vec![CcrProvider {
            name: "openrouter".to_string(),
            api_base_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            api_key,
            models: vec![
                "anthropic/claude-3.5-sonnet".to_string(),
                "google/gemini-2.5-pro-preview".to_string(),
                "anthropic/claude-sonnet-4".to_string(),
            ],
            transformer: Some(json!({"use": ["openrouter"]})),
        }],
        router: CcrRouter {
            default: "openrouter,anthropic/claude-3.5-sonnet".to_string(),
            background: Some("openrouter,anthropic/claude-3.5-sonnet".to_string()),
            think: None,
            long_context: Some("openrouter,google/gemini-2.5-pro-preview".to_string()),
            long_context_threshold: Some(60000),
            web_search: None,
        },
        api_timeout_ms: Some(600000),
        proxy_url: None,
        log: Some(true),
        api_key: None,
        host: None,
        description: None,
        created_at: None,
    };

    Ok(profile)
}

/// 创建自定义CCR配置
fn create_custom_ccr_profile() -> AppResult<CcrProfile> {
    println!("⚠️  自定义配置功能还未完全实现");
    println!("💡 建议先选择预设模板，然后手动编辑配置文件");

    // 回退到DeepSeek模板
    create_deepseek_template()
}
