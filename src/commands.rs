use crate::ccr_manager::CcrManager;
use crate::config::{CcrProfile, Config, Profile, ProviderType};
use crate::error::{AppError, AppResult};
use chrono::Utc;
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
            println!("   ⏱️  超时: {timeout}ms");
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

/// 添加CCR配置（交互式，单provider模式）
pub fn cmd_add_ccr(name: String) -> AppResult<()> {
    let mut config = Config::load().unwrap_or_default();

    // 检查配置是否已存在
    if config.groups.ccr.contains_key(&name) {
        return Err(AppError::Config(format!("CCR配置 '{name}' 已存在")));
    }

    println!("🚀 添加新CCR配置: {name}");
    println!();

    // 选择provider类型
    println!("📋 选择Provider类型:");
    let provider_types = [
        ProviderType::OpenAI,
        ProviderType::OpenRouter,
        ProviderType::DeepSeek,
        ProviderType::Gemini,
        ProviderType::Qwen,
        ProviderType::Custom,
    ];

    for (index, provider_type) in provider_types.iter().enumerate() {
        println!(
            "  {}) {} ({})",
            index + 1,
            provider_type.display_name(),
            provider_type.url_format_hint()
        );
    }

    print!("请选择 [1-6]: ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    let provider_type = match choice {
        "1" => ProviderType::OpenAI,
        "2" => ProviderType::OpenRouter,
        "3" => ProviderType::DeepSeek,
        "4" => ProviderType::Gemini,
        "5" => ProviderType::Qwen,
        "6" => ProviderType::Custom,
        _ => {
            println!("❌ 无效选择，默认使用OpenAI兼容类型");
            ProviderType::OpenAI
        }
    };

    println!();
    println!("🔧 配置 {} 类型的Provider:", provider_type.display_name());

    // 显示配置提示
    for hint in provider_type.get_configuration_hints() {
        println!("  {hint}");
    }
    println!();

    // 获取Provider名称
    print!("📝 请输入Provider名称 (默认: {name}): ");
    io::stdout().flush().unwrap();
    let mut provider_name = String::new();
    io::stdin().read_line(&mut provider_name)?;
    let provider_name = provider_name.trim();
    let provider_name = if provider_name.is_empty() {
        name.clone()
    } else {
        provider_name.to_string()
    };

    // 获取API密钥
    print!("🔑 请输入API Key: ");
    io::stdout().flush().unwrap();
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    // 获取API URL（可选）
    println!("📍 API URL配置:");
    println!("  默认: {}", provider_type.url_format_hint());
    print!("  自定义URL (直接回车使用默认): ");
    io::stdout().flush().unwrap();
    let mut api_url = String::new();
    io::stdin().read_line(&mut api_url)?;
    let api_url = api_url.trim();
    let custom_url = if api_url.is_empty() {
        None
    } else {
        Some(api_url.to_string())
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

    println!();
    println!("🔧 正在创建CCR配置...");

    // 使用模板创建CCR配置
    match CcrProfile::create_template(
        provider_type.clone(),
        provider_name.clone(),
        api_key,
        custom_url,
        description,
    ) {
        Ok(mut ccr_profile) => {
            // 设置创建时间
            ccr_profile.created_at = Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());

            // 添加并保存配置
            config.add_ccr_profile(name.clone(), ccr_profile)?;
            config.save()?;

            println!();
            println!("✅ CCR配置 '{name}' 添加成功！");
            println!(
                "🔗 Provider: {} ({})",
                provider_name,
                provider_type.display_name()
            );

            if config.groups.ccr.len() == 1 {
                println!("🎯 已自动设为默认CCR配置");
            }

            // 询问是否立即生成CCR配置文件
            print!("📄 是否立即生成claude-code-router配置文件? (y/N): ");
            io::stdout().flush().unwrap();
            let mut generate_config = String::new();
            io::stdin().read_line(&mut generate_config)?;

            if generate_config.trim().to_lowercase() == "y" {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    let manager = CcrManager::new()?;
                    if let Ok(profile) = config.get_ccr_profile(&name) {
                        manager.generate_ccr_config(profile)?;
                        println!("✅ claude-code-router配置文件已生成");
                    }
                    Ok::<(), AppError>(())
                })?;
            }
        }
        Err(e) => {
            return Err(AppError::Config(format!("创建CCR配置失败: {e}")));
        }
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

/// 运行CCR配置（支持智能配置检测）
pub fn cmd_run_ccr(name: Option<String>) -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut config = Config::load().unwrap_or_default();

        // 智能配置检测：如果CCR配置为空，尝试从claude-code-router导入
        if config.groups.ccr.is_empty() {
            let manager = CcrManager::new()?;
            let is_ccr_config_empty = manager.is_ccr_config_empty().await?;

            if !is_ccr_config_empty {
                println!("🔍 检测到ccode CCR配置为空，但claude-code-router配置文件存在");
                print!("📥 是否自动导入claude-code-router配置？(y/N): ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() == "y" {
                    println!("📥 正在导入配置...");
                    match manager.import_from_ccr_config().await? {
                        Some(message) => {
                            println!("✅ {message}");
                            // 重新加载配置
                            config = Config::load()?;
                        }
                        None => {
                            println!("⚠️  导入失败或配置为空");
                        }
                    }
                }
            }
        }

        // 如果仍然没有CCR配置，提示用户
        if config.groups.ccr.is_empty() {
            println!("❌ 暂无CCR配置，请使用 'ccode add-ccr <name>' 添加配置");
            return Ok(());
        }

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

        // 显示Provider信息
        if let Some(provider) = profile.get_primary_provider() {
            println!(
                "🔗 Provider: {} ({})",
                provider.name,
                provider
                    .provider_type
                    .as_ref()
                    .map_or("未知类型", |t| t.display_name())
            );
            println!("📊 模型数量: {}", provider.models.len());
        } else {
            println!("🔗 提供商数量: {}", profile.providers.len());
        }

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
        println!("{logs}");

        Ok::<(), crate::error::AppError>(())
    })?;
    Ok(())
}

// ==================== Router配置管理命令 ====================

/// 显示CCR配置的Router设置
#[allow(dead_code)]
pub fn cmd_ccr_router_show(name: Option<String>) -> AppResult<()> {
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

    println!("🎯 CCR配置 '{profile_name}' 的Router设置:");
    println!();

    // 显示所有路由配置
    let routes = profile.router.get_all_routes();
    for (route_name, route_value) in routes {
        let icon = match route_name.as_str() {
            "default" => "🎯",
            "background" => "🔄",
            "think" => "💭",
            "longContext" => "📜",
            "webSearch" => "🔍",
            _ => "📌",
        };
        println!("  {icon} {route_name}: {route_value}");
    }

    // 显示长上下文阈值
    if let Some(threshold) = profile.router.long_context_threshold {
        println!("  ⚖️  longContextThreshold: {threshold}");
    }

    Ok(())
}

/// 设置CCR配置的Router选项
#[allow(dead_code)]
pub fn cmd_ccr_router_set(name: String, route_type: String, route_value: String) -> AppResult<()> {
    let mut config = Config::load()?;

    // 获取CCR配置
    let profile = config.get_ccr_profile(&name)?;
    let mut updated_profile = profile.clone();

    // 验证路由值格式
    if !route_value.is_empty() && !route_value.contains(',') {
        return Err(AppError::Config(
            "路由值格式无效，应为'provider,model'格式".to_string(),
        ));
    }

    // 设置路由配置
    match route_type.as_str() {
        "default" => {
            updated_profile.router.default = route_value;
        }
        "background" => {
            updated_profile.router.background = if route_value.is_empty() {
                None
            } else {
                Some(route_value)
            };
        }
        "think" => {
            updated_profile.router.think = if route_value.is_empty() {
                None
            } else {
                Some(route_value)
            };
        }
        "longContext" => {
            updated_profile.router.long_context = if route_value.is_empty() {
                None
            } else {
                Some(route_value)
            };
        }
        "webSearch" => {
            updated_profile.router.web_search = if route_value.is_empty() {
                None
            } else {
                Some(route_value)
            };
        }
        _ => {
            return Err(AppError::Config(format!(
                "未知的路由类型: {route_type}。支持的类型: default, background, think, longContext, webSearch"
            )));
        }
    }

    // 验证更新后的配置
    updated_profile.validate()?;

    // 更新配置
    config.groups.ccr.insert(name.clone(), updated_profile);
    config.save()?;

    println!("✅ 已更新CCR配置 '{name}' 的 {route_type} 路由设置");

    Ok(())
}

/// 设置长上下文阈值
#[allow(dead_code)]
pub fn cmd_ccr_router_set_threshold(name: String, threshold: u32) -> AppResult<()> {
    let mut config = Config::load()?;

    // 获取CCR配置
    let profile = config.get_ccr_profile(&name)?;
    let mut updated_profile = profile.clone();

    // 设置阈值
    updated_profile.router.long_context_threshold = Some(threshold);

    // 验证更新后的配置
    updated_profile.validate()?;

    // 更新配置
    config.groups.ccr.insert(name.clone(), updated_profile);
    config.save()?;

    println!("✅ 已设置CCR配置 '{name}' 的长上下文阈值为: {threshold}");

    Ok(())
}

/// 重置CCR配置的Router设置为默认值
#[allow(dead_code)]
pub fn cmd_ccr_router_reset(name: String) -> AppResult<()> {
    let mut config = Config::load()?;

    // 获取CCR配置
    let profile = config.get_ccr_profile(&name)?;
    let mut updated_profile = profile.clone();

    // 重置路由设置
    updated_profile.router.apply_defaults();

    // 验证更新后的配置
    updated_profile.validate()?;

    // 更新配置
    config.groups.ccr.insert(name.clone(), updated_profile);
    config.save()?;

    println!("✅ 已重置CCR配置 '{name}' 的Router设置为默认值");

    Ok(())
}

/// 交互式Router配置设置
#[allow(dead_code)]
pub fn cmd_ccr_router_config(name: String) -> AppResult<()> {
    let mut config = Config::load()?;

    // 获取CCR配置
    let profile = config.get_ccr_profile(&name)?;
    let mut updated_profile = profile.clone();

    println!("🎯 配置CCR '{name}' 的Router设置");
    println!();

    // 显示当前Provider信息
    if let Some(provider) = updated_profile.get_primary_provider() {
        println!("📊 当前Provider信息:");
        println!("  名称: {}", provider.name);
        println!("  模型: {}", provider.models.join(", "));
        println!();
    }

    // 交互式设置各路由
    let route_configs = [
        ("default", "🎯 默认路由", true),
        ("background", "🔄 后台任务路由", false),
        ("think", "💭 思考任务路由", false),
        ("longContext", "📜 长上下文路由", false),
        ("webSearch", "🔍 网络搜索路由", false),
    ];

    for (route_key, route_desc, is_required) in route_configs.iter() {
        let current_value = match *route_key {
            "default" => Some(updated_profile.router.default.clone()),
            "background" => updated_profile.router.background.clone(),
            "think" => updated_profile.router.think.clone(),
            "longContext" => updated_profile.router.long_context.clone(),
            "webSearch" => updated_profile.router.web_search.clone(),
            _ => None,
        };

        let current_display = current_value.unwrap_or_else(|| "未设置".to_string());

        if *is_required {
            print!("{route_desc} (当前: {current_display}): ");
        } else {
            print!("{route_desc} (当前: {current_display}, 直接回车跳过): ");
        }

        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if !input.is_empty() {
            // 验证格式
            if !input.contains(',') {
                println!("⚠️  路由格式应为'provider,model'，跳过此设置");
                continue;
            }

            // 设置路由值
            match *route_key {
                "default" => updated_profile.router.default = input.to_string(),
                "background" => updated_profile.router.background = Some(input.to_string()),
                "think" => updated_profile.router.think = Some(input.to_string()),
                "longContext" => updated_profile.router.long_context = Some(input.to_string()),
                "webSearch" => updated_profile.router.web_search = Some(input.to_string()),
                _ => {}
            }
        }
    }

    // 设置长上下文阈值
    let current_threshold = updated_profile
        .router
        .long_context_threshold
        .unwrap_or(60000);
    print!("⚖️  长上下文阈值 (当前: {current_threshold}, 直接回车跳过): ");
    io::stdout().flush().unwrap();
    let mut threshold_input = String::new();
    io::stdin().read_line(&mut threshold_input)?;
    let threshold_input = threshold_input.trim();

    if !threshold_input.is_empty() {
        match threshold_input.parse::<u32>() {
            Ok(threshold) => {
                updated_profile.router.long_context_threshold = Some(threshold);
            }
            Err(_) => {
                println!("⚠️  无效的阈值格式，保持原值");
            }
        }
    }

    // 验证配置
    match updated_profile.validate() {
        Ok(_) => {
            // 更新配置
            config.groups.ccr.insert(name.clone(), updated_profile);
            config.save()?;

            println!();
            println!("✅ Router配置已更新成功！");

            // 询问是否重新生成CCR配置文件
            print!("📄 是否重新生成claude-code-router配置文件? (y/N): ");
            io::stdout().flush().unwrap();
            let mut generate_config = String::new();
            io::stdin().read_line(&mut generate_config)?;

            if generate_config.trim().to_lowercase() == "y" {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    let manager = CcrManager::new()?;
                    if let Ok(profile) = config.get_ccr_profile(&name) {
                        manager.generate_ccr_config(profile)?;
                        println!("✅ claude-code-router配置文件已重新生成");
                    }
                    Ok::<(), AppError>(())
                })?;
            }
        }
        Err(e) => {
            println!("❌ 配置验证失败: {e}");
            println!("💡 请检查路由配置是否正确");
        }
    }

    Ok(())
}

// ==================== 配置导入和备份管理命令 ====================

/// 从claude-code-router配置文件导入CCR配置
#[allow(dead_code)]
pub fn cmd_ccr_import() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let manager = CcrManager::new()?;

        // 检查CCR配置是否为空
        let is_empty = manager.is_ccr_config_empty().await?;

        if !is_empty {
            println!("⚠️  ccode CCR配置不为空，请手动进行配置迁移");
            println!("💡 如需强制导入，请先删除现有CCR配置");
            return Ok(());
        }

        println!("📥 正在从claude-code-router配置文件导入...");

        match manager.import_from_ccr_config().await? {
            Some(message) => {
                println!("✅ {message}");
                println!("💡 已将claude-code-router中的每个provider创建为独立的CCR配置");
            }
            None => {
                println!("ℹ️  未找到有效的claude-code-router配置或配置为空");
            }
        }

        Ok::<(), crate::error::AppError>(())
    })?;

    Ok(())
}

/// 列出CCR配置文件备份
#[allow(dead_code)]
pub fn cmd_ccr_backup_list() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let manager = CcrManager::new()?;
        let backups = manager.list_backups()?;

        if backups.is_empty() {
            println!("📋 暂无备份文件");
            return Ok(());
        }

        println!("📋 CCR配置文件备份列表:");
        println!();

        for (index, backup) in backups.iter().enumerate() {
            // 从文件名提取时间戳
            if let Some(timestamp_part) = backup
                .strip_prefix("config_backup_")
                .and_then(|s| s.strip_suffix(".json"))
            {
                // 解析时间戳格式: YYYYMMDD_HHMMSS
                if timestamp_part.len() == 15 {
                    let date_part = &timestamp_part[..8];
                    let time_part = &timestamp_part[9..];

                    if let (Ok(year), Ok(month), Ok(day)) = (
                        date_part[..4].parse::<u32>(),
                        date_part[4..6].parse::<u32>(),
                        date_part[6..8].parse::<u32>(),
                    ) {
                        if let (Ok(hour), Ok(minute), Ok(second)) = (
                            time_part[..2].parse::<u32>(),
                            time_part[2..4].parse::<u32>(),
                            time_part[4..6].parse::<u32>(),
                        ) {
                            println!(
                                "  {}) {} ({}-{:02}-{:02} {:02}:{:02}:{:02})",
                                index + 1,
                                backup,
                                year,
                                month,
                                day,
                                hour,
                                minute,
                                second
                            );
                            continue;
                        }
                    }
                }
            }

            // 如果时间戳解析失败，就显示原文件名
            println!("  {}) {}", index + 1, backup);
        }

        Ok::<(), crate::error::AppError>(())
    })?;

    Ok(())
}

/// 创建CCR配置文件备份
#[allow(dead_code)]
pub fn cmd_ccr_backup_create() -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let manager = CcrManager::new()?;

        match manager.create_backup() {
            Ok(backup_filename) => {
                println!("✅ 备份创建成功: {backup_filename}");
            }
            Err(e) => {
                println!("❌ 备份创建失败: {e}");
            }
        }

        Ok::<(), crate::error::AppError>(())
    })?;

    Ok(())
}

/// 从备份恢复CCR配置文件
#[allow(dead_code)]
pub fn cmd_ccr_backup_restore(backup_filename: String) -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let manager = CcrManager::new()?;

        // 先列出可用的备份让用户确认
        let backups = manager.list_backups()?;

        if !backups.contains(&backup_filename) {
            println!("❌ 指定的备份文件不存在: {backup_filename}");
            println!("💡 使用 'ccode ccr backup list' 查看可用备份");
            return Ok(());
        }

        // 确认恢复操作
        print!("⚠️  确定要从备份 '{backup_filename}' 恢复配置吗？当前配置将被覆盖。(y/N): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("❌ 取消恢复");
            return Ok(());
        }

        match manager.restore_from_backup(&backup_filename) {
            Ok(_) => {
                println!("✅ 配置恢复成功");
                println!("💡 如果CCR服务正在运行，建议重启服务使配置生效");
            }
            Err(e) => {
                println!("❌ 配置恢复失败: {e}");
            }
        }

        Ok::<(), crate::error::AppError>(())
    })?;

    Ok(())
}

/// 删除CCR配置文件备份
#[allow(dead_code)]
pub fn cmd_ccr_backup_delete(backup_filename: String) -> AppResult<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let manager = CcrManager::new()?;

        // 先检查备份是否存在
        let backups = manager.list_backups()?;

        if !backups.contains(&backup_filename) {
            println!("❌ 指定的备份文件不存在: {backup_filename}");
            println!("💡 使用 'ccode ccr backup list' 查看可用备份");
            return Ok(());
        }

        // 确认删除操作
        print!("⚠️  确定要删除备份 '{backup_filename}' 吗？此操作不可恢复。(y/N): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("❌ 取消删除");
            return Ok(());
        }

        match manager.delete_backup(&backup_filename) {
            Ok(_) => {
                println!("✅ 备份删除成功");
            }
            Err(e) => {
                println!("❌ 备份删除失败: {e}");
            }
        }

        Ok::<(), crate::error::AppError>(())
    })?;

    Ok(())
}

/// 清理旧的CCR配置文件备份
#[allow(dead_code)]
pub fn cmd_ccr_backup_cleanup(keep_count: Option<usize>) -> AppResult<()> {
    let keep_count = keep_count.unwrap_or(5); // 默认保留5个备份

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let manager = CcrManager::new()?;

        match manager.cleanup_old_backups(keep_count) {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    println!("✅ 已清理 {deleted_count} 个旧备份文件，保留最新的 {keep_count} 个");
                } else {
                    println!("ℹ️  无需清理，当前备份数量未超过保留限制 ({keep_count})");
                }
            }
            Err(e) => {
                println!("❌ 备份清理失败: {e}");
            }
        }

        Ok::<(), crate::error::AppError>(())
    })?;

    Ok(())
}
