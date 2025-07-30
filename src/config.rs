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

    /// 更新模型列表并重新生成transformer
    #[allow(dead_code)]
    pub fn update_models(&mut self, models: Vec<String>) {
        self.models = models;
        if let Some(provider_type) = &self.provider_type {
            self.transformer = provider_type.generate_transformer(&self.models);
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

    /// 设置background路由
    #[allow(dead_code)]
    pub fn with_background(mut self, background: Option<String>) -> Self {
        self.background = background;
        self
    }

    /// 设置think路由
    #[allow(dead_code)]
    pub fn with_think(mut self, think: Option<String>) -> Self {
        self.think = think;
        self
    }

    /// 设置longContext路由
    #[allow(dead_code)]
    pub fn with_long_context(mut self, long_context: Option<String>) -> Self {
        self.long_context = long_context;
        self
    }

    /// 设置longContextThreshold
    #[allow(dead_code)]
    pub fn with_long_context_threshold(mut self, threshold: Option<u32>) -> Self {
        self.long_context_threshold = threshold.or(Some(60000)); // 默认60000
        self
    }

    /// 设置webSearch路由
    #[allow(dead_code)]
    pub fn with_web_search(mut self, web_search: Option<String>) -> Self {
        self.web_search = web_search;
        self
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

    /// 应用默认值到未设置的路由
    pub fn apply_defaults(&mut self) {
        let default_route = self.default.clone();

        if self.background.is_none() || self.background.as_ref().is_none_or(|s| s.is_empty()) {
            self.background = Some(default_route.clone());
        }
        if self.think.is_none() || self.think.as_ref().is_none_or(|s| s.is_empty()) {
            self.think = Some(default_route.clone());
        }
        if self.long_context.is_none() || self.long_context.as_ref().is_none_or(|s| s.is_empty()) {
            self.long_context = Some(default_route.clone());
        }
        if self.web_search.is_none() || self.web_search.as_ref().is_none_or(|s| s.is_empty()) {
            self.web_search = Some(default_route);
        }

        // 确保longContextThreshold有值
        if self.long_context_threshold.is_none() {
            self.long_context_threshold = Some(60000);
        }
    }
}

/// CCR模式配置项（单provider模式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcrProfile {
    /// 单个provider配置
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

impl CcrProfile {
    /// 创建新的CCR配置（单provider模式）
    pub fn new(
        provider: CcrProvider,
        default_route: String,
        description: Option<String>,
    ) -> AppResult<Self> {
        // 验证provider
        provider.validate()?;

        // 创建Router配置
        let mut router = CcrRouter::new(default_route);
        router.apply_defaults(); // 应用默认值

        Ok(Self {
            providers: vec![provider],
            router,
            api_timeout_ms: Some(600000), // 默认10分钟
            proxy_url: None,
            log: Some(true), // 默认开启日志
            api_key: None,
            host: None,
            description,
            created_at: None,
        })
    }

    /// 获取主要的provider（在单provider模式下）
    pub fn get_primary_provider(&self) -> Option<&CcrProvider> {
        self.providers.first()
    }

    /// 更新provider信息
    #[allow(dead_code)]
    pub fn update_provider(&mut self, provider: CcrProvider) -> AppResult<()> {
        provider.validate()?;

        if self.providers.is_empty() {
            self.providers.push(provider);
        } else {
            self.providers[0] = provider;
        }

        Ok(())
    }

    /// 更新Router配置
    #[allow(dead_code)]
    pub fn update_router(&mut self, router: CcrRouter) -> AppResult<()> {
        router.validate()?;
        self.router = router;
        Ok(())
    }

    /// 验证配置有效性
    pub fn validate(&self) -> AppResult<()> {
        // 验证providers不为空
        if self.providers.is_empty() {
            return Err(AppError::InvalidConfig("CCR提供商列表不能为空".to_string()));
        }

        // 验证每个provider
        for provider in &self.providers {
            provider.validate()?;
        }

        // 验证router配置
        self.router.validate()?;

        // 验证router中的provider名称是否存在
        let provider_names: std::collections::HashSet<_> =
            self.providers.iter().map(|p| p.name.as_str()).collect();

        for (route_name, route_value) in self.router.get_all_routes() {
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

    /// 生成用于CCR的配置文件内容
    pub fn to_ccr_config(&self) -> serde_json::Value {
        let mut config = serde_json::json!({
            "Providers": self.providers,
            "Router": self.router
        });

        // 添加可选字段
        if let Some(timeout) = self.api_timeout_ms {
            config["API_TIMEOUT_MS"] = serde_json::Value::Number(timeout.into());
        }
        if let Some(proxy_url) = &self.proxy_url {
            config["PROXY_URL"] = serde_json::Value::String(proxy_url.clone());
        }
        if let Some(log) = self.log {
            config["LOG"] = serde_json::Value::Bool(log);
        }
        if let Some(api_key) = &self.api_key {
            config["APIKEY"] = serde_json::Value::String(api_key.clone());
        }
        if let Some(host) = &self.host {
            config["HOST"] = serde_json::Value::String(host.clone());
        }

        config
    }
}

/// Provider模板生成器
impl ProviderType {
    /// 创建provider模板（用于交互式配置）
    pub fn create_provider_template(
        &self,
        name: String,
        api_key: String,
        custom_url: Option<String>,
    ) -> AppResult<CcrProvider> {
        let api_base_url = custom_url.unwrap_or_else(|| self.get_default_api_url());
        let models = self.get_default_models();

        let provider = CcrProvider::new(name, api_base_url, api_key, models, self.clone());

        provider.validate()?;
        Ok(provider)
    }

    /// 获取默认的API URL
    fn get_default_api_url(&self) -> String {
        match self {
            ProviderType::OpenAI => "https://api.openai.com/v1/chat/completions".to_string(),
            ProviderType::OpenRouter => "https://openrouter.ai/api/v1/chat/completions".to_string(),
            ProviderType::DeepSeek => "https://api.deepseek.com/chat/completions".to_string(),
            ProviderType::Gemini => {
                "https://generativelanguage.googleapis.com/v1beta/models/".to_string()
            }
            ProviderType::Qwen => {
                "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string()
            }
            ProviderType::Custom => "https://your-api-url/v1/chat/completions".to_string(),
        }
    }

    /// 获取默认的模型列表
    fn get_default_models(&self) -> Vec<String> {
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

    /// 获取推荐的默认路由
    pub fn get_recommended_default_route(&self, provider_name: &str) -> String {
        let models = self.get_default_models();
        let default_model = models
            .first()
            .map(|s| s.as_str())
            .unwrap_or("default-model");
        format!("{provider_name},{default_model}")
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

/// CCR配置模板生成器
impl CcrProfile {
    /// 快速创建预设模板
    pub fn create_template(
        provider_type: ProviderType,
        name: String,
        api_key: String,
        custom_url: Option<String>,
        description: Option<String>,
    ) -> AppResult<Self> {
        // 创建provider
        let provider = provider_type.create_provider_template(name.clone(), api_key, custom_url)?;

        // 创建默认路由
        let default_route = provider_type.get_recommended_default_route(&name);

        // 创建CCR配置
        let mut profile = Self::new(provider, default_route, description)?;

        // 根据provider类型设置特定的配置选项
        match provider_type {
            ProviderType::DeepSeek => {
                // DeepSeek的think路由使用reasoner模型
                if let Some(provider) = profile.get_primary_provider() {
                    if provider.models.iter().any(|m| m.contains("reasoner")) {
                        profile.router.think = Some(format!("{},deepseek-reasoner", provider.name));
                    }
                }
            }
            ProviderType::OpenRouter => {
                // OpenRouter的长上下文设置
                if let Some(provider) = profile.get_primary_provider() {
                    if provider.models.iter().any(|m| m.contains("gemini-2.5-pro")) {
                        profile.router.long_context =
                            Some(format!("{},google/gemini-2.5-pro-preview", provider.name));
                    }
                }
            }
            ProviderType::Qwen => {
                // Qwen的API超时设置更长
                profile.api_timeout_ms = Some(900000); // 15分钟
            }
            _ => {}
        }

        Ok(profile)
    }

    /// 创建多provider组合模板
    #[allow(dead_code)]
    pub fn create_multi_provider_template(
        providers: Vec<(ProviderType, String, String, Option<String>)>, // (type, name, api_key, custom_url)
        description: Option<String>,
    ) -> AppResult<Self> {
        if providers.is_empty() {
            return Err(AppError::InvalidConfig("至少需要一个provider".to_string()));
        }

        // 创建第一个provider作为主要provider
        let (first_type, first_name, first_api_key, first_url) = &providers[0];
        let mut profile = Self::create_template(
            first_type.clone(),
            first_name.clone(),
            first_api_key.clone(),
            first_url.clone(),
            description,
        )?;

        // 添加其他providers
        for (provider_type, name, api_key, custom_url) in providers.iter().skip(1) {
            let provider = provider_type.create_provider_template(
                name.clone(),
                api_key.clone(),
                custom_url.clone(),
            )?;

            profile.providers.push(provider);
        }

        // 重新验证整个配置
        profile.validate()?;
        Ok(profile)
    }
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
        // 使用CcrProfile自己的验证方法
        profile.validate()
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
        assert!(config.groups.ccr.is_empty());
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
        assert_eq!(config.groups.direct.len(), 1);
        assert_eq!(
            config.default_profile.as_ref().unwrap().direct,
            Some("test".to_string())
        );
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
        assert_eq!(config.groups.direct.len(), 1);

        let result = config.remove_profile("test");
        assert!(result.is_ok());
        assert!(config.groups.direct.is_empty());
        assert_eq!(config.default_profile.as_ref().unwrap().direct, None);
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
        assert_eq!(
            config.default_profile.as_ref().unwrap().direct,
            Some("test2".to_string())
        );
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
