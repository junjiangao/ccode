# ccode 🚀

**Claude Code 配置管理工具** - 专注 Direct 模式的 Claude 环境快速切换工具

[![CI Status](https://github.com/junjiangao/ccode/workflows/CI/badge.svg)](https://github.com/junjiangao/ccode/actions)
[![Release](https://github.com/junjiangao/ccode/workflows/Release/badge.svg)](https://github.com/junjiangao/ccode/actions)
[![Version](https://img.shields.io/github/v/release/junjiangao/ccode?include_prereleases)](https://github.com/junjiangao/ccode/releases)
[![License](https://img.shields.io/github/license/junjiangao/ccode)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgray)](https://github.com/junjiangao/ccode/releases)

## ✨ 核心特性

### 🎯 模式
- **Direct 模式**：简单的 API 配置，直接启动 `claude`

### 🌟 主要功能
- 📋 **配置管理**：支持多配置存储和快速切换
- 🚀 **参数透传**：支持将参数透传给claude命令（仅Direct模式）
- 📱 **交互式操作**：友好的命令行交互界面
- 🌐 **跨平台支持**：Windows、macOS、Linux

### 🛠️ 工作模式
- **Direct模式**：传统的token+base_url配置方式，直接启动claude程序
 

## 🚀 快速开始

### 📋 系统要求

- **Rust**: 1.70+（如需从源码编译）
- **Claude CLI**: 已安装 `claude` 命令行工具

### 📦 安装

#### 🚀 预编译二进制文件（推荐）

从[GitHub Releases](https://github.com/junjiangao/ccode/releases)下载对应平台的预编译二进制文件：

```bash
# Linux (Ubuntu 22.04 LTS)
wget https://github.com/junjiangao/ccode/releases/latest/download/ccode-linux-x86_64
chmod +x ccode-linux-x86_64
sudo mv ccode-linux-x86_64 /usr/local/bin/ccode

# macOS (Intel)
wget https://github.com/junjiangao/ccode/releases/latest/download/ccode-macos-x86_64
chmod +x ccode-macos-x86_64
sudo mv ccode-macos-x86_64 /usr/local/bin/ccode

# macOS (Apple Silicon)
wget https://github.com/junjiangao/ccode/releases/latest/download/ccode-macos-aarch64
chmod +x ccode-macos-aarch64
sudo mv ccode-macos-aarch64 /usr/local/bin/ccode

# Windows: 下载 ccode-windows-x86_64.exe 并放到 PATH 中
```

#### 🔨 从源码编译

```bash
git clone https://github.com/junjiangao/ccode.git
cd ccode
cargo build --release
sudo cp target/release/ccode /usr/local/bin/
```

## 📖 使用指南

### 🎯 Direct 模式（简单配置）

适合简单的API切换需求，与传统版本完全兼容。

#### 添加Direct配置
```bash
ccode add myapi --group direct
# 或使用默认的direct组
ccode add myapi
```

按提示输入：
- **ANTHROPIC_AUTH_TOKEN**: `your-api-token` (必需)
- **ANTHROPIC_BASE_URL**: `https://api.example.com` (必需)
- **ANTHROPIC_MODEL**: `claude-3-5-sonnet-20241022` (可选)
- **ANTHROPIC_SMALL_FAST_MODEL**: `claude-3-haiku-20240307` (可选)
- **描述**: `我的API服务` (可选)

#### 使用Direct配置
```bash
# 列出Direct配置
ccode list --group direct

# 设置默认配置
ccode use myapi --group direct

# 启动claude
ccode run myapi --group direct

# 启动claude并透传参数（仅Direct模式支持）
ccode run myapi --group direct --version
ccode run myapi code --project myapp
# 注意：对于可能冲突的参数（如--help），需要使用--分隔符：
ccode run myapi -- --help
```

<!-- Router/Provider 功能已移除 -->

## 📋 命令参考

### 🔄 命令

```bash
# 列出配置
ccode list [--group direct]

# 添加配置
ccode add <name> [--group direct]

# 设置默认配置
ccode use <name> [--group direct]

# 启动 claude（支持参数透传）
ccode run [name] [--group direct] [<claude_args>...]

# 示例：
# ccode run myapi --version                    # 直接透传
# ccode run myapi code                         # 透传子命令
# ccode run myapi -- --help                    # 使用 -- 分隔符避免冲突

# 删除配置
ccode remove <name> [--group direct]
```

<!-- Router/Provider 快捷命令已移除 -->

## 📁 配置文件

### 配置存储位置
- **Linux/macOS**: `~/.config/ccode/config.json`
- **Windows**: `%APPDATA%/ccode/config.json`

### ccode 配置文件结构（Direct）

```json
{
  "version": "2.0",
  "default_group": "direct",
  "default_profile": {
    "direct": "myapi"
  },
  "groups": {
    "direct": {
      "myapi": {
        "ANTHROPIC_AUTH_TOKEN": "your-token",
        "ANTHROPIC_BASE_URL": "https://api.example.com",
        "ANTHROPIC_MODEL": "claude-3-5-sonnet-20241022",
        "ANTHROPIC_SMALL_FAST_MODEL": "claude-3-haiku-20240307",
        "description": "我的API服务",
        "created_at": "2025-07-31T10:00:00Z"
      }
    }
  }
}
```

<!-- CCR 配置文件相关章节已移除 -->

## 🔧 工作原理

### Direct模式
1. 读取Direct配置中的认证信息和可选设置
2. 设置必需环境变量：`ANTHROPIC_AUTH_TOKEN`、`ANTHROPIC_BASE_URL`
3. 条件设置可选环境变量：`ANTHROPIC_MODEL`、`ANTHROPIC_SMALL_FAST_MODEL`（仅在配置时设置）
4. 可选择透传额外参数给claude命令
5. 启动claude程序

<!-- Router 模式工作原理已移除 -->

<!-- 架构图（CCR 集成）已移除 -->

## 🎯 使用场景

### 个人开发者
- **Direct 模式**：简单 API 切换，快速上手

### 团队协作
- 标准化 Direct 配置管理（开发/测试/生产）

### 高级用户
- 通过可选变量实现更细粒度的模型控制

## ⚠️ 重要说明

### 系统依赖
- **claude CLI**：必须预先安装 `claude` 命令行工具
- **配置文件**：ccode 仅管理配置，不包含服务管理功能

### 兼容性
- **向后兼容**：现有 Direct 模式配置无需修改
- **配置迁移**：自动从 v1.0 配置格式升级到 v2.0

### 限制说明
- ccode 不包含服务管理功能（start/stop/restart等）

## 📊 构建状态

| 平台 | 状态 | 说明 |
|------|------|------|
| **持续集成** | [![CI Status](https://github.com/junjiangao/ccode/workflows/CI/badge.svg)](https://github.com/junjiangao/ccode/actions) | 代码质量、测试、安全扫描 |
| **自动发布** | [![Release](https://github.com/junjiangao/ccode/workflows/Release/badge.svg)](https://github.com/junjiangao/ccode/actions) | 跨平台二进制构建发布 |
| **Linux (Ubuntu 22.04)** | ✅ 官方支持 | CI/CD标准环境 |
| **其他Linux发行版** | ⚠️ 社区支持 | 需要从源码编译 |
| **Windows/macOS** | ✅ 支持 | 跨平台兼容测试 |

## 🔧 技术栈

### 核心技术
- **语言**: Rust 2024 Edition
- **最低版本**: Rust 1.70+
- **CLI框架**: clap 4.x (derive API)

### 依赖管理
- **序列化**: serde + serde_json
- **目录处理**: dirs (跨平台)
- **时间处理**: chrono
- **错误处理**: anyhow
- **系统信息**: sysinfo

### 质量保证
- **测试覆盖**: 单元测试 + 集成测试
- **代码质量**: Zero warnings (clippy + rustfmt)
- **安全扫描**: cargo-audit 自动检查
- **CI/CD**: GitHub Actions 全平台构建

## 🛠️ 开发

### 项目结构

```
src/
├── main.rs           # CLI入口和命令路由
├── commands.rs       # 命令实现逻辑
├── config.rs         # 配置数据结构和管理
├── error.rs          # 统一错误处理
└── lib.rs            # 库入口模块导出
```

### 开发命令

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 代码格式化（提交前必须）
cargo fmt

# 代码质量检查（零警告要求）
cargo clippy -- -D warnings

# 安全漏洞扫描
cargo audit

# 完整CI检查流程
cargo fmt --check && \
cargo clippy -- -D warnings && \
cargo test && \
cargo build --release
```

### 🔄 CI/CD流程

- **🔍 持续集成**: 每次push和PR触发
  - 代码格式检查(rustfmt)
  - 代码质量检查(clippy)
  - 单元测试执行
  - 跨平台构建验证
  - 安全漏洞扫描(cargo-audit)

- **🚀 自动发布**: git tag推送触发
  - 多平台二进制构建
  - GitHub Releases自动创建  
  - 源码归档和资产上传

## 📄 许可证

本项目采用 [LICENSE](LICENSE) 许可证。

## 🤝 贡献

欢迎提交Issue和Pull Request！

### 贡献指南
1. Fork项目仓库
2. 创建功能分支
3. 提交更改（记得`cargo fmt`）
4. 推送到分支
5. 创建Pull Request

### 开发规范
- 遵循Rust官方代码风格
- 保持零clippy警告
- 添加适当的测试覆盖
- 更新相关文档

--- 

**最后更新**: 2025-08-10 | **架构版本**: v0.2.0（配置管理工具）
