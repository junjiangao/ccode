use crate::config::{CcrConfig, CcrProvider, CcrRouter, Config, RouterProfile};
use crate::error::{AppError, AppResult};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

/// Provider操作类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderOperation {
    /// 添加Provider
    Add,
    /// 更新Provider
    Update,
    /// 删除Provider
    Remove,
}

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
    #[allow(dead_code)]
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

    /// 列出所有 Providers
    pub fn list_providers(&self) -> AppResult<Vec<CcrProvider>> {
        let config = self.load_config()?;
        Ok(config.Providers)
    }

    /// 添加 Provider
    pub fn add_provider(&self, provider: CcrProvider) -> AppResult<()> {
        // 使用精确更新方法
        self.update_provider_only(&provider, ProviderOperation::Add)?;
        Ok(())
    }

    /// 删除 Provider
    pub fn remove_provider(&self, name: &str) -> AppResult<()> {
        // 创建一个临时的Provider对象（只需要name字段）
        let temp_provider = CcrProvider {
            name: name.to_string(),
            api_base_url: String::new(), // 临时值，删除操作不需要验证
            api_key: String::new(),      // 临时值，删除操作不需要验证
            models: Vec::new(),          // 临时值，删除操作不需要验证
            transformer: None,
            provider_type: None,
        };

        // 使用精确更新方法
        self.update_provider_only(&temp_provider, ProviderOperation::Remove)?;
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
        // 使用精确更新方法
        self.update_provider_only(&provider, ProviderOperation::Update)?;
        Ok(())
    }

    /// 检查 Provider 是否存在
    pub fn provider_exists(&self, name: &str) -> AppResult<bool> {
        let config = self.load_config()?;
        Ok(config.get_provider(name).is_some())
    }

    /// 获取当前 Router 配置
    pub fn get_current_router(&self) -> AppResult<CcrRouter> {
        let config = self.load_config()?;
        Ok(config.Router)
    }

    /// 应用 Router Profile 配置（只修改 Router 部分）
    pub fn apply_router_profile(&self, router_profile: &RouterProfile) -> AppResult<()> {
        // 使用精确更新方法，只修改Router节点
        self.update_router_only(&router_profile.router)?;

        println!("✅ 已应用 Router Profile '{}'", router_profile.name);
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
            if let Some(provider_name) = route_value.split(',').next()
                && !provider_names.contains(provider_name)
            {
                errors.push(format!(
                    "路由 '{route_name}' 引用了不存在的提供商 '{provider_name}'"
                ));
            }
        }

        Ok(errors)
    }

    /// 确保存在可用的Router Profile配置
    /// 优先级：本地配置 → CCR配置自动生成 → 提示创建Provider
    pub fn ensure_router_profile_exists(&self) -> AppResult<RouterProfileStatus> {
        let local_config = Config::load().unwrap_or_default();
        if !local_config.groups.router.is_empty() {
            return Ok(RouterProfileStatus::LocalExists);
        }

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
            if let Some(provider_name) = route_value.split(',').next()
                && !provider_names.contains(provider_name)
            {
                println!("⚠️  警告: 路由 '{route_name}' 引用了不存在的提供商 '{provider_name}'");
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

    /// 从CCR配置文件同步Providers信息到本地缓存
    /// 这用于确保本地缓存与CCR配置文件保持一致
    pub fn sync_providers_from_ccr(&self) -> AppResult<()> {
        if !self.config_exists() {
            // CCR配置文件不存在，无需同步
            return Ok(());
        }

        let ccr_config = self.load_config()?;

        // 同步逻辑：这里主要用于信息展示和验证
        // Provider的管理仍然通过ccode命令进行，这里只是读取最新状态
        println!(
            "🔄 同步Provider信息: 发现 {} 个Provider",
            ccr_config.Providers.len()
        );

        Ok(())
    }

    /// 统一的配置同步入口点
    /// 在CCR相关命令启动时调用，确保配置信息同步
    pub fn sync_config_from_ccr(&self) -> AppResult<()> {
        self.sync_providers_from_ccr()?;
        // 未来可以在这里添加其他同步逻辑
        Ok(())
    }

    /// 仅更新CCR配置文件的Router节点
    /// 这是精确更新的核心方法，只修改Router部分而保持其他配置不变
    pub fn update_router_only(&self, router: &CcrRouter) -> AppResult<()> {
        router.validate()?;

        let mut config = self.load_config()?;

        // 验证Router配置中的Provider引用是否有效
        let provider_names: std::collections::HashSet<_> =
            config.Providers.iter().map(|p| p.name.as_str()).collect();

        for (route_name, route_value) in router.get_all_routes() {
            if let Some(provider_name) = route_value.split(',').next()
                && !provider_names.contains(provider_name)
            {
                return Err(AppError::InvalidConfig(format!(
                    "路由 '{}' 引用了不存在的提供商 '{}'",
                    route_name, provider_name
                )));
            }
        }

        // 如果配置文件已存在，先创建备份
        if self.config_path.exists() {
            self.create_backup()?;
        }

        // 仅更新Router节点
        config.Router = router.clone();

        // 保存配置
        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&self.config_path, content)?;

        println!("✅ 已更新 CCR Router 配置");
        Ok(())
    }

    /// 仅更新CCR配置文件中的单个Provider
    /// 用于Provider的增删改操作，避免重写整个配置文件
    pub fn update_provider_only(
        &self,
        provider: &CcrProvider,
        operation: ProviderOperation,
    ) -> AppResult<()> {
        let mut config = self.load_config()?;

        match operation {
            ProviderOperation::Add => {
                provider.validate()?;
                if config.Providers.iter().any(|p| p.name == provider.name) {
                    return Err(AppError::Config(format!(
                        "Provider '{}' 已存在",
                        provider.name
                    )));
                }
                config.Providers.push(provider.clone());
            }
            ProviderOperation::Update => {
                provider.validate()?;
                if let Some(existing) = config
                    .Providers
                    .iter_mut()
                    .find(|p| p.name == provider.name)
                {
                    *existing = provider.clone();
                } else {
                    return Err(AppError::Config(format!(
                        "Provider '{}' 不存在",
                        provider.name
                    )));
                }
            }
            ProviderOperation::Remove => {
                // 删除操作不需要验证Provider内容，只需要name
                let original_len = config.Providers.len();
                config.Providers.retain(|p| p.name != provider.name);

                if config.Providers.len() == original_len {
                    return Err(AppError::Config(format!(
                        "Provider '{}' 不存在",
                        provider.name
                    )));
                }
            }
        }

        // 如果配置文件已存在，先创建备份
        if self.config_path.exists() {
            self.create_backup()?;
        }

        // 保存配置
        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&self.config_path, content)?;

        println!("✅ 已更新 CCR Provider 配置");
        Ok(())
    }

    /// 仅更新CCR配置文件的Providers节点
    /// 用于批量Provider更新操作
    #[allow(dead_code)]
    pub fn update_providers_only(&self, providers: Vec<CcrProvider>) -> AppResult<()> {
        // 验证所有Provider
        for provider in &providers {
            provider.validate()?;
        }

        let mut config = self.load_config()?;

        // 如果配置文件已存在，先创建备份
        if self.config_path.exists() {
            self.create_backup()?;
        }

        // 更新Providers节点
        config.Providers = providers;

        // 保存配置
        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(&self.config_path, content)?;

        println!("✅ 已更新 CCR Providers 配置");
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
