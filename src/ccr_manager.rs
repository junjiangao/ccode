use crate::config::{CcrProfile, CcrProvider, Config, ProviderType};
use crate::error::{AppError, AppResult};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use sysinfo::System;
use tokio::time::timeout;

/// CCR服务管理器
pub struct CcrManager {
    config_dir: PathBuf,
    service_pid: Option<u32>,
}

impl CcrManager {
    /// 创建新的CCR管理器实例
    pub fn new() -> AppResult<Self> {
        let config_dir = Self::get_ccr_config_dir()?;

        Ok(Self {
            config_dir,
            service_pid: None,
        })
    }

    /// 获取CCR配置目录路径
    fn get_ccr_config_dir() -> AppResult<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| AppError::Config("无法获取用户主目录".to_string()))?;

        let ccr_dir = home_dir.join(".claude-code-router");

        // 确保CCR配置目录存在
        if !ccr_dir.exists() {
            fs::create_dir_all(&ccr_dir)?;
        }

        Ok(ccr_dir)
    }

    /// 获取CCR配置文件路径
    fn get_ccr_config_path(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    /// 获取CCR配置备份目录路径
    fn get_backup_dir(&self) -> PathBuf {
        self.config_dir.join("backups")
    }

    /// 创建配置文件备份
    pub fn create_backup(&self) -> AppResult<String> {
        let config_path = self.get_ccr_config_path();

        if !config_path.exists() {
            return Err(AppError::Config(
                "CCR配置文件不存在，无法创建备份".to_string(),
            ));
        }

        // 创建备份目录
        let backup_dir = self.get_backup_dir();
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)?;
        }

        // 生成备份文件名（带时间戳）
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("config_backup_{timestamp}.json");
        let backup_path = backup_dir.join(&backup_filename);

        // 复制配置文件到备份目录
        fs::copy(&config_path, &backup_path)?;

        println!("✅ 配置备份已创建: {}", backup_path.display());
        Ok(backup_filename)
    }

    /// 列出所有备份文件
    #[allow(dead_code)]
    pub fn list_backups(&self) -> AppResult<Vec<String>> {
        let backup_dir = self.get_backup_dir();

        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("config_backup_") {
                        backups.push(filename.to_string());
                    }
                }
            }
        }

        // 按时间戳排序（最新的在前）
        backups.sort_by(|a, b| b.cmp(a));

        Ok(backups)
    }

    /// 从备份恢复配置
    #[allow(dead_code)]
    pub fn restore_from_backup(&self, backup_filename: &str) -> AppResult<()> {
        let backup_dir = self.get_backup_dir();
        let backup_path = backup_dir.join(backup_filename);

        if !backup_path.exists() {
            return Err(AppError::Config(format!(
                "备份文件不存在: {backup_filename}"
            )));
        }

        let config_path = self.get_ccr_config_path();

        // 在恢复前创建当前配置的备份
        if config_path.exists() {
            let _ = self.create_backup(); // 忽略备份失败的错误
        }

        // 恢复配置文件
        fs::copy(&backup_path, &config_path)?;

        println!("✅ 配置已从备份恢复: {backup_filename}");
        Ok(())
    }

    /// 删除指定的备份文件
    #[allow(dead_code)]
    pub fn delete_backup(&self, backup_filename: &str) -> AppResult<()> {
        let backup_dir = self.get_backup_dir();
        let backup_path = backup_dir.join(backup_filename);

        if !backup_path.exists() {
            return Err(AppError::Config(format!(
                "备份文件不存在: {backup_filename}"
            )));
        }

        fs::remove_file(&backup_path)?;

        println!("✅ 备份文件已删除: {backup_filename}");
        Ok(())
    }

    /// 清理旧的备份文件（保留最新的N个）
    #[allow(dead_code)]
    pub fn cleanup_old_backups(&self, keep_count: usize) -> AppResult<usize> {
        let backups = self.list_backups()?;

        if backups.len() <= keep_count {
            return Ok(0);
        }

        let to_delete = &backups[keep_count..];
        let mut deleted_count = 0;

        for backup_filename in to_delete {
            if let Err(e) = self.delete_backup(backup_filename) {
                eprintln!("⚠️  删除备份文件失败: {backup_filename}, 错误: {e}");
            } else {
                deleted_count += 1;
            }
        }

        if deleted_count > 0 {
            println!("🧹 已清理 {deleted_count} 个旧备份文件");
        }

        Ok(deleted_count)
    }

    /// 检查CCR配置是否为空
    pub async fn is_ccr_config_empty(&self) -> AppResult<bool> {
        let config_path = self.get_ccr_config_path();

        if !config_path.exists() {
            return Ok(true);
        }

        // 读取配置文件
        let content = fs::read_to_string(&config_path)?;

        // 尝试解析JSON
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => {
                // 检查是否有Providers字段且不为空
                if let Some(providers) = json.get("Providers") {
                    if let Some(providers_array) = providers.as_array() {
                        return Ok(providers_array.is_empty());
                    }
                }
                // 如果没有Providers字段，认为是空配置
                Ok(true)
            }
            Err(_) => {
                // 解析失败，认为是无效配置，当作空配置处理
                Ok(true)
            }
        }
    }

    /// 从现有的claude-code-router配置文件导入配置到ccode
    pub async fn import_from_ccr_config(&self) -> AppResult<Option<String>> {
        let config_path = self.get_ccr_config_path();

        if !config_path.exists() {
            return Ok(None);
        }

        // 读取配置文件
        let content = fs::read_to_string(&config_path)?;

        // 解析CCR配置
        let ccr_config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Config(format!("解析CCR配置文件失败: {e}")))?;

        // 提取Providers信息
        let providers = ccr_config
            .get("Providers")
            .and_then(|p| p.as_array())
            .ok_or_else(|| AppError::Config("CCR配置文件中没有找到Providers字段".to_string()))?;

        if providers.is_empty() {
            return Ok(None);
        }

        // 读取现有的ccode配置
        let mut ccode_config = Config::load().unwrap_or_default();

        let mut imported_count = 0;

        // 为每个provider创建一个ccode CCR配置
        for (index, provider_json) in providers.iter().enumerate() {
            // 解析provider信息
            let default_name = format!("imported_provider_{}", index + 1);
            let name = provider_json
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(&default_name);

            let api_base_url = provider_json
                .get("api_base_url")
                .and_then(|u| u.as_str())
                .unwrap_or("");

            let api_key = provider_json
                .get("api_key")
                .and_then(|k| k.as_str())
                .unwrap_or("");

            let models: Vec<String> = provider_json
                .get("models")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();

            if api_base_url.is_empty() || models.is_empty() {
                continue; // 跳过无效的provider
            }

            // 检测provider类型
            let provider_type = self.detect_provider_type(api_base_url, name);

            // 创建CcrProvider
            let provider = CcrProvider::new(
                name.to_string(),
                api_base_url.to_string(),
                api_key.to_string(),
                models.clone(),
                provider_type,
            );

            // 创建默认路由
            let default_route = if !models.is_empty() {
                format!("{name},{}", models[0])
            } else {
                format!("{name},default-model")
            };

            // 创建CCR配置
            match CcrProfile::new(
                provider,
                default_route,
                Some(format!("从CCR配置导入: {name}")),
            ) {
                Ok(ccr_profile) => {
                    let profile_name = format!("imported_{name}");

                    // 检查是否已存在同名配置
                    if !ccode_config.groups.ccr.contains_key(&profile_name) {
                        if let Err(e) =
                            ccode_config.add_ccr_profile(profile_name.clone(), ccr_profile)
                        {
                            eprintln!("⚠️  导入provider '{name}'失败: {e}");
                        } else {
                            imported_count += 1;
                            println!("✅ 已导入provider '{name}' 为CCR配置 '{profile_name}'");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  创建CCR配置失败 '{name}': {e}");
                }
            }
        }

        if imported_count > 0 {
            // 保存配置
            ccode_config.save()?;
            Ok(Some(format!("成功导入 {imported_count} 个CCR配置")))
        } else {
            Ok(None)
        }
    }

    /// 检测provider类型
    fn detect_provider_type(&self, api_base_url: &str, name: &str) -> ProviderType {
        let url_lower = api_base_url.to_lowercase();
        let name_lower = name.to_lowercase();

        if url_lower.contains("openrouter.ai") || name_lower.contains("openrouter") {
            ProviderType::OpenRouter
        } else if url_lower.contains("deepseek") || name_lower.contains("deepseek") {
            ProviderType::DeepSeek
        } else if url_lower.contains("generativelanguage.googleapis.com")
            || url_lower.contains("/v1beta/models/")
            || name_lower.contains("gemini")
        {
            ProviderType::Gemini
        } else if name_lower.contains("qwen")
            || url_lower.contains("dashscope")
            || url_lower.contains("modelscope")
        {
            ProviderType::Qwen
        } else {
            ProviderType::OpenAI // 默认为OpenAI兼容
        }
    }
    pub async fn check_ccr_availability(&self) -> AppResult<bool> {
        // 检查是否安装了 @musistudio/claude-code-router
        let output = Command::new("pnpm")
            .args(["list", "-g", "@musistudio/claude-code-router"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(stdout.contains("@musistudio/claude-code-router"))
                } else {
                    Ok(false)
                }
            }
            Err(_) => {
                // 尝试检查ccr命令是否直接可用
                let ccr_check = Command::new("ccr")
                    .arg("--version")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output();

                Ok(ccr_check.is_ok() && ccr_check.unwrap().status.success())
            }
        }
    }

    /// 安装CCR依赖
    pub async fn install_ccr(&self) -> AppResult<()> {
        println!("📦 正在安装CCR依赖...");

        let install_result = timeout(
            Duration::from_secs(120),
            self.run_npm_command(&["install", "-g", "@musistudio/claude-code-router"]),
        )
        .await;

        match install_result {
            Ok(Ok(())) => {
                println!("✅ CCR依赖安装成功");
                Ok(())
            }
            Ok(Err(e)) => {
                println!("❌ CCR依赖安装失败");
                Err(e)
            }
            Err(_) => {
                println!("❌ CCR依赖安装超时");
                Err(AppError::Config("CCR安装超时".to_string()))
            }
        }
    }

    /// 运行npm命令
    async fn run_npm_command(&self, args: &[&str]) -> AppResult<()> {
        let mut cmd = Command::new("npm");
        cmd.args(args);

        let output = cmd.output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Config(format!("npm命令执行失败: {stderr}")))
        }
    }

    /// 生成CCR配置文件（带备份）
    pub fn generate_ccr_config(&self, profile: &CcrProfile) -> AppResult<()> {
        let config_path = self.get_ccr_config_path();

        // 如果配置文件已存在，先创建备份
        if config_path.exists() {
            if let Err(e) = self.create_backup() {
                eprintln!("⚠️  创建备份失败: {e}");
            }
        }

        // 创建CCR标准格式的配置
        let ccr_config = profile.to_ccr_config();
        let formatted_config = serde_json::to_string_pretty(&ccr_config)?;

        fs::write(&config_path, formatted_config)?;

        println!("✅ CCR配置文件已生成: {}", config_path.display());
        Ok(())
    }

    /// 启动CCR服务
    pub async fn start_service(&mut self) -> AppResult<()> {
        // 检查服务是否已经在运行
        if self.is_service_running().await? {
            println!("ℹ️  CCR服务已经在运行");
            return Ok(());
        }

        // 检查CCR是否可用
        if !self.check_ccr_availability().await? {
            println!("⚠️  CCR未安装，尝试自动安装...");
            self.install_ccr().await?;
        }

        println!("🚀 启动CCR服务...");

        // 启动CCR服务
        let mut cmd = Command::new("ccr");
        cmd.args(["start"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()?;
        self.service_pid = Some(child.id());

        // 等待服务启动
        tokio::time::sleep(Duration::from_secs(3)).await;

        if self.is_service_running().await? {
            println!("✅ CCR服务启动成功");
            Ok(())
        } else {
            println!("❌ CCR服务启动失败");
            Err(AppError::Config("CCR服务启动失败".to_string()))
        }
    }

    /// 停止CCR服务
    pub async fn stop_service(&mut self) -> AppResult<()> {
        if !self.is_service_running().await? {
            println!("ℹ️  CCR服务未在运行");
            return Ok(());
        }

        println!("🛑 停止CCR服务...");

        // 尝试优雅关闭
        let output = Command::new("ccr").args(["stop"]).output()?;

        if output.status.success() {
            self.service_pid = None;
            println!("✅ CCR服务已停止");
            Ok(())
        } else {
            // 如果优雅关闭失败，尝试强制终止
            self.force_kill_service().await
        }
    }

    /// 强制终止CCR服务
    async fn force_kill_service(&mut self) -> AppResult<()> {
        let pids = self.find_ccr_processes().await?;

        if pids.is_empty() {
            println!("ℹ️  没有找到运行中的CCR进程");
            return Ok(());
        }

        println!("🔪 强制终止CCR进程...");

        for pid in pids {
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let _ = Command::new("kill").args(["-9", &pid.to_string()]).exec();
            }

            #[cfg(windows)]
            {
                Command::new("taskkill")
                    .args(&["/F", "/PID", &pid.to_string()])
                    .output()?;
            }
        }

        self.service_pid = None;
        println!("✅ CCR进程已终止");
        Ok(())
    }

    /// 重启CCR服务
    pub async fn restart_service(&mut self) -> AppResult<()> {
        println!("🔄 重启CCR服务...");

        self.stop_service().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.start_service().await?;

        Ok(())
    }

    /// 检查CCR服务是否正在运行
    pub async fn is_service_running(&self) -> AppResult<bool> {
        // 检查CCR默认端口3456是否被占用
        self.check_port_in_use(3456).await
    }

    /// 检查端口是否被占用
    async fn check_port_in_use(&self, port: u16) -> AppResult<bool> {
        use reqwest;

        let url = format!("http://localhost:{port}/health");

        match timeout(Duration::from_secs(5), reqwest::get(&url)).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            _ => Ok(false),
        }
    }

    /// 查找CCR相关进程
    async fn find_ccr_processes(&self) -> AppResult<Vec<u32>> {
        let mut system = System::new_all();
        system.refresh_all();

        let mut pids = Vec::new();

        for (pid, process) in system.processes() {
            let process_name = process.name();
            let cmd_line = process.cmd().join(" ");

            // 查找包含ccr或claude-code-router的进程
            if process_name.contains("ccr")
                || process_name.contains("claude")
                || cmd_line.contains("claude-code-router")
                || cmd_line.contains("ccr")
            {
                pids.push(pid.as_u32());
            }
        }

        Ok(pids)
    }

    /// 获取CCR服务状态
    pub async fn get_service_status(&self) -> AppResult<CcrServiceStatus> {
        let is_running = self.is_service_running().await?;
        let is_available = self.check_ccr_availability().await?;
        let pids = self.find_ccr_processes().await?;

        Ok(CcrServiceStatus {
            is_running,
            is_available,
            process_ids: pids,
            config_exists: self.get_ccr_config_path().exists(),
        })
    }

    /// 获取CCR服务日志
    pub async fn get_service_logs(&self) -> AppResult<String> {
        let log_path = self.config_dir.join("logs").join("ccr.log");

        if log_path.exists() {
            let logs = fs::read_to_string(log_path)?;
            Ok(logs)
        } else {
            Ok("暂无日志文件".to_string())
        }
    }
}

/// CCR服务状态
#[derive(Debug)]
pub struct CcrServiceStatus {
    pub is_running: bool,
    pub is_available: bool,
    pub process_ids: Vec<u32>,
    pub config_exists: bool,
}

impl CcrServiceStatus {
    /// 格式化状态信息
    pub fn format_status(&self) -> String {
        let mut status = String::new();

        status.push_str(&format!(
            "🔧 CCR可用性: {}\n",
            if self.is_available {
                "✅ 已安装"
            } else {
                "❌ 未安装"
            }
        ));

        status.push_str(&format!(
            "🚀 服务状态: {}\n",
            if self.is_running {
                "✅ 运行中"
            } else {
                "❌ 未运行"
            }
        ));

        status.push_str(&format!(
            "📄 配置文件: {}\n",
            if self.config_exists {
                "✅ 存在"
            } else {
                "❌ 不存在"
            }
        ));

        if !self.process_ids.is_empty() {
            status.push_str(&format!("🔍 进程ID: {:?}\n", self.process_ids));
        }

        status
    }
}
