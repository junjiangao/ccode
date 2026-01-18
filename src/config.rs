//! 旧版 JSON 配置格式（仅用于迁移到 TOML）
//!
//! 此模块仅在 `migrate.rs` 中使用，用于读取旧版 `config.json` 并迁移到新的 `config.toml` 格式。
//! 新代码应使用 `toml_config.rs` 中的配置结构。

use crate::error::{AppError, AppResult};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Direct 模式配置项（旧版 JSON 格式）
#[derive(Debug, Clone, Deserialize)]
pub struct DirectProfile {
    #[serde(rename = "ANTHROPIC_AUTH_TOKEN")]
    pub anthropic_auth_token: String,
    #[serde(rename = "ANTHROPIC_BASE_URL")]
    pub anthropic_base_url: String,
    #[serde(rename = "ANTHROPIC_MODEL")]
    pub anthropic_model: Option<String>,
    #[serde(rename = "ANTHROPIC_SMALL_FAST_MODEL")]
    pub anthropic_small_fast_model: Option<String>,
    pub description: Option<String>,
}

/// 默认配置指针
#[derive(Debug, Clone, Deserialize)]
pub struct DefaultProfile {
    pub direct: Option<String>,
}

/// 配置分组
#[derive(Debug, Clone, Deserialize)]
pub struct Groups {
    #[serde(default)]
    pub direct: HashMap<String, DirectProfile>,
}

/// 旧版 ccode 配置文件结构（JSON 格式）
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub default_profile: Option<DefaultProfile>,
    pub groups: Groups,
    // 兼容更旧格式
    pub default: Option<String>,
    pub profiles: Option<HashMap<String, DirectProfile>>,
}

impl Config {
    /// 配置文件路径：~/.config/ccode/config.json
    pub fn get_config_path() -> AppResult<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| AppError::Config("无法获取配置目录".to_string()))?;
        Ok(config_dir.join("ccode").join("config.json"))
    }

    /// 从文件加载配置，包含旧格式迁移
    pub fn load() -> AppResult<Self> {
        let config_path = Self::get_config_path()?;
        if !config_path.exists() {
            return Err(AppError::ConfigNotFound);
        }
        let content = fs::read_to_string(config_path)?;
        let mut config: Config = serde_json::from_str(&content)?;
        config.migrate_legacy_format();
        Ok(config)
    }

    /// 迁移更旧格式（`profiles`/`default`）到分组结构
    fn migrate_legacy_format(&mut self) {
        if let Some(profiles) = self.profiles.take() {
            for (name, profile) in profiles {
                self.groups.direct.insert(name, profile);
            }
        }
        if let Some(old_default) = self.default.take() {
            if self.default_profile.is_none() {
                self.default_profile = Some(DefaultProfile {
                    direct: Some(old_default),
                });
            } else if let Some(ref mut dp) = self.default_profile
                && dp.direct.is_none()
            {
                dp.direct = Some(old_default);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_direct_profile() {
        let json = r#"{
            "ANTHROPIC_AUTH_TOKEN": "test-token",
            "ANTHROPIC_BASE_URL": "https://api.test.com",
            "ANTHROPIC_MODEL": "claude-3",
            "description": "Test"
        }"#;
        let profile: DirectProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.anthropic_auth_token, "test-token");
        assert_eq!(profile.anthropic_base_url, "https://api.test.com");
        assert_eq!(profile.anthropic_model, Some("claude-3".to_string()));
    }

    #[test]
    fn test_deserialize_minimal_profile() {
        let json = r#"{
            "ANTHROPIC_AUTH_TOKEN": "token",
            "ANTHROPIC_BASE_URL": "https://api.example.com"
        }"#;
        let profile: DirectProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.anthropic_auth_token, "token");
        assert!(profile.anthropic_model.is_none());
        assert!(profile.description.is_none());
    }
}
