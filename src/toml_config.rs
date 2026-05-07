use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TomlProfile {
    #[allow(dead_code)]
    pub name: Option<String>,
    pub base_url: String,
    pub env_key: String,
    pub model: Option<String>,
    pub model_haiku: Option<String>,
    pub model_sonnet: Option<String>,
    pub model_opus: Option<String>,
    pub model_subagent: Option<String>,
    pub max_tokens: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TomlConfig {
    pub default: Option<String>,
    pub profiles: HashMap<String, TomlProfile>,
}

impl TomlConfig {
    /// 配置文件路径：~/.config/ccode/config.toml
    pub fn get_config_path() -> AppResult<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| AppError::Config("无法获取配置目录".to_string()))?;
        let ccode_dir = config_dir.join("ccode");
        if !ccode_dir.exists() {
            fs::create_dir_all(&ccode_dir)?;
        }
        Ok(ccode_dir.join("config.toml"))
    }

    // 已不需要 exists()，统一通过 load_or_default()/load 判断

    pub fn load() -> AppResult<Self> {
        let path = Self::get_config_path()?;
        if !path.exists() {
            return Err(AppError::ConfigNotFound);
        }
        let content = fs::read_to_string(&path)?;
        let cfg: TomlConfig = toml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn load_or_default() -> AppResult<Self> {
        match Self::load() {
            Ok(c) => Ok(c),
            Err(AppError::ConfigNotFound) => Ok(TomlConfig {
                default: None,
                profiles: HashMap::new(),
            }),
            Err(e) => Err(e),
        }
    }

    pub fn save(&self) -> AppResult<()> {
        let path = Self::get_config_path()?;
        let content = toml::to_string_pretty(self).map_err(|e| AppError::Toml(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn validate_profile(profile: &TomlProfile) -> AppResult<()> {
        if profile.base_url.trim().is_empty() {
            return Err(AppError::InvalidConfig("base_url 不能为空".to_string()));
        }
        if !(profile.base_url.starts_with("http://") || profile.base_url.starts_with("https://")) {
            return Err(AppError::InvalidConfig(
                "base_url 必须以 http:// 或 https:// 开头".to_string(),
            ));
        }
        if profile.env_key.trim().is_empty() {
            return Err(AppError::InvalidConfig("env_key 不能为空".to_string()));
        }
        Ok(())
    }

    pub fn add_profile(&mut self, name: &str, mut profile: TomlProfile) -> AppResult<()> {
        if self.profiles.contains_key(name) {
            return Err(AppError::Config(format!("配置 '{name}' 已存在")));
        }
        Self::validate_profile(&profile)?;
        // 写回 name 字段以保持自描述
        profile.name = Some(name.to_string());
        self.profiles.insert(name.to_string(), profile);
        // 首个配置自动设为默认
        if self.default.is_none() {
            self.default = Some(name.to_string());
        }
        Ok(())
    }

    pub fn set_default(&mut self, name: &str) -> AppResult<()> {
        if !self.profiles.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }
        self.default = Some(name.to_string());
        Ok(())
    }

    pub fn remove_profile(&mut self, name: &str) -> AppResult<()> {
        if self.profiles.remove(name).is_none() {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }
        if let Some(d) = &self.default
            && d == name
        {
            let next = self.profiles.keys().next().cloned();
            self.default = next;
        }
        Ok(())
    }

    pub fn get_profile<'a>(&'a self, name: Option<&str>) -> AppResult<(&'a str, &'a TomlProfile)> {
        if let Some(name) = name {
            if let Some((k, v)) = self.profiles.get_key_value(name) {
                return Ok((k.as_str(), v));
            }
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        let default_name = self
            .default
            .as_ref()
            .ok_or_else(|| AppError::Config("未设置默认 profile".to_string()))?;
        if let Some((k, v)) = self.profiles.get_key_value(default_name) {
            return Ok((k.as_str(), v));
        }
        Err(AppError::ProfileNotFound(default_name.to_string()))
    }
}

/// 从 config.toml 同目录加载 .env 并读取 env_key 对应的 token
pub fn load_token_from_env(ccode_toml_path: &Path, env_key: &str) -> AppResult<String> {
    // 先尝试加载同级 .env（不报错，静默）
    if let Some(dir) = ccode_toml_path.parent() {
        let dotenv_path = dir.join(".env");
        if dotenv_path.exists() {
            let _ = dotenvy::from_path_override(&dotenv_path);
        }
    }

    // 从进程环境读取
    match std::env::var(env_key) {
        Ok(val) if !val.trim().is_empty() => Ok(val),
        _ => Err(AppError::Config(format!(
            "未在系统环境或 .env 中找到环境变量 '{env_key}'"
        ))),
    }
}

/// 根据 profile 名称生成 env_key（小写、非字母数字转为下划线，并追加 _key）
pub fn derive_env_key_from_profile(name: &str) -> String {
    let mut s = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .to_lowercase();
    if !s.ends_with("_key") {
        s.push_str("_key");
    }
    s
}

/// 将 token 持久化写入 ~/.config/ccode/.env 中对应 env_key（存在则更新，缺失则追加）
pub fn persist_token_to_env(ccode_toml_path: &Path, env_key: &str, token: &str) -> AppResult<()> {
    let dir = ccode_toml_path
        .parent()
        .ok_or_else(|| AppError::Config("无法定位配置目录".to_string()))?;
    let dotenv_path = dir.join(".env");

    let mut lines: Vec<String> = if dotenv_path.exists() {
        fs::read_to_string(&dotenv_path)
            .unwrap_or_default()
            .lines()
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };

    let quoted = format!("\"{}\"", token.replace('"', "\\\""));
    let assignment = format!("{}={}", env_key, quoted);

    let mut updated = false;
    for line in lines.iter_mut() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find('=') {
            let (k, _) = trimmed.split_at(pos);
            if k.trim() == env_key {
                *line = assignment.clone();
                updated = true;
                break;
            }
        }
    }
    if !updated {
        lines.push(assignment);
    }

    let content = lines.join("\n");
    fs::write(&dotenv_path, content)?;
    Ok(())
}
