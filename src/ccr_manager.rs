use crate::config::CcrProfile;
use crate::error::{AppError, AppResult};
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

    /// 检查CCR命令是否可用
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

    /// 生成CCR配置文件
    pub fn generate_ccr_config(&self, profile: &CcrProfile) -> AppResult<()> {
        let config_path = self.get_ccr_config_path();

        // 创建CCR标准格式的配置
        let ccr_config = serde_json::to_value(profile)?;
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
