use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

/// Provider类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    /// OpenAI兼容格式（无需特殊transformer）
    #[serde(rename = "openai")]
    OpenAI,
    /// OpenRouter
    #[serde(rename = "openrouter")]
    OpenRouter,
    /// DeepSeek
    #[serde(rename = "deepseek")]
    DeepSeek,
    /// Gemini
    #[serde(rename = "gemini")]
    Gemini,
    /// Qwen系列
    #[serde(rename = "qwen")]
    Qwen,
    /// 自定义类型
    #[serde(rename = "custom")]
    Custom,
}

impl ProviderType {
    /// 获取provider类型的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderType::OpenAI => "OpenAI兼容",
            ProviderType::OpenRouter => "OpenRouter",
            ProviderType::DeepSeek => "DeepSeek",
            ProviderType::Gemini => "Gemini",
            ProviderType::Qwen => "Qwen",
            ProviderType::Custom => "自定义",
        }
    }

    /// 获取默认的API URL格式提示
    pub fn url_format_hint(&self) -> &'static str {
        match self {
            ProviderType::OpenAI => "https://api.openai.com/v1/chat/completions",
            ProviderType::OpenRouter => "https://openrouter.ai/api/v1/chat/completions",
            ProviderType::DeepSeek => "https://api.deepseek.com/chat/completions",
            ProviderType::Gemini => "https://generativelanguage.googleapis.com/v1beta/models/",
            ProviderType::Qwen => {
                "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
            }
            ProviderType::Custom => "https://your-api-url/v1/chat/completions",
        }
    }

    /// 生成对应的transformer配置
    pub fn generate_transformer(&self, models: &[String]) -> Option<serde_json::Value> {
        match self {
            ProviderType::OpenAI => None, // OpenAI兼容格式无需transformer
            ProviderType::OpenRouter => Some(json!({
                "use": ["openrouter"]
            })),
            ProviderType::DeepSeek => {
                let mut transformer = json!({
                    "use": ["deepseek"]
                });

                // 为deepseek-chat模型添加tooluse transformer
                let mut model_specific = serde_json::Map::new();
                for model in models {
                    if model.contains("deepseek-chat") {
                        model_specific.insert(model.clone(), json!({"use": ["tooluse"]}));
                    }
                }

                if !model_specific.is_empty() {
                    if let Some(obj) = transformer.as_object_mut() {
                        for (key, value) in model_specific {
                            obj.insert(key, value);
                        }
                    }
                }

                Some(transformer)
            }
            ProviderType::Gemini => Some(json!({
                "use": ["gemini"]
            })),
            ProviderType::Qwen => {
                let mut transformer = json!({
                    "use": [
                        ["maxtoken", {"max_tokens": 65536}],
                        "enhancetool"
                    ]
                });

                // 为Thinking模型添加reasoning transformer
                let mut model_specific = serde_json::Map::new();
                for model in models {
                    if model.contains("Thinking") || model.contains("thinking") {
                        model_specific.insert(model.clone(), json!({"use": ["reasoning"]}));
                    }
                }

                if !model_specific.is_empty() {
                    if let Some(obj) = transformer.as_object_mut() {
                        for (key, value) in model_specific {
                            obj.insert(key, value);
                        }
                    }
                }

                Some(transformer)
            }
            ProviderType::Custom => None, // 自定义类型不自动生成transformer
        }
    }

    /// 验证API URL格式是否符合provider类型
    pub fn validate_url_format(&self, url: &str) -> AppResult<()> {
        match self {
            ProviderType::Gemini => {
                if !url.contains("/v1beta/models/") {
                    return Err(AppError::InvalidConfig(
                        "Gemini API URL应包含'/v1beta/models/'路径".to_string(),
                    ));
                }
            }
            _ => {
                // 其他类型检查是否包含chat/completions
                if !url.contains("/chat/completions") && *self != ProviderType::Custom {
                    return Err(AppError::InvalidConfig(
                        "API URL应包含'/chat/completions'路径".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// CCR提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcrProvider {
    pub name: String,
    pub api_base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformer: Option<serde_json::Value>,
    /// Provider类型（用于生成transformer配置）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<ProviderType>,
}

impl CcrProvider {
    /// 创建新的Provider配置
    pub fn new(
        name: String,
        api_base_url: String,
        api_key: String,
        models: Vec<String>,
        provider_type: ProviderType,
    ) -> Self {
        let transformer = provider_type.generate_transformer(&models);

        Self {
            name,
            api_base_url,
            api_key,
            models,
            transformer,
            provider_type: Some(provider_type),
        }
    }

    /// 验证配置有效性
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::InvalidConfig("提供商名称不能为空".to_string()));
        }

        if self.api_base_url.trim().is_empty() {
            return Err(AppError::InvalidConfig("API URL不能为空".to_string()));
        }

        if !self.api_base_url.starts_with("http://") && !self.api_base_url.starts_with("https://") {
            return Err(AppError::InvalidConfig(
                "API URL格式无效，应以'http://'或'https://'开头".to_string(),
            ));
        }

        if self.models.is_empty() {
            return Err(AppError::InvalidConfig("模型列表不能为空".to_string()));
        }

        // 验证URL格式是否符合provider类型
        if let Some(provider_type) = &self.provider_type {
            provider_type.validate_url_format(&self.api_base_url)?;
        }

        Ok(())
    }
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

impl CcrRouter {
    /// 创建新的Router配置
    pub fn new(default: String) -> Self {
        Self {
            default,
            background: None,
            think: None,
            long_context: None,
            long_context_threshold: Some(60000), // 默认60000
            web_search: None,
        }
    }

    /// 验证路由配置有效性
    pub fn validate(&self) -> AppResult<()> {
        if self.default.trim().is_empty() {
            return Err(AppError::InvalidConfig("默认路由配置不能为空".to_string()));
        }

        // 验证默认路由格式（应该是 "provider,model" 格式）
        if !self.default.contains(',') {
            return Err(AppError::InvalidConfig(
                "默认路由配置格式无效，应为'provider,model'格式".to_string(),
            ));
        }

        // 验证其他路由配置格式
        let routes = [
            ("background", &self.background),
            ("think", &self.think),
            ("longContext", &self.long_context),
            ("webSearch", &self.web_search),
        ];

        for (name, route) in routes.iter() {
            if let Some(route_value) = route {
                if !route_value.trim().is_empty() && !route_value.contains(',') {
                    return Err(AppError::InvalidConfig(format!(
                        "{name}路由配置格式无效，应为'provider,model'格式"
                    )));
                }
            }
        }

        Ok(())
    }

    /// 获取所有配置的路由
    pub fn get_all_routes(&self) -> Vec<(String, String)> {
        let mut routes = vec![("default".to_string(), self.default.clone())];

        if let Some(background) = &self.background {
            routes.push(("background".to_string(), background.clone()));
        }
        if let Some(think) = &self.think {
            routes.push(("think".to_string(), think.clone()));
        }
        if let Some(long_context) = &self.long_context {
            routes.push(("longContext".to_string(), long_context.clone()));
        }
        if let Some(web_search) = &self.web_search {
            routes.push(("webSearch".to_string(), web_search.clone()));
        }

        routes
    }
}

/// Provider模板生成器
impl ProviderType {
    /// 获取默认的模型列表
    pub fn get_default_models(&self) -> Vec<String> {
        match self {
            ProviderType::OpenAI => vec![
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            ProviderType::OpenRouter => vec![
                "anthropic/claude-3.5-sonnet".to_string(),
                "google/gemini-2.5-pro-preview".to_string(),
                "anthropic/claude-sonnet-4".to_string(),
            ],
            ProviderType::DeepSeek => {
                vec!["deepseek-chat".to_string(), "deepseek-reasoner".to_string()]
            }
            ProviderType::Gemini => {
                vec!["gemini-2.5-flash".to_string(), "gemini-2.5-pro".to_string()]
            }
            ProviderType::Qwen => vec![
                "qwen3-coder-plus".to_string(),
                "Qwen/Qwen3-Coder-480B-A35B-Instruct".to_string(),
                "Qwen/Qwen3-235B-A22B-Thinking-2507".to_string(),
            ],
            ProviderType::Custom => vec!["custom-model".to_string()],
        }
    }

    /// 获取provider类型的配置提示信息
    pub fn get_configuration_hints(&self) -> Vec<&'static str> {
        match self {
            ProviderType::OpenAI => vec![
                "• 标准OpenAI API格式",
                "• 无需特殊transformer配置",
                "• 支持大部分第三方兼容API",
            ],
            ProviderType::OpenRouter => vec![
                "• 支持多种AI模型路由",
                "• 自动添加OpenRouter transformer",
                "• WebSearch功能需要在模型后加':online'后缀",
            ],
            ProviderType::DeepSeek => vec![
                "• DeepSeek专用API",
                "• 自动配置DeepSeek transformer",
                "• deepseek-chat模型自动启用tooluse",
            ],
            ProviderType::Gemini => vec![
                "• Google Gemini API",
                "• API路径格式: /v1beta/models/",
                "• 自动配置Gemini transformer",
            ],
            ProviderType::Qwen => vec![
                "• 通义千问系列模型",
                "• 自动配置最大token限制(65536)",
                "• Thinking模型自动启用reasoning模式",
                "• 增强的工具调用支持",
            ],
            ProviderType::Custom => vec![
                "• 自定义API配置",
                "• 需要手动配置transformer(如需要)",
                "• 适用于其他AI服务提供商",
            ],
        }
    }
}

/// 完整的 claude-code-router 配置文件结构
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcrConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub APIKEY: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub PROXY_URL: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub LOG: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub API_TIMEOUT_MS: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub HOST: Option<String>,
    pub Providers: Vec<CcrProvider>,
    pub Router: CcrRouter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformers: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub CUSTOM_ROUTER_PATH: Option<String>,
}

impl CcrConfig {
    /// 创建新的 CCR 配置文件
    pub fn new() -> Self {
        Self {
            APIKEY: None,
            PROXY_URL: None,
            LOG: Some(true),
            API_TIMEOUT_MS: Some(600000),
            HOST: None,
            Providers: Vec::new(),
            Router: CcrRouter::new("provider,model".to_string()),
            transformers: None,
            CUSTOM_ROUTER_PATH: None,
        }
    }

    /// 验证配置有效性
    pub fn validate(&self) -> AppResult<()> {
        // 验证 Providers 不为空
        if self.Providers.is_empty() {
            return Err(AppError::InvalidConfig("Providers列表不能为空".to_string()));
        }

        // 验证每个 Provider
        for provider in &self.Providers {
            provider.validate()?;
        }

        // 验证 Router 配置
        self.Router.validate()?;

        // 验证 Router 中引用的 provider 存在
        let provider_names: std::collections::HashSet<_> =
            self.Providers.iter().map(|p| p.name.as_str()).collect();

        for (route_name, route_value) in self.Router.get_all_routes() {
            if let Some(provider_name) = route_value.split(',').next() {
                if !provider_names.contains(provider_name) {
                    return Err(AppError::InvalidConfig(format!(
                        "路由'{route_name}'中的提供商'{provider_name}'不存在"
                    )));
                }
            }
        }

        Ok(())
    }

    /// 添加 Provider
    pub fn add_provider(&mut self, provider: CcrProvider) -> AppResult<()> {
        // 检查名称是否重复
        if self.Providers.iter().any(|p| p.name == provider.name) {
            return Err(AppError::Config(format!(
                "Provider '{}' 已存在",
                provider.name
            )));
        }

        provider.validate()?;
        self.Providers.push(provider);
        Ok(())
    }

    /// 删除 Provider
    pub fn remove_provider(&mut self, name: &str) -> AppResult<()> {
        let original_len = self.Providers.len();
        self.Providers.retain(|p| p.name != name);

        if self.Providers.len() == original_len {
            return Err(AppError::Config(format!("Provider '{name}' 不存在")));
        }

        Ok(())
    }

    /// 获取 Provider
    pub fn get_provider(&self, name: &str) -> Option<&CcrProvider> {
        self.Providers.iter().find(|p| p.name == name)
    }

    /// 更新 Provider
    pub fn update_provider(&mut self, provider: CcrProvider) -> AppResult<()> {
        provider.validate()?;

        if let Some(existing) = self.Providers.iter_mut().find(|p| p.name == provider.name) {
            *existing = provider;
            Ok(())
        } else {
            Err(AppError::Config(format!(
                "Provider '{}' 不存在",
                provider.name
            )))
        }
    }

    /// 更新 Router 配置
    pub fn update_router(&mut self, router: CcrRouter) -> AppResult<()> {
        router.validate()?;
        self.Router = router;
        Ok(())
    }
}

impl Default for CcrConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Router Profile - 路由配置预设
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterProfile {
    pub name: String,
    pub router: CcrRouter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl RouterProfile {
    /// 创建新的 Router Profile
    pub fn new(name: String, router: CcrRouter, description: Option<String>) -> AppResult<Self> {
        router.validate()?;

        Ok(Self {
            name,
            router,
            description,
            created_at: None,
        })
    }

    /// 验证配置有效性
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::InvalidConfig(
                "Router Profile 名称不能为空".to_string(),
            ));
        }

        self.router.validate()
    }
}

/// 默认配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultProfile {
    pub direct: Option<String>,
    /// 默认的 Router Profile
    pub router: Option<String>,
}

/// 配置组集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Groups {
    pub direct: HashMap<String, DirectProfile>,
    /// Router Profile 配置集合
    pub router: HashMap<String, RouterProfile>,
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
                router: None,
            }),
            groups: Groups {
                direct: HashMap::new(),
                router: HashMap::new(),
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
                    router: None,
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
                    router: None,
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

        Err(AppError::ProfileNotFound(name.to_string()))
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

    /// 获取Direct配置
    pub fn get_direct_profile(&self, name: &str) -> AppResult<&DirectProfile> {
        self.groups
            .direct
            .get(name)
            .ok_or_else(|| AppError::ProfileNotFound(name.to_string()))
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

    /// 设置默认配置（向后兼容，优先设置direct组）
    pub fn set_default(&mut self, name: &str) -> AppResult<()> {
        if self.groups.direct.contains_key(name) {
            return self.set_default_direct(name);
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
                router: None,
            });
        }
        Ok(())
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

    /// 添加 Router Profile
    pub fn add_router_profile(&mut self, name: String, profile: RouterProfile) -> AppResult<()> {
        if self.groups.router.contains_key(&name) {
            return Err(AppError::Config(format!("Router Profile '{name}' 已存在")));
        }

        // 验证配置
        profile.validate()?;

        self.groups.router.insert(name.clone(), profile);

        // 如果这是第一个 Router Profile，设为默认
        if self.groups.router.len() == 1 {
            if let Some(ref mut default_profile) = self.default_profile {
                default_profile.router = Some(name);
            } else {
                self.default_profile = Some(DefaultProfile {
                    direct: None,
                    router: Some(name),
                });
            }
        }

        Ok(())
    }

    /// 删除 Router Profile
    pub fn remove_router_profile(&mut self, name: &str) -> AppResult<()> {
        if !self.groups.router.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        self.groups.router.remove(name);

        // 如果删除的是默认配置，选择新的默认配置
        if let Some(ref mut default_profile) = self.default_profile {
            if default_profile.router.as_ref() == Some(&name.to_string()) {
                default_profile.router = self.groups.router.keys().next().cloned();
            }
        }

        Ok(())
    }

    /// 获取 Router Profile
    pub fn get_router_profile(&self, name: &str) -> AppResult<&RouterProfile> {
        self.groups
            .router
            .get(name)
            .ok_or_else(|| AppError::ProfileNotFound(name.to_string()))
    }

    /// 获取默认的 Router Profile
    pub fn get_default_router_profile(&self) -> AppResult<(&String, &RouterProfile)> {
        let default_name = self
            .default_profile
            .as_ref()
            .and_then(|dp| dp.router.as_ref())
            .ok_or_else(|| AppError::Config("未设置默认 Router Profile".to_string()))?;

        let profile = self.get_router_profile(default_name)?;
        Ok((default_name, profile))
    }

    /// 设置默认 Router Profile
    pub fn set_default_router(&mut self, name: &str) -> AppResult<()> {
        if !self.groups.router.contains_key(name) {
            return Err(AppError::ProfileNotFound(name.to_string()));
        }

        if let Some(ref mut default_profile) = self.default_profile {
            default_profile.router = Some(name.to_string());
        } else {
            self.default_profile = Some(DefaultProfile {
                direct: None,
                router: Some(name.to_string()),
            });
        }
        Ok(())
    }

    /// 列出 Router Profiles
    pub fn list_router_profiles(&self) -> Vec<(String, &RouterProfile, bool)> {
        let default_name = self
            .default_profile
            .as_ref()
            .and_then(|dp| dp.router.as_ref());

        self.groups
            .router
            .iter()
            .map(|(name, profile)| {
                let is_default = default_name == Some(name);
                (name.clone(), profile, is_default)
            })
            .collect()
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
        assert!(config.groups.direct.is_empty());
        assert!(config.groups.router.is_empty());
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

        let result = config.add_direct_profile("test".to_string(), profile);
        assert!(result.is_ok());
        assert_eq!(config.groups.direct.len(), 1);
        assert_eq!(
            config.default_profile.as_ref().unwrap().direct,
            Some("test".to_string())
        );
    }

    #[test]
    fn test_remove_profile() {
        let mut config = Config::default();
        let profile = create_test_profile();

        config
            .add_direct_profile("test".to_string(), profile)
            .unwrap();
        assert_eq!(config.groups.direct.len(), 1);

        let result = config.remove_profile("test");
        assert!(result.is_ok());
        assert!(config.groups.direct.is_empty());
        assert_eq!(config.default_profile.as_ref().unwrap().direct, None);
    }

    #[test]
    fn test_get_profile() {
        let mut config = Config::default();
        let profile = create_test_profile();

        config
            .add_direct_profile("test".to_string(), profile)
            .unwrap();

        let result = config.get_direct_profile("test");
        assert!(result.is_ok());
        let retrieved_profile = result.unwrap();
        assert_eq!(retrieved_profile.anthropic_auth_token, "test-token-123");
    }

    #[test]
    fn test_set_default() {
        let mut config = Config::default();
        let profile = create_test_profile();

        config
            .add_direct_profile("test".to_string(), profile)
            .unwrap();
        config
            .add_direct_profile("test2".to_string(), create_test_profile())
            .unwrap();

        let result = config.set_default("test2");
        assert!(result.is_ok());
        assert_eq!(
            config.default_profile.as_ref().unwrap().direct,
            Some("test2".to_string())
        );
    }

    #[test]
    fn test_list_profiles() {
        let mut config = Config::default();
        let profile1 = create_test_profile();
        let profile2 = create_test_profile();

        config
            .add_direct_profile("test1".to_string(), profile1)
            .unwrap();
        config
            .add_direct_profile("test2".to_string(), profile2)
            .unwrap();

        let profiles = config.list_direct_profiles();
        assert_eq!(profiles.len(), 2);

        let default_count = profiles
            .iter()
            .filter(|(_, _, is_default)| *is_default)
            .count();
        assert_eq!(default_count, 1);
    }
}
