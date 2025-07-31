use crate::config::{CcrConfig, CcrProvider, CcrRouter, Config, RouterProfile};
use crate::error::{AppError, AppResult};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

/// CCR 配置文件直接管理器
pub struct CcrConfigManager {
    config_path: PathBuf,
    backup_dir: PathBuf,
}

impl CcrConfigManager {
    /// 创建新的 CCR 配置管理器
    pub fn new() -> AppResult<Self> {
        let config_path = Self::get_ccr_config_path()?;
        let backup_dir = Self::get_backup_dir()?;

        // 确保备份目录存在
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)?;
        }

        Ok(Self {
            config_path,
            backup_dir,
        })
    }

    /// 获取 CCR 配置文件路径
    fn get_ccr_config_path() -> AppResult<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| AppError::Config("无法获取用户主目录".to_string()))?;

        let ccr_dir = home_dir.join(".claude-code-router");

        // 确保 CCR 配置目录存在
        if !ccr_dir.exists() {
            fs::create_dir_all(&ccr_dir)?;
        }

        Ok(ccr_dir.join("config.json"))
    }

    /// 获取备份目录路径
    fn get_backup_dir() -> AppResult<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| AppError::Config("无法获取用户主目录".to_string()))?;

        Ok(home_dir.join(".claude-code-router").join("backups"))
    }

    /// 读取 CCR 配置文件
    pub fn load_config(&self) -> AppResult<CcrConfig> {
        if !self.config_path.exists() {
            // 如果配置文件不存在，返回默认配置
            return Ok(CcrConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: CcrConfig = serde_json::from_str(&content)
            .map_err(|e| AppError::Config(format!("解析 CCR 配置文件失败: {e}")))?;

        Ok(config)
    }

    /// 保存 CCR 配置文件
    pub fn save_config(&self, config: &CcrConfig) -> AppResult<()> {
        // 验证配置
        config.validate()?;

        // 如果配置文件已存在，先创建备份
        if self.config_path.exists() {
            self.create_backup()?;
        }

        // 写入配置文件
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;

        println!("✅ CCR 配置文件已保存: {}", self.config_path.display());
        Ok(())
    }

    /// 创建配置文件备份
    pub fn create_backup(&self) -> AppResult<String> {
        if !self.config_path.exists() {
            return Err(AppError::Config(
                "CCR 配置文件不存在，无法创建备份".to_string(),
            ));
        }

        // 生成备份文件名（带时间戳）
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("config_backup_{timestamp}.json");
        let backup_path = self.backup_dir.join(&backup_filename);

        // 复制配置文件到备份目录
        fs::copy(&self.config_path, &backup_path)?;

        println!("📦 配置备份已创建: {}", backup_path.display());
        Ok(backup_filename)
    }

    /// 检查配置文件是否存在
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }

    // ==================== Provider 管理方法 ====================

    /// 列出所有 Providers
    pub fn list_providers(&self) -> AppResult<Vec<CcrProvider>> {
        let config = self.load_config()?;
        Ok(config.Providers)
    }

    /// 添加 Provider
    pub fn add_provider(&self, provider: CcrProvider) -> AppResult<()> {
        let mut config = self.load_config()?;
        config.add_provider(provider)?;
        self.save_config(&config)?;
        Ok(())
    }

    /// 删除 Provider
    pub fn remove_provider(&self, name: &str) -> AppResult<()> {
        let mut config = self.load_config()?;
        config.remove_provider(name)?;
        self.save_config(&config)?;
        Ok(())
    }

    /// 获取指定 Provider
    pub fn get_provider(&self, name: &str) -> AppResult<CcrProvider> {
        let config = self.load_config()?;
        config
            .get_provider(name)
            .cloned()
            .ok_or_else(|| AppError::Config(format!("Provider '{name}' 不存在")))
    }

    /// 更新 Provider
    pub fn update_provider(&self, provider: CcrProvider) -> AppResult<()> {
        let mut config = self.load_config()?;
        config.update_provider(provider)?;
        self.save_config(&config)?;
        Ok(())
    }

    /// 检查 Provider 是否存在
    pub fn provider_exists(&self, name: &str) -> AppResult<bool> {
        let config = self.load_config()?;
        Ok(config.get_provider(name).is_some())
    }

    // ==================== Router 管理方法 ====================

    /// 获取当前 Router 配置
    pub fn get_current_router(&self) -> AppResult<CcrRouter> {
        let config = self.load_config()?;
        Ok(config.Router)
    }

    /// 应用 Router Profile 配置（只修改 Router 部分）
    pub fn apply_router_profile(&self, router_profile: &RouterProfile) -> AppResult<()> {
        let mut config = self.load_config()?;

        // 验证 router profile 中的 provider 引用是否存在
        let provider_names: std::collections::HashSet<_> =
            config.Providers.iter().map(|p| p.name.as_str()).collect();

        for (route_name, route_value) in router_profile.router.get_all_routes() {
            if let Some(provider_name) = route_value.split(',').next() {
                if !provider_names.contains(provider_name) {
                    return Err(AppError::InvalidConfig(format!(
                        "Router Profile '{}' 中的路由 '{}' 引用了不存在的提供商 '{}'",
                        router_profile.name, route_name, provider_name
                    )));
                }
            }
        }

        // 更新 Router 配置
        config.update_router(router_profile.router.clone())?;
        self.save_config(&config)?;

        println!("✅ 已应用 Router Profile '{}'", router_profile.name);
        Ok(())
    }

    /// 设置基础配置选项
    #[allow(dead_code)]
    pub fn set_basic_options(
        &self,
        apikey: Option<String>,
        proxy_url: Option<String>,
        log: Option<bool>,
        timeout_ms: Option<u32>,
        host: Option<String>,
    ) -> AppResult<()> {
        let mut config = self.load_config()?;

        if let Some(apikey) = apikey {
            config.APIKEY = Some(apikey);
        }
        if let Some(proxy_url) = proxy_url {
            config.PROXY_URL = Some(proxy_url);
        }
        if let Some(log) = log {
            config.LOG = Some(log);
        }
        if let Some(timeout_ms) = timeout_ms {
            config.API_TIMEOUT_MS = Some(timeout_ms);
        }
        if let Some(host) = host {
            config.HOST = Some(host);
        }

        self.save_config(&config)?;
        Ok(())
    }

    /// 获取配置统计信息
    pub fn get_config_stats(&self) -> AppResult<ConfigStats> {
        let config = self.load_config()?;

        Ok(ConfigStats {
            provider_count: config.Providers.len(),
            current_default_route: config.Router.default.clone(),
            has_background_route: config.Router.background.is_some(),
            has_think_route: config.Router.think.is_some(),
            has_long_context_route: config.Router.long_context.is_some(),
            has_web_search_route: config.Router.web_search.is_some(),
            api_timeout_ms: config.API_TIMEOUT_MS,
            log_enabled: config.LOG.unwrap_or(false),
        })
    }

    /// 验证所有 Router 路由的 Provider 引用
    pub fn validate_router_references(&self) -> AppResult<Vec<String>> {
        let config = self.load_config()?;
        let mut errors = Vec::new();

        let provider_names: std::collections::HashSet<_> =
            config.Providers.iter().map(|p| p.name.as_str()).collect();

        for (route_name, route_value) in config.Router.get_all_routes() {
            if let Some(provider_name) = route_value.split(',').next() {
                if !provider_names.contains(provider_name) {
                    errors.push(format!(
                        "路由 '{route_name}' 引用了不存在的提供商 '{provider_name}'"
                    ));
                }
            }
        }

        Ok(errors)
    }

    // ==================== 智能配置管理方法 ====================

    /// 确保存在可用的Router Profile配置
    /// 优先级：本地配置 → CCR配置自动生成 → 提示创建Provider
    pub fn ensure_router_profile_exists(&self) -> AppResult<RouterProfileStatus> {
        // 1. 检查本地是否已有Router Profile配置
        let local_config = Config::load().unwrap_or_default();
        if !local_config.groups.router.is_empty() {
            return Ok(RouterProfileStatus::LocalExists);
        }

        // 2. 检查claude-code-router配置是否存在且有Provider
        if !self.config_exists() {
            return Ok(RouterProfileStatus::NeedCreateProvider);
        }

        let ccr_config = self.load_config()?;
        if ccr_config.Providers.is_empty() {
            return Ok(RouterProfileStatus::NeedCreateProvider);
        }

        // 3. 从CCR配置自动生成default router profile
        match self.generate_default_router_profile()? {
            Some(router_profile) => {
                // 保存到本地配置
                let mut local_config = local_config;
                local_config.add_router_profile("default".to_string(), router_profile)?;
                local_config.save()?;
                Ok(RouterProfileStatus::GeneratedDefault)
            }
            None => Ok(RouterProfileStatus::NeedCreateProvider),
        }
    }

    /// 从claude-code-router配置生成默认的Router Profile
    pub fn generate_default_router_profile(&self) -> AppResult<Option<RouterProfile>> {
        let ccr_config = self.load_config()?;

        if ccr_config.Providers.is_empty() {
            return Ok(None);
        }

        // 验证Router配置中的provider引用是否有效
        let provider_names: std::collections::HashSet<_> = ccr_config
            .Providers
            .iter()
            .map(|p| p.name.as_str())
            .collect();

        for (route_name, route_value) in ccr_config.Router.get_all_routes() {
            if let Some(provider_name) = route_value.split(',').next() {
                if !provider_names.contains(provider_name) {
                    println!(
                        "⚠️  警告: 路由 '{route_name}' 引用了不存在的提供商 '{provider_name}'"
                    );
                }
            }
        }

        let router_profile = RouterProfile::new(
            "default".to_string(),
            ccr_config.Router.clone(),
            Some("从 claude-code-router 配置自动生成".to_string()),
        )?;

        Ok(Some(router_profile))
    }

    /// 获取本地Router Profile列表（结合智能生成）
    pub fn get_router_profiles(&self) -> AppResult<Vec<(String, RouterProfile, bool)>> {
        // 确保有可用的Router Profile
        if self.ensure_router_profile_exists()? == RouterProfileStatus::NeedCreateProvider {
            return Ok(Vec::new()); // 返回空列表，调用者处理提示
        }

        let config = Config::load()?;
        let profiles = config.list_router_profiles();
        // 转换引用为拥有的值
        let owned_profiles = profiles
            .into_iter()
            .map(|(name, profile, is_default)| (name, profile.clone(), is_default))
            .collect();
        Ok(owned_profiles)
    }

    /// 获取指定的Router Profile（支持智能生成）
    pub fn get_router_profile(&self, name: &str) -> AppResult<RouterProfile> {
        // 如果请求default且本地不存在，尝试智能生成
        let config = Config::load().unwrap_or_default();

        if let Ok(profile) = config.get_router_profile(name) {
            return Ok(profile.clone());
        }

        // 如果是请求default且不存在，尝试自动生成
        if name == "default" {
            match self.ensure_router_profile_exists()? {
                RouterProfileStatus::GeneratedDefault => {
                    let updated_config = Config::load()?;
                    return Ok(updated_config.get_router_profile(name)?.clone());
                }
                RouterProfileStatus::NeedCreateProvider => {
                    return Err(AppError::Config(
                        "暂无 Provider 配置，请先使用 'ccode provider add <name>' 添加 Provider"
                            .to_string(),
                    ));
                }
                _ => {}
            }
        }

        Err(AppError::ProfileNotFound(name.to_string()))
    }

    /// 添加Router Profile到本地配置
    pub fn add_router_profile(&self, name: String, router_profile: RouterProfile) -> AppResult<()> {
        let mut config = Config::load().unwrap_or_default();
        config.add_router_profile(name, router_profile)?;
        config.save()?;
        Ok(())
    }

    /// 删除Router Profile从本地配置
    pub fn remove_router_profile(&self, name: &str) -> AppResult<()> {
        let mut config = Config::load()?;
        config.remove_router_profile(name)?;
        config.save()?;
        Ok(())
    }

    /// 设置默认Router Profile并应用到CCR配置
    pub fn use_router_profile(&self, name: &str) -> AppResult<()> {
        let mut config = Config::load()?;
        let router_profile = config.get_router_profile(name)?.clone();

        // 设置为默认
        config.set_default_router(name)?;
        config.save()?;

        // 应用到claude-code-router配置
        self.apply_router_profile(&router_profile)?;

        Ok(())
    }
}

/// Router Profile状态枚举
#[derive(Debug, PartialEq)]
pub enum RouterProfileStatus {
    /// 本地已存在Router Profile配置
    LocalExists,
    /// 自动生成了default配置
    GeneratedDefault,
    /// 需要先创建Provider
    NeedCreateProvider,
}

/// 配置统计信息
#[derive(Debug)]
pub struct ConfigStats {
    pub provider_count: usize,
    pub current_default_route: String,
    pub has_background_route: bool,
    pub has_think_route: bool,
    pub has_long_context_route: bool,
    pub has_web_search_route: bool,
    pub api_timeout_ms: Option<u32>,
    pub log_enabled: bool,
}

impl ConfigStats {
    /// 格式化统计信息显示
    pub fn format_display(&self) -> String {
        let mut stats = String::new();

        stats.push_str(&format!("🔗 Provider 数量: {}\n", self.provider_count));
        stats.push_str(&format!("🎯 默认路由: {}\n", self.current_default_route));

        if self.has_background_route {
            stats.push_str("🔄 后台路由: ✅\n");
        }
        if self.has_think_route {
            stats.push_str("💭 思考路由: ✅\n");
        }
        if self.has_long_context_route {
            stats.push_str("📜 长上下文路由: ✅\n");
        }
        if self.has_web_search_route {
            stats.push_str("🔍 网络搜索路由: ✅\n");
        }

        if let Some(timeout) = self.api_timeout_ms {
            stats.push_str(&format!("⏱️  API 超时: {timeout}ms\n"));
        }

        stats.push_str(&format!(
            "📝 日志记录: {}\n",
            if self.log_enabled { "启用" } else { "禁用" }
        ));

        stats
    }
}
