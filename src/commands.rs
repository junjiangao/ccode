use crate::error::{AppError, AppResult};
use crate::tmux_env::{self, TmuxEnvMode};
use crate::toml_config::{TomlConfig, TomlProfile, load_token_from_env};
use std::io::{self, Write};
use std::process::Command;

/// 读取可选字符串输入的通用函数
fn read_optional_input(prompt: &str) -> AppResult<Option<String>> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    Ok(if input.is_empty() {
        None
    } else {
        Some(input.to_string())
    })
}

/// 列出配置
pub fn cmd_list() -> AppResult<()> {
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

/// 添加配置（交互式）
pub fn cmd_add(name: String) -> AppResult<()> {
    let mut config = TomlConfig::load_or_default()?;
    if config.profiles.contains_key(&name) {
        return Err(AppError::Config(format!("配置 '{name}' 已存在")));
    }

    println!("🔧 添加 TOML 配置: {name}");
    println!();

    // base_url（使用环境变量名提示）
    print!("📍 请输入 ANTHROPIC_BASE_URL (如: https://api.anthropic.com): ");
    io::stdout().flush()?;
    let mut base_url = String::new();
    io::stdin().read_line(&mut base_url)?;
    let base_url = base_url.trim().to_string();

    // 令牌输入：直接输入 Key，程序将写入 .env 中自动生成的 name_key 变量
    let env_key = crate::toml_config::derive_env_key_from_profile(&name);
    print!("🔑 请输入 ANTHROPIC_AUTH_TOKEN（将保存为 .env 的 {env_key}）: ");
    io::stdout().flush()?;
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

/// 设置默认配置
pub fn cmd_use(name: String) -> AppResult<()> {
    let mut config = TomlConfig::load_or_default()?;
    config.set_default(&name)?;
    config.save()?;
    println!("✅ 已将 '{name}' 设为默认配置");
    Ok(())
}

/// 删除配置
pub fn cmd_remove(name: String) -> AppResult<()> {
    let mut config = TomlConfig::load_or_default()?;
    print!("⚠️  确定要删除配置 '{name}' 吗？(y/N): ");
    io::stdout().flush()?;
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
        println!("🎯 当前默认配置: {d}");
    } else {
        println!("📋 暂无默认配置，可通过 'ccode profile use <name>' 设置");
    }
    Ok(())
}

/// 运行 claude（核心函数）
pub fn cmd_run(
    name: Option<String>,
    tmux_env: TmuxEnvMode,
    claude_args: Vec<String>,
    quiet: bool,
) -> AppResult<()> {
    let config = TomlConfig::load()?;
    let toml_path = TomlConfig::get_config_path()?;
    let (profile_name, profile) = config.get_profile(name.as_deref())?;

    // 读取 token：优先同目录 .env，其次系统环境
    let token = load_token_from_env(&toml_path, &profile.env_key)?;

    if !quiet {
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
    }

    let _tmux_update_guard =
        tmux_env::try_patch_tmux_update_environment(tmux_env, &claude_args, quiet);

    // 设置需要注入到 tmux 全局的环境变量
    let mut env_vars = vec![
        ("ANTHROPIC_AUTH_TOKEN".to_string(), token.clone()),
        ("ANTHROPIC_BASE_URL".to_string(), profile.base_url.clone()),
    ];
    if let Some(m) = &profile.model {
        env_vars.push(("ANTHROPIC_MODEL".to_string(), m.clone()));
    }
    if let Some(m) = &profile.model_haiku {
        env_vars.push(("ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(), m.clone()));
        env_vars.push(("ANTHROPIC_SMALL_FAST_MODEL".to_string(), m.clone()));
    }
    // opus/sonnet 回退逻辑：未指定则使用默认模型
    let opus_model = profile.model_opus.as_ref().or(profile.model.as_ref());
    if let Some(m) = opus_model {
        env_vars.push(("ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(), m.clone()));
    }
    let sonnet_model = profile.model_sonnet.as_ref().or(profile.model.as_ref());
    if let Some(m) = sonnet_model {
        env_vars.push(("ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(), m.clone()));
    }
    if let Some(max) = &profile.max_tokens {
        env_vars.push(("CLAUDE_CODE_MAX_OUTPUT_TOKENS".to_string(), max.clone()));
    }

    let _tmux_global_env_guard = tmux_env::try_set_tmux_global_env(&env_vars);

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
    // opus/sonnet 回退逻辑：未指定则使用默认模型
    let opus_model_fallback = profile.model_opus.as_ref().or(profile.model.as_ref());
    if let Some(m) = opus_model_fallback {
        cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", m);
    }
    let sonnet_model_fallback = profile.model_sonnet.as_ref().or(profile.model.as_ref());
    if let Some(m) = sonnet_model_fallback {
        cmd.env("ANTHROPIC_DEFAULT_SONNET_MODEL", m);
    }
    if let Some(max) = &profile.max_tokens {
        cmd.env("CLAUDE_CODE_MAX_OUTPUT_TOKENS", max);
    }

    if !quiet && !claude_args.is_empty() {
        cmd.args(&claude_args);
        println!("📄 透传参数: {}", claude_args.join(" "));
    } else if !claude_args.is_empty() {
        cmd.args(&claude_args);
    }

    match cmd.status() {
        Ok(status) => {
            if !quiet {
                if status.success() {
                    println!("✅ claude 程序正常退出");
                } else {
                    println!("⚠️  claude 程序异常退出，退出码: {:?}", status.code());
                }
            }
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
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

/// 清理 tmux 中 ccode 注入相关环境变量
pub fn cmd_tmux_clear_env() -> AppResult<()> {
    let report = tmux_env::clear_tmux_env_vars()?;
    if !report.had_server {
        println!("ℹ️ 未检测到 tmux server，无需清理");
        return Ok(());
    }

    println!(
        "🧹 已清理 tmux 环境变量（会话数: {}）",
        report.session_count
    );
    println!("ℹ️ 仅影响后续新建 pane/window，不影响已运行进程");
    Ok(())
}
