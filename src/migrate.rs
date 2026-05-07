use crate::config as json_cfg;
use crate::error::{AppError, AppResult};
use crate::toml_config::{self as toml_cfg, TomlConfig, TomlProfile};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub backup_path: PathBuf,
    pub profiles_total: usize,
    pub profiles_migrated: usize,
    pub profiles_skipped: usize,
    pub created_toml: bool,
    pub merged_into_existing: bool,
    pub default_set: Option<String>,
}

fn backup_json(json_path: &Path) -> AppResult<PathBuf> {
    let ts = Local::now().format("%Y%m%d-%H%M%S");
    let backup = json_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "{}.bak-{}",
            json_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("config.json"),
            ts
        ));
    fs::copy(json_path, &backup)?;
    Ok(backup)
}

fn remove_json(json_path: &Path) -> AppResult<()> {
    if json_path.exists() {
        fs::remove_file(json_path)?;
    }
    Ok(())
}

fn map_json_to_toml(
    json: &json_cfg::Config,
) -> (TomlConfig, Vec<(String, TomlProfile, Option<String>)>) {
    // Return the target TomlConfig skeleton and list of (name, profile, token_opt)
    let mut pairs = Vec::new();
    let mut default = None;
    if let Some(dp) = &json.default_profile {
        default = dp.direct.clone();
    }

    if let Some(groups) = Some(&json.groups) {
        for (name, prof) in &groups.direct {
            let env_key = toml_cfg::derive_env_key_from_profile(name);
            let comment = prof.description.as_ref().map(|s| s.to_string());
            let mut tp = TomlProfile {
                name: Some(name.clone()),
                base_url: prof.anthropic_base_url.clone(),
                env_key: env_key.clone(),
                model: prof.anthropic_model.clone(),
                model_haiku: None,
                model_sonnet: None,
                model_opus: None,
                subagent_model: None,
                max_tokens: None,
                comment,
            };
            if let Some(fast) = &prof.anthropic_small_fast_model {
                // 映射弃用字段到 haiku
                if tp.model_haiku.is_none() {
                    tp.model_haiku = Some(fast.clone());
                }
            }
            let token_opt = Some(prof.anthropic_auth_token.clone());
            pairs.push((name.clone(), tp, token_opt));
        }
    }

    let tgt = TomlConfig {
        default,
        ..Default::default()
    };
    (tgt, pairs)
}

#[allow(dead_code)]
fn merge_into_existing(
    mut existing: TomlConfig,
    pairs: &[(String, TomlProfile, Option<String>)],
) -> (TomlConfig, usize, usize) {
    let mut migrated = 0usize;
    let mut skipped = 0usize;
    for (name, profile, _token) in pairs {
        if existing.profiles.contains_key(name) {
            skipped += 1;
            continue;
        }
        existing.profiles.insert(name.clone(), profile.clone());
        migrated += 1;
    }
    (existing, migrated, skipped)
}

fn persist_tokens_for_pairs(
    toml_path: &Path,
    pairs: &[(String, TomlProfile, Option<String>)],
) -> AppResult<()> {
    for (_, profile, token_opt) in pairs {
        if let Some(token) = token_opt {
            toml_cfg::persist_token_to_env(toml_path, &profile.env_key, token)?;
        }
    }
    Ok(())
}

/// 开机自动迁移：当存在 config.json 且缺少 config.toml 时执行
pub fn auto_migrate_if_needed() -> AppResult<()> {
    let json_path = json_cfg::Config::get_config_path()?;
    let toml_path = TomlConfig::get_config_path()?;

    let json_exists = json_path.exists();
    let toml_exists = toml_path.exists();

    if !json_exists {
        return Ok(());
    }

    if toml_exists {
        println!(
            "⚠️ 检测到同时存在 config.toml 与 config.json。为避免覆盖，请删除旧的 config.json 或手动合并。当前将继续使用 TOML。"
        );
        return Ok(());
    }

    // 自动迁移流程
    let backup = backup_json(&json_path)?;
    let json = json_cfg::Config::load()?; // 会迁移旧字段到 groups.direct
    let (toml_cfg_out, pairs) = map_json_to_toml(&json);

    // 写 TOML
    toml_cfg_out.save()?;
    // 写 .env 中的 token
    persist_tokens_for_pairs(&toml_path, &pairs)?;
    // 将 profiles 放入刚保存的 TOML（必须重新加载以防并发覆盖）
    let mut final_toml = TomlConfig::load_or_default()?;
    for (name, profile, _) in &pairs {
        final_toml.profiles.insert(name.clone(), profile.clone());
    }
    if final_toml.default.is_none() {
        final_toml.default = toml_cfg_out.default.clone();
    }
    final_toml.save()?;

    // 移除 JSON
    remove_json(&json_path)?;

    println!(
        "✅ 已自动从 JSON 迁移到 TOML（{} 个配置），备份: {}",
        pairs.len(),
        backup.display()
    );
    Ok(())
}

/// 手动迁移/合并：`ccode config merge`
#[allow(dead_code)]
pub fn manual_merge() -> AppResult<MigrationReport> {
    let json_path = json_cfg::Config::get_config_path()?;
    let toml_path = TomlConfig::get_config_path()?;

    if !json_path.exists() {
        return Err(AppError::Config("未找到 config.json，无需迁移".into()));
    }

    let backup = backup_json(&json_path)?;
    let json = json_cfg::Config::load()?;
    let (mut target_toml, pairs) = map_json_to_toml(&json);
    let total = pairs.len();

    let mut created_toml = false;
    let mut merged_into_existing = false;
    let (migrated, skipped);

    if toml_path.exists() {
        // 合并到现有 TOML（遇到同名跳过）
        let existing = TomlConfig::load_or_default()?;
        let (mut merged, m_count, s_count) = merge_into_existing(existing, &pairs);
        migrated = m_count;
        skipped = s_count;
        // 默认值：若现有无默认且 JSON 有默认且该 profile 存在于合并结果，则设置
        if merged.default.is_none() {
            merged.default = target_toml.default.take();
        }
        merged.save()?;
        merged_into_existing = true;
    } else {
        // 直接生成新 TOML
        for (name, p, _) in &pairs {
            target_toml.profiles.insert(name.clone(), p.clone());
        }
        target_toml.save()?;
        created_toml = true;
        migrated = total;
        skipped = 0;
    }

    // 写入 .env token
    persist_tokens_for_pairs(&toml_path, &pairs)?;

    // 移除 JSON
    remove_json(&json_path)?;

    let report = MigrationReport {
        backup_path: backup,
        profiles_total: total,
        profiles_migrated: migrated,
        profiles_skipped: skipped,
        created_toml,
        merged_into_existing,
        default_set: target_toml.default,
    };

    Ok(report)
}
