use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 配置文件中的单个配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "ANTHROPIC_AUTH_TOKEN")]
    pub anthropic_auth_token: String,
    #[serde(rename = "ANTHROPIC_BASE_URL")]
    pub anthropic_base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    pub profiles: HashMap<String, Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            default: None,
            profiles: HashMap::new(),
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
        let config: Config = serde_json::from_str(&content)?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self) -> AppResult<()> {
        let config_path = Self::get_config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// 添加新配置
    pub fn add_profile(&mut self, name: String, profile: Profile) -> AppResult<()> {
        if self.profiles.contains_key(&name) {
            return Err(AppError::Config(format!("配置 '{}' 已存在", name)));
        }

        // 验证配置
        self.validate_profile(&profile)?;

        self.profiles.insert(name.clone(), profile);

        // 如果这是第一个配置，设为默认
        if self.profiles.len() == 1 {
            self.default = Some(name);
        }

        Ok(())
    }

    /// 删除配置
    pub fn remove_profile(&mut self, name: &str) -> AppResult<()> {
        if !self.profiles.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        self.profiles.remove(name);

        // 如果删除的是默认配置，清除默认设置
        if self.default.as_ref() == Some(&name.to_string()) {
            self.default = self.profiles.keys().next().cloned();
        }

        Ok(())
    }

    /// 获取指定配置
    pub fn get_profile(&self, name: &str) -> AppResult<&Profile> {
        self.profiles
            .get(name)
            .ok_or_else(|| AppError::ProfileNotFound(name.to_string()))
    }

    /// 获取默认配置
    pub fn get_default_profile(&self) -> AppResult<(&String, &Profile)> {
        let default_name = self
            .default
            .as_ref()
            .ok_or_else(|| AppError::Config("未设置默认配置".to_string()))?;

        let profile = self.get_profile(default_name)?;
        Ok((default_name, profile))
    }

    /// 设置默认配置
    pub fn set_default(&mut self, name: &str) -> AppResult<()> {
        if !self.profiles.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        self.default = Some(name.to_string());
        Ok(())
    }

    /// 列出所有配置
    pub fn list_profiles(&self) -> Vec<(String, &Profile, bool)> {
        self.profiles
            .iter()
            .map(|(name, profile)| {
                let is_default = self.default.as_ref() == Some(name);
                (name.clone(), profile, is_default)
            })
            .collect()
    }

    /// 验证配置有效性
    fn validate_profile(&self, profile: &Profile) -> AppResult<()> {
        // 验证token格式
        if profile.anthropic_auth_token.trim().is_empty() {
            return Err(AppError::InvalidConfig("认证令牌不能为空".to_string()));
        }

        if !profile.anthropic_auth_token.starts_with("sk-") {
            return Err(AppError::InvalidConfig(
                "认证令牌格式无效，应以 'sk-' 开头".to_string(),
            ));
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
}
