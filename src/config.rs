use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Direct模式配置项（原有的简单配置）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectProfile {
    #[serde(rename = "ANTHROPIC_AUTH_TOKEN")]
    pub anthropic_auth_token: String,
    #[serde(rename = "ANTHROPIC_BASE_URL")]
    pub anthropic_base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// 向后兼容的Profile类型别名
pub type Profile = DirectProfile;

/// CCR提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcrProvider {
    pub name: String,
    pub api_base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformer: Option<serde_json::Value>,
}

/// CCR路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcrRouter {
    pub default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<String>,
    #[serde(rename = "longContext", skip_serializing_if = "Option::is_none")]
    pub long_context: Option<String>,
    #[serde(
        rename = "longContextThreshold",
        skip_serializing_if = "Option::is_none"
    )]
    pub long_context_threshold: Option<u32>,
    #[serde(rename = "webSearch", skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,
}

/// CCR模式配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcrProfile {
    #[serde(rename = "Providers")]
    pub providers: Vec<CcrProvider>,
    #[serde(rename = "Router")]
    pub router: CcrRouter,
    #[serde(rename = "API_TIMEOUT_MS", skip_serializing_if = "Option::is_none")]
    pub api_timeout_ms: Option<u32>,
    #[serde(rename = "PROXY_URL", skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(rename = "LOG", skip_serializing_if = "Option::is_none")]
    pub log: Option<bool>,
    #[serde(rename = "APIKEY", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(rename = "HOST", skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// 默认配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultProfile {
    pub direct: Option<String>,
    pub ccr: Option<String>,
}

/// 配置组集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groups {
    pub direct: HashMap<String, DirectProfile>,
    pub ccr: HashMap<String, CcrProfile>,
}

/// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<DefaultProfile>,
    pub groups: Groups,

    // 兼容旧格式的字段
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
            default_profile: Some(DefaultProfile {
                direct: None,
                ccr: None,
            }),
            groups: Groups {
                direct: HashMap::new(),
                ccr: HashMap::new(),
            },
            // 兼容字段设为None
            default: None,
            profiles: None,
        }
    }
}

impl Config {
    /// 获取配置文件路径
    pub fn get_config_path() -> AppResult<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| AppError::Config("无法获取配置目录".to_string()))?;

        let ccode_dir = config_dir.join("ccode");

        // 确保配置目录存在
        if !ccode_dir.exists() {
            fs::create_dir_all(&ccode_dir)?;
        }

        Ok(ccode_dir.join("config.json"))
    }

    /// 从配置文件加载配置
    pub fn load() -> AppResult<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Err(AppError::ConfigNotFound);
        }

        let content = fs::read_to_string(config_path)?;
        let mut config: Config = serde_json::from_str(&content)?;

        // 迁移旧格式配置到新格式
        config.migrate_legacy_format()?;

        Ok(config)
    }

    /// 迁移旧格式配置到新的分组格式
    fn migrate_legacy_format(&mut self) -> AppResult<()> {
        // 如果存在旧格式的profiles字段，迁移它们到groups.direct
        if let Some(profiles) = self.profiles.take() {
            for (name, profile) in profiles {
                self.groups.direct.insert(name, profile);
            }
        }

        // 迁移旧的default字段到新的default_profile.direct
        if let Some(old_default) = self.default.take() {
            if self.default_profile.is_none() {
                self.default_profile = Some(DefaultProfile {
                    direct: Some(old_default),
                    ccr: None,
                });
            } else if let Some(ref mut default_profile) = self.default_profile {
                if default_profile.direct.is_none() {
                    default_profile.direct = Some(old_default);
                }
            }
        }

        // 确保default_group存在
        if self.default_group.is_none() {
            self.default_group = Some("direct".to_string());
        }

        Ok(())
    }

    /// 保存配置到文件
    pub fn save(&self) -> AppResult<()> {
        let config_path = Self::get_config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// 添加新的Direct配置（保持向后兼容）
    #[allow(dead_code)]
    pub fn add_profile(&mut self, name: String, profile: Profile) -> AppResult<()> {
        self.add_direct_profile(name, profile)
    }

    /// 添加Direct配置
    pub fn add_direct_profile(&mut self, name: String, profile: DirectProfile) -> AppResult<()> {
        if self.groups.direct.contains_key(&name) {
            return Err(AppError::Config(format!("配置 '{name}' 已存在")));
        }

        // 验证配置
        self.validate_direct_profile(&profile)?;

        self.groups.direct.insert(name.clone(), profile);

        // 如果这是第一个配置，设为默认
        if self.groups.direct.len() == 1 {
            if let Some(ref mut default_profile) = self.default_profile {
                default_profile.direct = Some(name);
            } else {
                self.default_profile = Some(DefaultProfile {
                    direct: Some(name),
                    ccr: None,
                });
            }
        }

        Ok(())
    }

    /// 添加CCR配置
    pub fn add_ccr_profile(&mut self, name: String, profile: CcrProfile) -> AppResult<()> {
        if self.groups.ccr.contains_key(&name) {
            return Err(AppError::Config(format!("配置 '{name}' 已存在")));
        }

        // 验证配置
        self.validate_ccr_profile(&profile)?;

        self.groups.ccr.insert(name.clone(), profile);

        // 如果这是第一个CCR配置，设为默认
        if self.groups.ccr.len() == 1 {
            if let Some(ref mut default_profile) = self.default_profile {
                default_profile.ccr = Some(name);
            } else {
                self.default_profile = Some(DefaultProfile {
                    direct: None,
                    ccr: Some(name),
                });
            }
        }

        Ok(())
    }

    /// 删除配置（自动检测组类型）
    pub fn remove_profile(&mut self, name: &str) -> AppResult<()> {
        // 先尝试从direct组删除
        if self.groups.direct.contains_key(name) {
            return self.remove_direct_profile(name);
        }

        // 再尝试从ccr组删除
        if self.groups.ccr.contains_key(name) {
            return self.remove_ccr_profile(name);
        }

        Err(AppError::ProfileNotFound(name.to_string()))
    }

    /// 从指定组删除配置
    #[allow(dead_code)]
    pub fn remove_profile_from_group(&mut self, group: &str, name: &str) -> AppResult<()> {
        match group {
            "direct" => self.remove_direct_profile(name),
            "ccr" => self.remove_ccr_profile(name),
            _ => Err(AppError::Config(format!("未知的配置组: {group}"))),
        }
    }

    /// 删除Direct配置
    pub fn remove_direct_profile(&mut self, name: &str) -> AppResult<()> {
        if !self.groups.direct.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        self.groups.direct.remove(name);

        // 如果删除的是默认配置，选择新的默认配置
        if let Some(ref mut default_profile) = self.default_profile {
            if default_profile.direct.as_ref() == Some(&name.to_string()) {
                default_profile.direct = self.groups.direct.keys().next().cloned();
            }
        }

        Ok(())
    }

    /// 删除CCR配置
    pub fn remove_ccr_profile(&mut self, name: &str) -> AppResult<()> {
        if !self.groups.ccr.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        self.groups.ccr.remove(name);

        // 如果删除的是默认配置，选择新的默认配置
        if let Some(ref mut default_profile) = self.default_profile {
            if default_profile.ccr.as_ref() == Some(&name.to_string()) {
                default_profile.ccr = self.groups.ccr.keys().next().cloned();
            }
        }

        Ok(())
    }

    /// 获取指定配置（向后兼容，优先从direct组查找）
    pub fn get_profile(&self, name: &str) -> AppResult<&Profile> {
        self.get_direct_profile(name)
    }

    /// 获取Direct配置
    pub fn get_direct_profile(&self, name: &str) -> AppResult<&DirectProfile> {
        self.groups
            .direct
            .get(name)
            .ok_or_else(|| AppError::ProfileNotFound(name.to_string()))
    }

    /// 获取CCR配置
    pub fn get_ccr_profile(&self, name: &str) -> AppResult<&CcrProfile> {
        self.groups
            .ccr
            .get(name)
            .ok_or_else(|| AppError::ProfileNotFound(name.to_string()))
    }

    /// 从指定组获取配置
    #[allow(dead_code)]
    pub fn get_profile_from_group(&self, group: &str, name: &str) -> AppResult<(&str, String)> {
        match group {
            "direct" => {
                if self.groups.direct.contains_key(name) {
                    Ok(("direct", name.to_string()))
                } else {
                    Err(AppError::ProfileNotFound(name.to_string()))
                }
            }
            "ccr" => {
                if self.groups.ccr.contains_key(name) {
                    Ok(("ccr", name.to_string()))
                } else {
                    Err(AppError::ProfileNotFound(name.to_string()))
                }
            }
            _ => Err(AppError::Config(format!("未知的配置组: {group}"))),
        }
    }

    /// 获取默认配置（向后兼容，优先返回direct组）
    pub fn get_default_profile(&self) -> AppResult<(&String, &Profile)> {
        let (default_name, _) = self.get_default_direct_profile()?;
        let profile = self.get_direct_profile(default_name)?;
        Ok((default_name, profile))
    }

    /// 获取默认的Direct配置
    pub fn get_default_direct_profile(&self) -> AppResult<(&String, &DirectProfile)> {
        let default_name = self
            .default_profile
            .as_ref()
            .and_then(|dp| dp.direct.as_ref())
            .ok_or_else(|| AppError::Config("未设置默认Direct配置".to_string()))?;

        let profile = self.get_direct_profile(default_name)?;
        Ok((default_name, profile))
    }

    /// 获取默认的CCR配置
    pub fn get_default_ccr_profile(&self) -> AppResult<(&String, &CcrProfile)> {
        let default_name = self
            .default_profile
            .as_ref()
            .and_then(|dp| dp.ccr.as_ref())
            .ok_or_else(|| AppError::Config("未设置默认CCR配置".to_string()))?;

        let profile = self.get_ccr_profile(default_name)?;
        Ok((default_name, profile))
    }

    /// 获取指定组的默认配置
    #[allow(dead_code)]
    pub fn get_default_profile_from_group(&self, group: &str) -> AppResult<(String, String)> {
        match group {
            "direct" => {
                let (name, _) = self.get_default_direct_profile()?;
                Ok(("direct".to_string(), name.clone()))
            }
            "ccr" => {
                let (name, _) = self.get_default_ccr_profile()?;
                Ok(("ccr".to_string(), name.clone()))
            }
            _ => Err(AppError::Config(format!("未知的配置组: {group}"))),
        }
    }

    /// 设置默认配置（向后兼容，优先设置direct组）
    pub fn set_default(&mut self, name: &str) -> AppResult<()> {
        if self.groups.direct.contains_key(name) {
            return self.set_default_direct(name);
        }

        if self.groups.ccr.contains_key(name) {
            return self.set_default_ccr(name);
        }

        Err(AppError::ProfileNotFound(name.to_string()))
    }

    /// 设置默认Direct配置
    pub fn set_default_direct(&mut self, name: &str) -> AppResult<()> {
        if !self.groups.direct.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        if let Some(ref mut default_profile) = self.default_profile {
            default_profile.direct = Some(name.to_string());
        } else {
            self.default_profile = Some(DefaultProfile {
                direct: Some(name.to_string()),
                ccr: None,
            });
        }
        Ok(())
    }

    /// 设置默认CCR配置
    pub fn set_default_ccr(&mut self, name: &str) -> AppResult<()> {
        if !self.groups.ccr.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        if let Some(ref mut default_profile) = self.default_profile {
            default_profile.ccr = Some(name.to_string());
        } else {
            self.default_profile = Some(DefaultProfile {
                direct: None,
                ccr: Some(name.to_string()),
            });
        }
        Ok(())
    }

    /// 从指定组设置默认配置
    #[allow(dead_code)]
    pub fn set_default_from_group(&mut self, group: &str, name: &str) -> AppResult<()> {
        match group {
            "direct" => self.set_default_direct(name),
            "ccr" => self.set_default_ccr(name),
            _ => Err(AppError::Config(format!("未知的配置组: {group}"))),
        }
    }

    /// 列出所有配置（向后兼容，只返回direct组）
    #[allow(dead_code)]
    pub fn list_profiles(&self) -> Vec<(String, &Profile, bool)> {
        self.list_direct_profiles()
    }

    /// 列出Direct配置
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

    /// 列出CCR配置
    pub fn list_ccr_profiles(&self) -> Vec<(String, &CcrProfile, bool)> {
        let default_name = self.default_profile.as_ref().and_then(|dp| dp.ccr.as_ref());

        self.groups
            .ccr
            .iter()
            .map(|(name, profile)| {
                let is_default = default_name == Some(name);
                (name.clone(), profile, is_default)
            })
            .collect()
    }

    /// 列出指定组的配置
    #[allow(dead_code)]
    pub fn list_profiles_from_group(&self, group: &str) -> AppResult<Vec<(String, bool)>> {
        match group {
            "direct" => {
                let profiles = self.list_direct_profiles();
                Ok(profiles
                    .into_iter()
                    .map(|(name, _, is_default)| (name, is_default))
                    .collect())
            }
            "ccr" => {
                let profiles = self.list_ccr_profiles();
                Ok(profiles
                    .into_iter()
                    .map(|(name, _, is_default)| (name, is_default))
                    .collect())
            }
            _ => Err(AppError::Config(format!("未知的配置组: {group}"))),
        }
    }

    /// 列出所有组的配置
    #[allow(dead_code)]
    pub fn list_all_profiles(&self) -> Vec<(String, String, bool)> {
        let mut all_profiles = Vec::new();

        // 添加direct组配置
        for (name, is_default) in self.list_profiles_from_group("direct").unwrap_or_default() {
            all_profiles.push(("direct".to_string(), name, is_default));
        }

        // 添加ccr组配置
        for (name, is_default) in self.list_profiles_from_group("ccr").unwrap_or_default() {
            all_profiles.push(("ccr".to_string(), name, is_default));
        }

        all_profiles
    }

    /// 验证配置有效性（向后兼容）
    #[allow(dead_code)]
    fn validate_profile(&self, profile: &Profile) -> AppResult<()> {
        self.validate_direct_profile(profile)
    }

    /// 验证Direct配置有效性
    fn validate_direct_profile(&self, profile: &DirectProfile) -> AppResult<()> {
        // 验证token格式
        if profile.anthropic_auth_token.trim().is_empty() {
            return Err(AppError::InvalidConfig("认证令牌不能为空".to_string()));
        }

        // 验证URL格式
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

    /// 验证CCR配置有效性
    fn validate_ccr_profile(&self, profile: &CcrProfile) -> AppResult<()> {
        // 验证providers不为空
        if profile.providers.is_empty() {
            return Err(AppError::InvalidConfig("CCR提供商列表不能为空".to_string()));
        }

        // 验证每个provider
        for provider in &profile.providers {
            if provider.name.trim().is_empty() {
                return Err(AppError::InvalidConfig("提供商名称不能为空".to_string()));
            }

            if provider.api_base_url.trim().is_empty() {
                return Err(AppError::InvalidConfig("提供商API URL不能为空".to_string()));
            }

            if !provider.api_base_url.starts_with("http://")
                && !provider.api_base_url.starts_with("https://")
            {
                return Err(AppError::InvalidConfig(
                    "提供商API URL格式无效，应以 'http://' 或 'https://' 开头".to_string(),
                ));
            }

            if provider.models.is_empty() {
                return Err(AppError::InvalidConfig(format!(
                    "提供商 '{}' 的模型列表不能为空",
                    provider.name
                )));
            }
        }

        // 验证router默认配置
        if profile.router.default.trim().is_empty() {
            return Err(AppError::InvalidConfig(
                "CCR路由默认配置不能为空".to_string(),
            ));
        }

        // 验证默认路由格式（应该是 "provider,model" 格式）
        if !profile.router.default.contains(',') {
            return Err(AppError::InvalidConfig(
                "CCR路由默认配置格式无效，应为 'provider,model' 格式".to_string(),
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
            description: Some("Test profile".to_string()),
            created_at: Some("2025-07-29T00:00:00Z".to_string()),
        }
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.default, None);
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_profile_creation() {
        let profile = create_test_profile();
        assert_eq!(profile.anthropic_auth_token, "test-token-123");
        assert_eq!(profile.anthropic_base_url, "https://api.anthropic.com");
        assert_eq!(profile.description, Some("Test profile".to_string()));
    }

    #[test]
    fn test_add_profile() {
        let mut config = Config::default();
        let profile = create_test_profile();

        let result = config.add_profile("test".to_string(), profile);
        assert!(result.is_ok());
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.default, Some("test".to_string()));
    }

    #[test]
    fn test_add_duplicate_profile() {
        let mut config = Config::default();
        let profile = create_test_profile();

        config
            .add_profile("test".to_string(), profile.clone())
            .unwrap();
        let result = config.add_profile("test".to_string(), profile);

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Config(msg) => assert!(msg.contains("已存在")),
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_remove_profile() {
        let mut config = Config::default();
        let profile = create_test_profile();

        config.add_profile("test".to_string(), profile).unwrap();
        assert_eq!(config.profiles.len(), 1);

        let result = config.remove_profile("test");
        assert!(result.is_ok());
        assert!(config.profiles.is_empty());
        assert_eq!(config.default, None);
    }

    #[test]
    fn test_remove_nonexistent_profile() {
        let mut config = Config::default();
        let result = config.remove_profile("nonexistent");

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ProfileNotFound(name) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected ProfileNotFound error"),
        }
    }

    #[test]
    fn test_get_profile() {
        let mut config = Config::default();
        let profile = create_test_profile();

        config.add_profile("test".to_string(), profile).unwrap();

        let result = config.get_profile("test");
        assert!(result.is_ok());
        let retrieved_profile = result.unwrap();
        assert_eq!(retrieved_profile.anthropic_auth_token, "test-token-123");
    }

    #[test]
    fn test_set_default() {
        let mut config = Config::default();
        let profile = create_test_profile();

        config.add_profile("test".to_string(), profile).unwrap();
        config
            .add_profile("test2".to_string(), create_test_profile())
            .unwrap();

        let result = config.set_default("test2");
        assert!(result.is_ok());
        assert_eq!(config.default, Some("test2".to_string()));
    }

    #[test]
    fn test_validate_profile_empty_token() {
        let config = Config::default();
        let mut profile = create_test_profile();
        profile.anthropic_auth_token = "".to_string();

        let result = config.validate_profile(&profile);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidConfig(msg) => assert!(msg.contains("认证令牌不能为空")),
            _ => panic!("Expected InvalidConfig error"),
        }
    }

    #[test]
    fn test_validate_profile_invalid_url() {
        let config = Config::default();
        let mut profile = create_test_profile();
        profile.anthropic_base_url = "invalid-url".to_string();

        let result = config.validate_profile(&profile);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidConfig(msg) => assert!(msg.contains("基础URL格式无效")),
            _ => panic!("Expected InvalidConfig error"),
        }
    }

    #[test]
    fn test_list_profiles() {
        let mut config = Config::default();
        let profile1 = create_test_profile();
        let profile2 = create_test_profile();

        config.add_profile("test1".to_string(), profile1).unwrap();
        config.add_profile("test2".to_string(), profile2).unwrap();

        let profiles = config.list_profiles();
        assert_eq!(profiles.len(), 2);

        // 检查默认配置标记
        let default_count = profiles
            .iter()
            .filter(|(_, _, is_default)| *is_default)
            .count();
        assert_eq!(default_count, 1);
    }
}
