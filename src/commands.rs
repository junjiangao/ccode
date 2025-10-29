use crate::error::{AppError, AppResult};
use crate::migrate;
use crate::toml_config::{TomlConfig, TomlProfile, load_token_from_env};
use std::io::{self, Write};
use std::process::Command;

/// 读取可选字符串输入的通用函数
fn read_optional_input(prompt: &str) -> AppResult<Option<String>> {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    Ok(if input.is_empty() {
        None
    } else {
        Some(input.to_string())
    })
}

// 已移除：CCR 路由类型智能推荐（依赖 CCR Provider/Router）
/*
fn get_route_recommendations(
    route_key: &str,
    providers: &[CcrProvider],
) -> Vec<(String, &'static str)> {
    let mut recommendations = Vec::new();

    match route_key {
        "background" => {
            // 后台任务推荐快速、经济的模型
            for provider in providers {
                if let Some(provider_type) = &provider.provider_type {
                    match provider_type {
                        ProviderType::OpenAI => {
                            if let Some(model) = provider
                                .models
                                .iter()
                                .find(|m| m.contains("gpt-3.5") || m.contains("4o-mini"))
                            {
                                recommendations
                                    .push((format!("{},{}", provider.name, model), "🚀 快速响应"));
                            }
                        }
                        ProviderType::DeepSeek => {
                            if let Some(model) = provider.models.first() {
                                recommendations
                                    .push((format!("{},{}", provider.name, model), "💰 高性价比"));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        "think" => {
            // 思考任务推荐推理能力强的模型
            for provider in providers {
                if let Some(provider_type) = &provider.provider_type {
                    match provider_type {
                        ProviderType::DeepSeek => {
                            if let Some(model) =
                                provider.models.iter().find(|m| m.contains("reasoner"))
                            {
                                recommendations
                                    .push((format!("{},{}", provider.name, model), "🧠 强大推理"));
                            }
                        }
                        ProviderType::Qwen => {
                            if let Some(model) = provider
                                .models
                                .iter()
                                .find(|m| m.contains("Thinking") || m.contains("thinking"))
                            {
                                recommendations.push((
                                    format!("{},{}", provider.name, model),
                                    "🤔 思维链推理",
                                ));
                            }
                        }
                        ProviderType::OpenRouter => {
                            if let Some(model) = provider
                                .models
                                .iter()
                                .find(|m| m.contains("claude") || m.contains("o1"))
                            {
                                recommendations
                                    .push((format!("{},{}", provider.name, model), "🔬 逻辑分析"));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        "longContext" => {
            // 长上下文推荐支持大输入的模型
            for provider in providers {
                if let Some(provider_type) = &provider.provider_type {
                    match provider_type {
                        ProviderType::Qwen => {
                            if let Some(model) = provider.models.first() {
                                recommendations.push((
                                    format!("{},{}", provider.name, model),
                                    "📜 超长上下文",
                                ));
                            }
                        }
                        ProviderType::Gemini => {
                            if let Some(model) = provider.models.iter().find(|m| m.contains("pro"))
                            {
                                recommendations.push((
                                    format!("{},{}", provider.name, model),
                                    "🌐 海量信息处理",
                                ));
                            }
                        }
                        ProviderType::OpenRouter => {
                            if let Some(model) =
                                provider.models.iter().find(|m| m.contains("claude"))
                            {
                                recommendations.push((
                                    format!("{},{}", provider.name, model),
                                    "📖 文档分析专家",
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        "webSearch" => {
            // 网络搜索推荐支持联网的模型
            for provider in providers {
                if let Some(provider_type) = &provider.provider_type {
                    match provider_type {
                        ProviderType::OpenRouter => {
                            if let Some(model) = provider.models.first() {
                                let route_with_online =
                                    format!("{},{}:online", provider.name, model);
                                recommendations.push((route_with_online, "🔍 实时搜索"));
                            }
                        }
                        _ => {
                            if let Some(model) = provider.models.first() {
                                recommendations.push((
                                    format!("{},{}", provider.name, model),
                                    "🌐 基础网络查询",
                                ));
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    // 限制推荐数量避免过多选项
    recommendations.truncate(3);
    recommendations
}
*/

// 旧 JSON 模式命令已移除

// 旧 JSON 模式删除命令已移除

/// 列出配置（统一接口）
pub fn cmd_list_with_group(_group: Option<String>) -> AppResult<()> {
    cmd_list_toml()
}

/// 添加配置（统一接口）
pub fn cmd_add_with_group(name: String, _group: Option<String>) -> AppResult<()> {
    cmd_add_toml(name)
}

/// 设置默认配置（统一接口）
pub fn cmd_use_with_group(name: String, _group: Option<String>) -> AppResult<()> {
    cmd_use_toml(name)
}

/// 运行配置（统一接口）
pub fn cmd_run_with_group(
    name: Option<String>,
    _group: Option<String>,
    claude_args: Vec<String>,
) -> AppResult<()> {
    cmd_run_toml(name, claude_args)
}

/// 删除配置（统一接口）
pub fn cmd_remove_with_group(name: String, _group: Option<String>) -> AppResult<()> {
    cmd_remove_toml(name)
}

/// 主动触发 JSON→TOML 迁移/合并
pub fn cmd_config_merge() -> AppResult<()> {
    let report = migrate::manual_merge()?;

    println!(
        "✅ 迁移完成：共 {}，迁移 {}，跳过 {}",
        report.profiles_total, report.profiles_migrated, report.profiles_skipped
    );
    if report.created_toml {
        println!("🆕 已创建 config.toml");
    }
    if report.merged_into_existing {
        println!("🔗 已合并到现有 config.toml（同名跳过）");
    }
    if let Some(d) = report.default_set {
        println!("🎯 默认配置: {}", d);
    }
    println!("🗄️  已备份旧 JSON: {}", report.backup_path.display());
    println!("🧹 已移除 config.json");
    Ok(())
}

/// 使用 TOML 新格式运行 claude
pub fn cmd_run_toml(name: Option<String>, claude_args: Vec<String>) -> AppResult<()> {
    let config = TomlConfig::load()?;
    let toml_path = TomlConfig::get_config_path()?;
    let (profile_name, profile) = config.get_profile(name.as_deref())?;

    // 读取 token：优先同目录 .env，其次系统环境
    let token = load_token_from_env(&toml_path, &profile.env_key)?;

    println!("🚀 使用 TOML 配置 '{profile_name}' 启动 claude...");
    println!("📍 API URL: {}", profile.base_url);
    if let Some(m) = &profile.model {
        println!("🤖 默认模型: {}", m);
    }
    if profile.model_haiku.is_some()
        || profile.model_sonnet.is_some()
        || profile.model_opus.is_some()
    {
        println!(
            "🧩 家族模型: {}{}{}",
            profile.model_haiku.as_deref().unwrap_or("-"),
            if profile.model_sonnet.is_some() {
                " | "
            } else {
                ""
            },
            profile.model_sonnet.as_deref().unwrap_or("")
        );
    }

    let mut cmd = Command::new("claude");
    // 必填环境变量
    cmd.env("ANTHROPIC_AUTH_TOKEN", &token);
    cmd.env("ANTHROPIC_BASE_URL", &profile.base_url);

    // 新映射的模型变量
    if let Some(m) = &profile.model {
        cmd.env("ANTHROPIC_MODEL", m);
    }
    if let Some(m) = &profile.model_haiku {
        // 新变量
        cmd.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", m);
        // 兼容变量（已弃用，但为向后兼容继续设置）
        cmd.env("ANTHROPIC_SMALL_FAST_MODEL", m);
    }
    if let Some(m) = &profile.model_sonnet {
        cmd.env("ANTHROPIC_DEFAULT_SONNET_MODEL", m);
    }
    if let Some(m) = &profile.model_opus {
        cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", m);
    }
    if let Some(max) = &profile.max_tokens {
        cmd.env("CLAUDE_CODE_MAX_OUTPUT_TOKENS", max);
    }

    if !claude_args.is_empty() {
        cmd.args(&claude_args);
        println!("📄 透传参数: {}", claude_args.join(" "));
    }

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

/// 列出 TOML 配置
pub fn cmd_list_toml() -> AppResult<()> {
    let config = match TomlConfig::load() {
        Ok(c) => c,
        Err(AppError::ConfigNotFound) => {
            println!("📋 暂无配置，请创建 ~/.config/ccode/config.toml");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    if config.profiles.is_empty() {
        println!("📋 暂无配置，请编辑 ~/.config/ccode/config.toml");
        return Ok(());
    }

    let default_name = config.default.as_deref();
    println!("📋 TOML 配置：");
    println!();
    for (name, p) in &config.profiles {
        let default_marker = if Some(name.as_str()) == default_name {
            " (默认)"
        } else {
            ""
        };
        println!("🔧 {name}{default_marker}");
        println!("   📍 base_url: {}", p.base_url);
        println!("   🔑 env_key: {}", p.env_key);
        if let Some(m) = &p.model {
            println!("   🤖 model: {}", m);
        }
        if let Some(m) = &p.model_haiku {
            println!("   🐦 haiku: {}", m);
        }
        if let Some(m) = &p.model_sonnet {
            println!("   🎼 sonnet: {}", m);
        }
        if let Some(m) = &p.model_opus {
            println!("   🎻 opus: {}", m);
        }
        if let Some(mt) = &p.max_tokens {
            println!("   📦 max_tokens: {}", mt);
        }
        if let Some(c) = &p.comment {
            println!("   📝 说明: {}", c);
        }
        println!();
    }
    Ok(())
}

/// 添加 TOML 配置（交互式）
pub fn cmd_add_toml(name: String) -> AppResult<()> {
    let mut config = TomlConfig::load_or_default()?;
    if config.profiles.contains_key(&name) {
        return Err(AppError::Config(format!("配置 '{name}' 已存在")));
    }

    println!("🔧 添加 TOML 配置: {name}");
    println!();

    // base_url（使用环境变量名提示）
    print!("📍 请输入 ANTHROPIC_BASE_URL (如: https://api.anthropic.com): ");
    io::stdout().flush().unwrap();
    let mut base_url = String::new();
    io::stdin().read_line(&mut base_url)?;
    let base_url = base_url.trim().to_string();

    // 令牌输入：直接输入 Key，程序将写入 .env 中自动生成的 name_key 变量
    let env_key = crate::toml_config::derive_env_key_from_profile(&name);
    print!("🔑 请输入 ANTHROPIC_AUTH_TOKEN（将保存为 .env 的 {env_key}）: ");
    io::stdout().flush().unwrap();
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let token_input = token_input.trim().to_string();

    // 可选项
    let model = read_optional_input("🤖 请输入 ANTHROPIC_MODEL (可选): ")?;
    let model_haiku = read_optional_input("🐦 请输入 ANTHROPIC_DEFAULT_HAIKU_MODEL (可选): ")?;
    let model_sonnet = read_optional_input("🎼 请输入 ANTHROPIC_DEFAULT_SONNET_MODEL (可选): ")?;
    let model_opus = read_optional_input("🎻 请输入 ANTHROPIC_DEFAULT_OPUS_MODEL (可选): ")?;
    let max_tokens =
        read_optional_input("📦 请输入 CLAUDE_CODE_MAX_OUTPUT_TOKENS (可选，如 32000): ")?;
    let comment = read_optional_input("📝 请输入 comment (可选): ")?;

    let profile = TomlProfile {
        name: Some(name.clone()),
        base_url,
        env_key: env_key.clone(),
        model,
        model_haiku,
        model_sonnet,
        model_opus,
        max_tokens,
        comment,
    };

    TomlConfig::validate_profile(&profile)?;
    config.add_profile(&name, profile)?;
    config.save()?;

    // 将 Key 写入 ~/.config/ccode/.env 的 {env_key}=... 中
    let toml_path = TomlConfig::get_config_path()?;
    crate::toml_config::persist_token_to_env(&toml_path, &env_key, &token_input)?;

    println!("✅ 配置 '{name}' 已添加");
    if config.default.as_deref() == Some(&name) {
        println!("🎯 已设为默认配置");
    }
    Ok(())
}

/// 设置 TOML 默认配置
pub fn cmd_use_toml(name: String) -> AppResult<()> {
    let mut config = TomlConfig::load_or_default()?;
    config.set_default(&name)?;
    config.save()?;
    println!("✅ 已将 '{name}' 设为默认配置");
    Ok(())
}

/// 删除 TOML 配置
pub fn cmd_remove_toml(name: String) -> AppResult<()> {
    let mut config = TomlConfig::load_or_default()?;
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

    if let Some(d) = &config.default {
        println!("🎯 当前默认配置: {}", d);
    } else {
        println!("📋 暂无默认配置，可通过 'ccode use <name>' 设置");
    }
    Ok(())
}

/// 列出CCR配置（Router Profile）
#[cfg(any())]
pub fn cmd_list_ccr() -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // 列出前配置同步 - 读取CCR配置文件，更新provider信息
    manager.sync_config_from_ccr()?;

    println!("📋 CCR配置 (Router Profile) 列表：");
    println!();

    // 使用智能获取方法
    let profiles = manager.get_router_profiles()?;

    if profiles.is_empty() {
        // 检查具体原因并给出相应提示
        match manager.ensure_router_profile_exists()? {
            crate::ccr_config::RouterProfileStatus::NeedCreateProvider => {
                println!("❌ 暂无CCR配置");
                println!();
                println!("💡 要开始使用CCR，请按以下步骤操作:");
                println!("   1. ccode provider add <name>     # 添加Provider");
                println!("   2. ccode add-ccr <name>          # 添加CCR配置");
                return Ok(());
            }
            _ => {
                println!("❌ 暂无CCR配置");
                println!("💡 使用 'ccode add-ccr <name>' 添加CCR配置");
                return Ok(());
            }
        }
    }

    // 显示Router Profile列表
    for (name, profile, is_default) in profiles {
        let default_marker = if is_default { " (默认)" } else { "" };
        println!("🎯 {name}{default_marker}");
        println!("   🚀 默认路由: {}", profile.router.default);

        if let Some(background) = &profile.router.background {
            println!("   🔄 后台路由: {background}");
        }
        if let Some(think) = &profile.router.think {
            println!("   💭 思考路由: {think}");
        }
        if let Some(long_context) = &profile.router.long_context {
            println!("   📜 长上下文路由: {long_context}");
        }
        if let Some(web_search) = &profile.router.web_search {
            println!("   🔍 网络搜索路由: {web_search}");
        }

        if let Some(desc) = &profile.description {
            println!("   📝 描述: {desc}");
        }

        if let Some(created) = &profile.created_at {
            println!("   📅 创建: {created}");
        }

        println!();
    }

    // 显示当前应用的路由配置
    if manager.config_exists() {
        println!("📊 当前应用的路由配置：");
        let current_router = manager.get_current_router()?;
        println!("🎯 默认: {}", current_router.default);
        if let Some(background) = &current_router.background {
            println!("🔄 后台: {background}");
        }
        if let Some(think) = &current_router.think {
            println!("💭 思考: {think}");
        }
        if let Some(long_context) = &current_router.long_context {
            println!("📜 长上下文: {long_context}");
        }
        if let Some(web_search) = &current_router.web_search {
            println!("🔍 网络搜索: {web_search}");
        }

        // 显示Provider统计
        if let Ok(providers) = manager.list_providers() {
            println!();
            println!("🔗 可用 Provider: {}", providers.len());
        }
    } else {
        println!("⚠️  claude-code-router 配置文件不存在");
        println!("💡 请先使用 'ccode provider add <name>' 添加 Provider");
    }

    Ok(())
}

/// 添加CCR配置（Router Profile）
#[cfg(any())]
pub fn cmd_add_ccr(name: String) -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // 添加前配置同步 - 读取CCR配置文件，同步providers信息
    manager.sync_config_from_ccr()?;

    // 检查是否已存在同名Router Profile
    let config = Config::load().unwrap_or_default();
    if config.groups.router.contains_key(&name) {
        return Err(AppError::Config(format!("Router Profile '{name}' 已存在")));
    }

    // 检查是否有可用的 Providers
    if !manager.config_exists() {
        return Err(AppError::Config(
            "未找到 claude-code-router 配置文件，请先使用 'ccode provider add <name>' 添加 Provider".to_string()
        ));
    }

    let providers = manager.list_providers()?;
    if providers.is_empty() {
        return Err(AppError::Config(
            "暂无可用的 Provider，请先使用 'ccode provider add <name>' 添加 Provider".to_string(),
        ));
    }

    println!("🎯 添加新的CCR配置 (Router Profile): {name}");
    println!();

    // 显示可用的 Providers
    println!("📋 可用的 Providers:");
    for (index, provider) in providers.iter().enumerate() {
        println!(
            "  {}. {} [{}]",
            index + 1,
            provider.name,
            provider
                .provider_type
                .as_ref()
                .map(|t| t.display_name())
                .unwrap_or("未知类型")
        );
        println!("     📍 API URL: {}", provider.api_base_url);
        println!("     🤖 模型列表 ({} 个):", provider.models.len());

        // 显示所有模型，如果模型过多则分组显示
        if provider.models.len() <= 8 {
            for (model_idx, model) in provider.models.iter().enumerate() {
                println!("        {}. {}", model_idx + 1, model);
            }
        } else {
            // 显示前6个模型和最后2个模型
            for (model_idx, model) in provider.models.iter().take(6).enumerate() {
                println!("        {}. {}", model_idx + 1, model);
            }
            println!("        ... ({} 个模型)", provider.models.len() - 8);
            for (model_idx, model) in provider
                .models
                .iter()
                .skip(provider.models.len() - 2)
                .enumerate()
            {
                println!(
                    "        {}. {}",
                    provider.models.len() - 2 + model_idx + 1,
                    model
                );
            }
        }

        // 显示provider类型的特色功能提示
        if let Some(provider_type) = &provider.provider_type {
            let hints = provider_type.get_configuration_hints();
            if !hints.is_empty() {
                println!("     💡 特色功能:");
                for hint in hints.iter().take(2) {
                    // 只显示前2个提示避免过长
                    println!("        {hint}");
                }
            }
        }
        println!();
    }

    // 配置默认路由
    println!("🎯 配置默认路由 (格式: provider,model):");

    // 提供智能推荐
    if !providers.is_empty() {
        println!("💡 智能推荐路由:");
        let mut recommendations = Vec::new();

        for provider in &providers {
            if let Some(first_model) = provider.models.first() {
                let route = format!("{},{}", provider.name, first_model);
                let reason = if let Some(provider_type) = &provider.provider_type {
                    match provider_type {
                        crate::config::ProviderType::OpenAI => "🔑 最稳定兼容",
                        crate::config::ProviderType::OpenRouter => "🌐 多模型支持",
                        crate::config::ProviderType::DeepSeek => "🧠 强大的推理能力",
                        crate::config::ProviderType::Gemini => "🚀 Google最新技术",
                        crate::config::ProviderType::Qwen => "🎨 中文优化",
                        crate::config::ProviderType::Custom => "⚙️ 自定义配置",
                    }
                } else {
                    "💻 通用类型"
                };
                recommendations.push((route, reason));
            }
        }

        for (index, (route, reason)) in recommendations.iter().enumerate() {
            println!("  {}. {} - {}", index + 1, route, reason);
        }
        println!();
    }

    print!("默认路由: ");
    io::stdout().flush().unwrap();
    let mut default_route = String::new();
    io::stdin().read_line(&mut default_route)?;
    let default_route = default_route.trim().to_string();

    if default_route.is_empty() || !default_route.contains(',') {
        return Err(AppError::InvalidConfig(
            "默认路由格式无效，应为'provider,model'格式".to_string(),
        ));
    }

    // 验证路由配置是否有效
    let route_parts: Vec<&str> = default_route.split(',').collect();
    if route_parts.len() != 2 {
        return Err(AppError::InvalidConfig(
            "路由格式错误，应为'provider,model'格式".to_string(),
        ));
    }

    let (provider_name, model_name) = (route_parts[0].trim(), route_parts[1].trim());

    // 验证provider和model是否存在
    let provider_exists = providers.iter().any(|p| p.name == provider_name);
    if !provider_exists {
        return Err(AppError::InvalidConfig(format!(
            "提供商 '{provider_name}' 不存在"
        )));
    }

    let model_exists = providers
        .iter()
        .find(|p| p.name == provider_name)
        .map(|p| p.models.contains(&model_name.to_string()))
        .unwrap_or(false);

    if !model_exists {
        println!(
            "⚠️  警告: 模型 '{model_name}' 在提供商 '{provider_name}' 中不存在，请确认模型名称是否正确"
        );
    }

    // 创建基础 Router 配置
    let mut router = CcrRouter::new(default_route);

    // 可选路由配置
    let optional_routes = [
        ("background", "🔄 后台任务路由"),
        ("think", "💭 思考任务路由"),
        ("longContext", "📜 长上下文路由"),
        ("webSearch", "🔍 网络搜索路由"),
    ];

    for (route_key, route_desc) in optional_routes.iter() {
        println!();
        println!("{route_desc}:");

        // 为不同路由类型提供智能推荐
        let route_recommendations = get_route_recommendations(route_key, &providers);
        if !route_recommendations.is_empty() {
            println!("💡 推荐选项:");
            for (index, (route, reason)) in route_recommendations.iter().enumerate() {
                println!("  {}. {} - {}", index + 1, route, reason);
            }
        }

        print!("配置 {route_desc} (直接回车跳过): ");
        io::stdout().flush().unwrap();
        let mut route_input = String::new();
        io::stdin().read_line(&mut route_input)?;
        let route_input = route_input.trim();

        if !route_input.is_empty() {
            if !route_input.contains(',') {
                println!("⚠️  路由格式应为'provider,model'，跳过此设置");
                continue;
            }

            // 验证路由配置
            let parts: Vec<&str> = route_input.split(',').collect();
            if parts.len() == 2 {
                let (p_name, m_name) = (parts[0].trim(), parts[1].trim());
                if !providers.iter().any(|p| p.name == p_name) {
                    println!("⚠️  警告: 提供商 '{p_name}' 不存在");
                } else if !providers
                    .iter()
                    .any(|p| p.name == p_name && p.models.contains(&m_name.to_string()))
                {
                    println!("⚠️  警告: 模型 '{m_name}' 在提供商 '{p_name}' 中不存在");
                }
            }

            match *route_key {
                "background" => router.background = Some(route_input.to_string()),
                "think" => router.think = Some(route_input.to_string()),
                "longContext" => router.long_context = Some(route_input.to_string()),
                "webSearch" => router.web_search = Some(route_input.to_string()),
                _ => {}
            }
        }
    }

    // 配置长上下文阈值
    print!("⚖️  长上下文阈值 (默认: 60000): ");
    io::stdout().flush().unwrap();
    let mut threshold_input = String::new();
    io::stdin().read_line(&mut threshold_input)?;
    let threshold_input = threshold_input.trim();

    if !threshold_input.is_empty() {
        match threshold_input.parse::<u32>() {
            Ok(threshold) => {
                router.long_context_threshold = Some(threshold);
            }
            Err(_) => {
                println!("⚠️  无效的阈值格式，使用默认值 60000");
            }
        }
    }

    // 获取描述
    print!("📝 描述 (可选): ");
    io::stdout().flush().unwrap();
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim();
    let description = if description.is_empty() {
        None
    } else {
        Some(description.to_string())
    };

    // 创建 Router Profile
    let mut router_profile = RouterProfile::new(name.clone(), router, description)?;
    router_profile.created_at = Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());

    // 添加到本地配置
    manager.add_router_profile(name.clone(), router_profile)?;

    println!("✅ CCR配置 (Router Profile) '{name}' 添加成功！");

    // 检查是否是第一个Router Profile
    let updated_config = Config::load()?;
    if updated_config.groups.router.len() == 1 {
        println!("🎯 已自动设为默认CCR配置");
    }

    Ok(())
}

/// 使用CCR配置（激活Router Profile）
#[cfg(any())]
pub fn cmd_use_ccr(name: String) -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // 激活前配置同步 - 读取CCR配置文件，更新provider信息
    manager.sync_config_from_ccr()?;

    println!("🎯 激活CCR配置: {name}");
    println!();

    // 尝试获取指定的Router Profile（支持智能生成）
    let router_profile = manager.get_router_profile(&name)?;

    // 显示要激活的配置信息
    println!("📋 配置信息:");
    println!("   🚀 默认路由: {}", router_profile.router.default);
    if let Some(background) = &router_profile.router.background {
        println!("   🔄 后台路由: {background}");
    }
    if let Some(think) = &router_profile.router.think {
        println!("   💭 思考路由: {think}");
    }
    if let Some(long_context) = &router_profile.router.long_context {
        println!("   📜 长上下文路由: {long_context}");
    }
    if let Some(web_search) = &router_profile.router.web_search {
        println!("   🔍 网络搜索路由: {web_search}");
    }
    println!();

    // 验证Router配置中的Provider引用
    if manager.config_exists() {
        let validation_errors = manager.validate_router_references()?;
        if !validation_errors.is_empty() {
            println!("⚠️  发现配置问题:");
            for error in &validation_errors {
                println!("   • {error}");
            }
            print!("是否仍要继续激活此配置？某些路由可能无法工作 (y/N): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("❌ 已取消激活");
                println!("💡 请使用 'ccode provider add <name>' 添加缺失的 Provider");
                return Ok(());
            }
        }
    }

    // 使用CcrConfigManager的集成方法进行激活和同步
    manager.use_router_profile(&name)?;

    println!("✅ 已激活CCR配置 '{name}' 并同步到 claude-code-router");
    println!("🎯 默认路由: {}", router_profile.router.default);

    Ok(())
}

/// 运行CCR配置（使用原生ccr命令）
#[cfg(any())]
pub fn cmd_run_ccr(name: Option<String>) -> AppResult<()> {
    let ccr_manager = CcrConfigManager::new()?;

    // 启动时配置同步 - 读取CCR配置文件，更新provider信息
    ccr_manager.sync_config_from_ccr()?;

    println!("🚀 启动CCR配置...");
    println!("💡 使用ccr原生命令管理");
    println!();

    let config = Config::load().unwrap_or_default();

    // 检查是否有 Router Profile 配置
    if config.groups.router.is_empty() {
        println!("❌ 暂无 Router Profile 配置");
        if !ccr_manager.config_exists() {
            println!("💡 请先使用以下步骤配置:");
            println!("   1. ccode provider add <name>  # 添加 Provider");
            println!("   2. ccode add-ccr <name>       # 添加 Router Profile");
        } else {
            println!("💡 请使用 'ccode add-ccr <name>' 添加 Router Profile");
        }
        return Ok(());
    }

    // 获取要使用的 Router Profile
    let (profile_name, router_profile) = match name {
        Some(name) => {
            let profile = config.get_router_profile(&name)?;
            (name, profile)
        }
        None => match config.get_default_router_profile() {
            Ok((default_name, profile)) => (default_name.clone(), profile),
            Err(_) => {
                println!("❌ 未设置默认 Router Profile");
                let profiles = config.list_router_profiles();
                if !profiles.is_empty() {
                    println!("💡 可用的 Router Profile:");
                    for (name, _, _) in profiles {
                        println!("   • {name}");
                    }
                    println!("使用方法: ccode run-ccr <profile-name>");
                    println!("或者设置默认: ccode use-ccr <profile-name>");
                }
                return Ok(());
            }
        },
    };

    println!("🎯 使用 Router Profile '{profile_name}'");
    println!("🚀 默认路由: {}", router_profile.router.default);

    // 显示路由配置信息
    if let Some(background) = &router_profile.router.background {
        println!("🔄 后台路由: {background}");
    }
    if let Some(think) = &router_profile.router.think {
        println!("💭 思考路由: {think}");
    }
    if let Some(long_context) = &router_profile.router.long_context {
        println!("📜 长上下文路由: {long_context}");
    }
    if let Some(web_search) = &router_profile.router.web_search {
        println!("🔍 网络搜索路由: {web_search}");
    }
    println!();

    // 检查CCR配置文件是否存在
    if !ccr_manager.config_exists() {
        println!("❌ 未找到 claude-code-router 配置文件");
        println!("💡 请先使用 'ccode provider add <name>' 添加 Provider");
        return Ok(());
    }

    // 应用 Router Profile 到 claude-code-router 配置文件
    println!("📄 应用 Router Profile 到配置文件...");
    ccr_manager.apply_router_profile(router_profile)?;

    // 直接调用 ccr code 命令
    println!("🎯 启动 ccr code...");
    let mut cmd = Command::new("ccr");
    cmd.arg("code");

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                println!("✅ ccr code 程序正常退出");
            } else {
                println!("⚠️  ccr code 程序异常退出，退出码: {:?}", status.code());
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(AppError::CommandExecution(
                    "找不到 'ccr' 程序，请确保 claude-code-router 已安装并在 PATH 中".to_string(),
                ));
            } else {
                return Err(AppError::CommandExecution(format!(
                    "执行 ccr code 失败: {e}"
                )));
            }
        }
    }

    Ok(())
}

/// 删除CCR配置（Router Profile）
#[cfg(any())]
pub fn cmd_remove_ccr(name: String) -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // 检查Router Profile是否存在
    let config = Config::load().unwrap_or_default();
    if !config.groups.router.contains_key(&name) {
        return Err(AppError::ProfileNotFound(name));
    }

    println!("🗑️  删除CCR配置: {name}");
    println!();

    // 显示要删除的配置信息
    if let Ok(router_profile) = config.get_router_profile(&name) {
        println!("📋 将要删除的配置:");
        println!("   🚀 默认路由: {}", router_profile.router.default);
        if let Some(background) = &router_profile.router.background {
            println!("   🔄 后台路由: {background}");
        }
        if let Some(think) = &router_profile.router.think {
            println!("   💭 思考路由: {think}");
        }
        if let Some(long_context) = &router_profile.router.long_context {
            println!("   📜 长上下文路由: {long_context}");
        }
        if let Some(web_search) = &router_profile.router.web_search {
            println!("   🔍 网络搜索路由: {web_search}");
        }
        println!();
    }

    // 如果是默认配置，警告用户
    if let Some(default_profile) = &config.default_profile
        && default_profile.router.as_ref() == Some(&name)
    {
        println!("⚠️  '{name}' 是当前的默认CCR配置");
        println!("删除后需要重新设置默认配置");
        println!();
    }

    // 确认删除
    print!("确定要删除CCR配置 '{name}' 吗？(y/N): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("❌ 取消删除");
        return Ok(());
    }

    // 删除Router Profile
    manager.remove_router_profile(&name)?;

    println!("✅ CCR配置 '{name}' 已删除");

    // 显示当前默认配置状态
    let updated_config = Config::load().unwrap_or_default();
    if !updated_config.groups.router.is_empty() {
        if let Some(default_profile) = &updated_config.default_profile {
            if let Some(router) = &default_profile.router {
                println!("🎯 当前默认CCR配置: {router}");
            } else {
                println!("⚠️  无默认CCR配置，请使用 'ccode use-ccr <name>' 设置");
            }
        }
    } else {
        println!("📋 暂无CCR配置，请使用 'ccode add-ccr <name>' 添加配置");
    }

    Ok(())
}

/// 列出所有 Providers
#[cfg(any())]
pub fn cmd_provider_list() -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // Provider命令启动时同步配置
    manager.sync_config_from_ccr()?;

    if !manager.config_exists() {
        println!("📋 暂无 claude-code-router 配置文件");
        println!("💡 使用 'ccode provider add <name>' 添加第一个 Provider");
        return Ok(());
    }

    let providers = manager.list_providers()?;

    if providers.is_empty() {
        println!("📋 暂无 Provider 配置");
        println!("💡 使用 'ccode provider add <name>' 添加 Provider");
        return Ok(());
    }

    println!("📋 Provider 列表：");
    println!();

    for provider in providers {
        println!("🔗 {}", provider.name);
        println!("   📍 URL: {}", provider.api_base_url);
        println!(
            "   🔑 API Key: {}...",
            &provider.api_key[..7.min(provider.api_key.len())]
        );
        println!("   📊 模型数量: {}", provider.models.len());

        if let Some(provider_type) = &provider.provider_type {
            println!("   🏷️  类型: {}", provider_type.display_name());
        }

        if provider.models.len() <= 5 {
            println!("   🤖 模型: {}", provider.models.join(", "));
        } else {
            println!(
                "   🤖 模型: {} 等 {} 个",
                provider.models[..3].join(", "),
                provider.models.len()
            );
        }

        println!();
    }

    // 显示配置统计
    let stats = manager.get_config_stats()?;
    println!("📊 配置统计：");
    print!("{}", stats.format_display());

    Ok(())
}

/// 添加 Provider
#[cfg(any())]
pub fn cmd_provider_add(name: String) -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // Provider命令启动时同步配置
    manager.sync_config_from_ccr()?;

    // 检查 Provider 是否已存在
    if manager.provider_exists(&name)? {
        return Err(AppError::Config(format!("Provider '{name}' 已存在")));
    }

    println!("🔗 添加新 Provider: {name}");
    println!();

    // 选择 Provider 类型
    println!("📋 选择 Provider 类型:");
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

    // 获取 API 密钥
    print!("🔑 请输入 API Key: ");
    io::stdout().flush().unwrap();
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    // 获取 API URL（可选）
    println!("📍 API URL 配置:");
    println!("  默认: {}", provider_type.url_format_hint());
    print!("  自定义URL (直接回车使用默认): ");
    io::stdout().flush().unwrap();
    let mut api_url = String::new();
    io::stdin().read_line(&mut api_url)?;
    let api_url = api_url.trim();
    let api_base_url = if api_url.is_empty() {
        provider_type.url_format_hint().to_string()
    } else {
        api_url.to_string()
    };

    // 获取模型列表
    println!("🤖 模型配置:");
    println!(
        "  默认模型: {}",
        provider_type.get_default_models().join(", ")
    );
    print!("  自定义模型列表 (用逗号分隔，直接回车使用默认): ");
    io::stdout().flush().unwrap();
    let mut models_input = String::new();
    io::stdin().read_line(&mut models_input)?;
    let models_input = models_input.trim();
    let models = if models_input.is_empty() {
        provider_type.get_default_models()
    } else {
        models_input
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    };

    println!();
    println!("🔧 正在创建 Provider...");

    // 创建 Provider
    let provider = CcrProvider::new(
        name.clone(),
        api_base_url,
        api_key,
        models,
        provider_type.clone(),
    );

    // 添加 Provider
    manager.add_provider(provider)?;

    println!("✅ Provider '{name}' 添加成功！");
    println!("🔗 类型: {}", provider_type.display_name());

    Ok(())
}

/// 删除 Provider
#[cfg(any())]
pub fn cmd_provider_remove(name: String) -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // Provider命令启动时同步配置
    manager.sync_config_from_ccr()?;

    // 检查 Provider 是否存在
    if !manager.provider_exists(&name)? {
        return Err(AppError::Config(format!("Provider '{name}' 不存在")));
    }

    // 检查是否被 Router 引用
    let validation_errors = manager.validate_router_references()?;
    let is_referenced = validation_errors.iter().any(|error| error.contains(&name));

    if is_referenced {
        println!("⚠️  警告: Provider '{name}' 正被 Router 配置引用");
        println!("删除后相关路由将失效，请确认是否继续");
    }

    // 确认删除
    print!("⚠️  确定要删除 Provider '{name}' 吗？(y/N): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("❌ 取消删除");
        return Ok(());
    }

    manager.remove_provider(&name)?;
    println!("✅ Provider '{name}' 已删除");

    if is_referenced {
        println!("💡 建议使用 'ccode router list' 检查相关路由配置");
    }

    Ok(())
}

/// 显示 Provider 详情
#[cfg(any())]
pub fn cmd_provider_show(name: String) -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // Provider命令启动时同步配置
    manager.sync_config_from_ccr()?;

    let provider = manager.get_provider(&name)?;

    println!("🔗 Provider: {}", provider.name);
    println!();
    println!("📍 API URL: {}", provider.api_base_url);
    println!(
        "🔑 API Key: {}...",
        &provider.api_key[..7.min(provider.api_key.len())]
    );

    if let Some(provider_type) = &provider.provider_type {
        println!("🏷️  类型: {}", provider_type.display_name());
    }

    println!("📊 模型数量: {}", provider.models.len());
    println!("🤖 模型列表:");
    for (index, model) in provider.models.iter().enumerate() {
        println!("  {}. {}", index + 1, model);
    }

    if let Some(transformer) = &provider.transformer {
        println!("🔄 Transformer 配置:");
        println!("{}", serde_json::to_string_pretty(transformer)?);
    }

    Ok(())
}

/// 编辑 Provider
#[cfg(any())]
pub fn cmd_provider_edit(name: String) -> AppResult<()> {
    let manager = CcrConfigManager::new()?;

    // Provider命令启动时同步配置
    manager.sync_config_from_ccr()?;

    let mut provider = manager.get_provider(&name)?;

    println!("✏️  编辑 Provider: {}", provider.name);
    println!();

    // 编辑 API Key
    println!(
        "🔑 当前 API Key: {}...",
        &provider.api_key[..7.min(provider.api_key.len())]
    );
    print!("新 API Key (直接回车保持不变): ");
    io::stdout().flush().unwrap();
    let mut new_api_key = String::new();
    io::stdin().read_line(&mut new_api_key)?;
    let new_api_key = new_api_key.trim();
    if !new_api_key.is_empty() {
        provider.api_key = new_api_key.to_string();
    }

    // 编辑 API URL
    println!("📍 当前 API URL: {}", provider.api_base_url);
    print!("新 API URL (直接回车保持不变): ");
    io::stdout().flush().unwrap();
    let mut new_url = String::new();
    io::stdin().read_line(&mut new_url)?;
    let new_url = new_url.trim();
    if !new_url.is_empty() {
        provider.api_base_url = new_url.to_string();
    }

    // 编辑模型列表
    println!("🤖 当前模型: {}", provider.models.join(", "));
    print!("新模型列表 (用逗号分隔，直接回车保持不变): ");
    io::stdout().flush().unwrap();
    let mut new_models = String::new();
    io::stdin().read_line(&mut new_models)?;
    let new_models = new_models.trim();
    if !new_models.is_empty() {
        provider.models = new_models
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // 重新生成 transformer
        if let Some(provider_type) = &provider.provider_type {
            provider.transformer = provider_type.generate_transformer(&provider.models);
        }
    }

    // 保存更新
    manager.update_provider(provider)?;
    println!("✅ Provider '{name}' 更新成功！");

    Ok(())
}
