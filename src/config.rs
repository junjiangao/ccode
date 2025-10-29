#![allow(dead_code)]
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Direct 模式配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectProfile {
    #[serde(rename = "ANTHROPIC_AUTH_TOKEN")]
    pub anthropic_auth_token: String,
    #[serde(rename = "ANTHROPIC_BASE_URL")]
    pub anthropic_base_url: String,
    #[serde(rename = "ANTHROPIC_MODEL", skip_serializing_if = "Option::is_none")]
    pub anthropic_model: Option<String>,
    #[serde(
        rename = "ANTHROPIC_SMALL_FAST_MODEL",
        skip_serializing_if = "Option::is_none"
    )]
    pub anthropic_small_fast_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl DirectProfile {
    /// 显示可选字段信息（供命令行打印使用）
    pub fn display_optional_fields(&self, indent: &str) {
        if let Some(model) = &self.anthropic_model {
            println!("{indent}🤖 模型: {model}");
        }
        if let Some(fast_model) = &self.anthropic_small_fast_model {
            println!("{indent}⚡ 快速模型: {fast_model}");
        }
        if let Some(desc) = &self.description {
            println!("{indent}📝 描述: {desc}");
        }
        if let Some(created) = &self.created_at {
            println!("{indent}📅 创建: {created}");
        }
    }
}

/// 向后兼容的 Profile 类型别名（等同 DirectProfile）
pub type Profile = DirectProfile;

/// 默认配置指针
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultProfile {
    pub direct: Option<String>,
}

/// 配置分组（现仅 Direct）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groups {
    pub direct: HashMap<String, DirectProfile>,
}

/// ccode 配置文件结构（仅 Direct）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<DefaultProfile>,
    pub groups: Groups,

    // 兼容旧格式字段（profiles/default）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<HashMap<String, DirectProfile>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            default_group: Some("direct".to_string()),
            default_profile: Some(DefaultProfile { direct: None }),
            groups: Groups {
                direct: HashMap::new(),
            },
            default: None,
            profiles: None,
        }
    }
}

impl Config {
    /// 配置文件路径：~/.config/ccode/config.json
    pub fn get_config_path() -> AppResult<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| AppError::Config("无法获取配置目录".to_string()))?;
        let ccode_dir = config_dir.join("ccode");
        if !ccode_dir.exists() {
            fs::create_dir_all(&ccode_dir)?;
        }
        Ok(ccode_dir.join("config.json"))
    }

    /// 从文件加载配置，包含旧格式迁移
    pub fn load() -> AppResult<Self> {
        let config_path = Self::get_config_path()?;
        if !config_path.exists() {
            return Err(AppError::ConfigNotFound);
        }
        let content = fs::read_to_string(config_path)?;
        let mut config: Config = serde_json::from_str(&content)?;
        config.migrate_legacy_format()?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self) -> AppResult<()> {
        let config_path = Self::get_config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// 迁移旧格式（`profiles`/`default`）到新分组（direct）
    fn migrate_legacy_format(&mut self) -> AppResult<()> {
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
            } else if let Some(ref mut default_profile) = self.default_profile
                && default_profile.direct.is_none()
            {
                default_profile.direct = Some(old_default);
            }
        }
        if self.default_group.is_none() {
            self.default_group = Some("direct".to_string());
        }
        Ok(())
    }

    /// 添加 Direct 配置
    pub fn add_direct_profile(&mut self, name: String, profile: DirectProfile) -> AppResult<()> {
        if self.groups.direct.contains_key(&name) {
            return Err(AppError::Config(format!("配置 '{name}' 已存在")));
        }
        self.validate_direct_profile(&profile)?;
        self.groups.direct.insert(name.clone(), profile);
        if self.groups.direct.len() == 1 {
            if let Some(ref mut default_profile) = self.default_profile {
                default_profile.direct = Some(name);
            } else {
                self.default_profile = Some(DefaultProfile { direct: Some(name) });
            }
        }
        Ok(())
    }

    /// 删除配置（仅 direct）
    pub fn remove_profile(&mut self, name: &str) -> AppResult<()> {
        if self.groups.direct.contains_key(name) {
            return self.remove_direct_profile(name);
        }
        Err(AppError::ProfileNotFound(name.to_string()))
    }

    /// 删除 Direct 配置
    pub fn remove_direct_profile(&mut self, name: &str) -> AppResult<()> {
        if !self.groups.direct.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }
        self.groups.direct.remove(name);
        if let Some(ref mut default_profile) = self.default_profile
            && default_profile.direct.as_ref() == Some(&name.to_string())
        {
            default_profile.direct = self.groups.direct.keys().next().cloned();
        }
        Ok(())
    }

    /// 获取 Direct 配置
    pub fn get_direct_profile(&self, name: &str) -> AppResult<&DirectProfile> {
        self.groups
            .direct
            .get(name)
            .ok_or_else(|| AppError::ProfileNotFound(name.to_string()))
    }

    /// 获取默认 Direct 配置
    pub fn get_default_direct_profile(&self) -> AppResult<(&String, &DirectProfile)> {
        let default_name = self
            .default_profile
            .as_ref()
            .and_then(|dp| dp.direct.as_ref())
            .ok_or_else(|| AppError::Config("未设置默认Direct配置".to_string()))?;
        let profile = self.get_direct_profile(default_name)?;
        Ok((default_name, profile))
    }

    /// 设置默认（向后兼容）
    pub fn set_default(&mut self, name: &str) -> AppResult<()> {
        if self.groups.direct.contains_key(name) {
            return self.set_default_direct(name);
        }
        Err(AppError::ProfileNotFound(name.to_string()))
    }

    /// 设置默认 Direct 配置
    pub fn set_default_direct(&mut self, name: &str) -> AppResult<()> {
        if !self.groups.direct.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }
        if let Some(ref mut default_profile) = self.default_profile {
            default_profile.direct = Some(name.to_string());
        } else {
            self.default_profile = Some(DefaultProfile {
                direct: Some(name.to_string()),
            });
        }
        Ok(())
    }

    /// 列出 Direct 配置
    pub fn list_direct_profiles(&self) -> Vec<(String, &DirectProfile, bool)> {
        let default_name = self
            .default_profile
            .as_ref()
            .and_then(|dp| dp.direct.as_ref());

        self.groups
            .direct
            .iter()
            .map(|(name, profile)| {
                let is_default = default_name == Some(name);
                (name.clone(), profile, is_default)
            })
            .collect()
    }

    /// 验证 Direct 配置有效性
    fn validate_direct_profile(&self, profile: &DirectProfile) -> AppResult<()> {
        if profile.anthropic_auth_token.trim().is_empty() {
            return Err(AppError::InvalidConfig("认证令牌不能为空".to_string()));
        }
        if profile.anthropic_base_url.trim().is_empty() {
            return Err(AppError::InvalidConfig("基础URL不能为空".to_string()));
        }
        if !profile.anthropic_base_url.starts_with("http://")
            && !profile.anthropic_base_url.starts_with("https://")
        {
            return Err(AppError::InvalidConfig(
                "基础URL格式无效，应以 'http://' 或 'https://' 开头".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile() -> Profile {
        Profile {
            anthropic_auth_token: "test-token-123".to_string(),
            anthropic_base_url: "https://api.anthropic.com".to_string(),
            anthropic_model: None,
            anthropic_small_fast_model: None,
            description: Some("Test profile".to_string()),
            created_at: Some("2025-07-29T00:00:00Z".to_string()),
        }
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.default, None);
        assert!(config.groups.direct.is_empty());
    }

    #[test]
    fn test_profile_creation() {
        let profile = create_test_profile();
        assert_eq!(profile.anthropic_auth_token, "test-token-123");
        assert_eq!(profile.anthropic_base_url, "https://api.anthropic.com");
        assert_eq!(profile.anthropic_model, None);
        assert_eq!(profile.anthropic_small_fast_model, None);
        assert_eq!(profile.description, Some("Test profile".to_string()));
    }

    #[test]
    fn test_profile_with_optional_fields() {
        let profile = Profile {
            anthropic_auth_token: "test-token".to_string(),
            anthropic_base_url: "https://api.test.com".to_string(),
            anthropic_model: Some("claude-3-5-sonnet-20241022".to_string()),
            anthropic_small_fast_model: Some("claude-3-haiku-20240307".to_string()),
            description: Some("Test with models".to_string()),
            created_at: None,
        };
        assert_eq!(
            profile.anthropic_model,
            Some("claude-3-5-sonnet-20241022".to_string())
        );
        assert_eq!(
            profile.anthropic_small_fast_model,
            Some("claude-3-haiku-20240307".to_string())
        );
    }

    #[test]
    fn test_profile_serialization() {
        let profile = Profile {
            anthropic_auth_token: "test-token".to_string(),
            anthropic_base_url: "https://api.test.com".to_string(),
            anthropic_model: Some("test-model".to_string()),
            anthropic_small_fast_model: Some("test-fast-model".to_string()),
            description: Some("Test".to_string()),
            created_at: None,
        };
        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("ANTHROPIC_MODEL"));
        assert!(json.contains("ANTHROPIC_SMALL_FAST_MODEL"));
        assert!(json.contains("test-model"));
        assert!(json.contains("test-fast-model"));
        let deserialized: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.anthropic_model, Some("test-model".to_string()));
        assert_eq!(
            deserialized.anthropic_small_fast_model,
            Some("test-fast-model".to_string())
        );
    }

    #[test]
    fn test_profile_serialization_without_optional_fields() {
        let profile = Profile {
            anthropic_auth_token: "test-token".to_string(),
            anthropic_base_url: "https://api.test.com".to_string(),
            anthropic_model: None,
            anthropic_small_fast_model: None,
            description: None,
            created_at: None,
        };
        let json = serde_json::to_string(&profile).unwrap();
        assert!(!json.contains("ANTHROPIC_MODEL"));
        assert!(!json.contains("ANTHROPIC_SMALL_FAST_MODEL"));
        let deserialized: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.anthropic_model, None);
        assert_eq!(deserialized.anthropic_small_fast_model, None);
    }
}
