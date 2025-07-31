# ccode 🚀

**Claude Code 环境管理工具** - 支持双模式配置的Claude环境切换和智能路由代理工具

[![CI Status](https://github.com/junjiangao/ccode/workflows/CI/badge.svg)](https://github.com/junjiangao/ccode/actions)
[![Release](https://github.com/junjiangao/ccode/workflows/Release/badge.svg)](https://github.com/junjiangao/ccode/actions)
[![Version](https://img.shields.io/github/v/release/junjiangao/ccode?include_prereleases)](https://github.com/junjiangao/ccode/releases)
[![License](https://img.shields.io/github/license/junjiangao/ccode)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgray)](https://github.com/junjiangao/ccode/releases)

## ✨ 核心特性

### 🔄 双模式架构
- **🎯 Direct模式**：传统的简单API配置（向后兼容）
- **🚀 CCR模式**：集成Claude Code Router的智能路由系统

### 🌟 CCR模式优势
- 🔀 **智能路由**：根据任务类型自动选择最适合的模型
  - `default`: 日常任务的默认模型
  - `background`: 后台任务的高性价比模型  
  - `think`: 推理密集型任务的强推理模型
  - `longContext`: 长上下文任务的大窗口模型
  - `webSearch`: 网络搜索任务的专用模型
- 🏗️ **多Provider支持**：OpenRouter、DeepSeek、Gemini、Qwen、自定义等
- ⚖️ **负载均衡**：多个相同类型模型间的智能分配
- 🔧 **请求转换**：自动适配不同Provider的API格式
- 📊 **服务管理**：完整的CCR服务生命周期控制

### 🛠️ 管理功能
- 💾 **配置管理**：支持多配置存储和快速切换
- 🎯 **智能推荐**：基于Provider类型的模型推荐
- 📱 **交互式操作**：友好的命令行交互界面
- 🌐 **跨平台支持**：Windows、macOS、Linux

## 🚀 快速开始

### 📋 系统要求

- **Rust**: 1.70+（如需从源码编译）
- **Claude CLI**: 已安装claude命令行工具
- **Node.js/npm**: CCR模式需要npm环境（自动管理依赖）

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

### 🎯 Direct模式（简单配置）

适合简单的API切换需求，与传统版本完全兼容。

#### 添加Direct配置
```bash
ccode add myapi --group direct
# 或使用默认的direct组
ccode add myapi
```

按提示输入：
- ANTHROPIC_AUTH_TOKEN: `your-api-token`
- ANTHROPIC_BASE_URL: `https://api.example.com`
- 描述（可选）: `我的API服务`

#### 使用Direct配置
```bash
# 列出Direct配置
ccode list --group direct

# 设置默认配置
ccode use myapi --group direct

# 启动claude
ccode run myapi --group direct
```

### 🚀 CCR模式（智能路由）

适合复杂的多模型路由需求，支持智能选择和负载均衡。

#### 添加CCR配置
```bash
ccode add-ccr production
# 或
ccode add production --group ccr
```

交互式配置流程：
1. **Provider管理**：选择或添加多个Provider（OpenRouter、DeepSeek等）
2. **模型配置**：为每个Provider配置可用模型
3. **路由设置**：配置不同场景的路由规则
4. **服务管理**：自动启动CCR服务

#### CCR配置示例

```json
{
  "providers": [
    {
      "name": "openrouter",
      "api_base_url": "https://openrouter.ai/api/v1/chat/completions",
      "api_key": "sk-or-xxx",
      "models": ["anthropic/claude-3.5-sonnet", "google/gemini-2.5-pro-preview"],
      "provider_type": "openrouter"
    },
    {
      "name": "deepseek",
      "api_base_url": "https://api.deepseek.com/chat/completions", 
      "api_key": "sk-xxx",
      "models": ["deepseek-chat", "deepseek-reasoner"],
      "provider_type": "deepseek"
    }
  ],
  "router": {
    "default": "deepseek,deepseek-chat",
    "background": "deepseek,deepseek-chat", 
    "think": "deepseek,deepseek-reasoner",
    "longContext": "openrouter,google/gemini-2.5-pro-preview",
    "longContextThreshold": 60000
  }
}
```

#### 使用CCR配置
```bash
# 列出CCR配置
ccode list-ccr

# 设置默认CCR配置
ccode use-ccr production

# 启动claude（智能路由）
ccode run-ccr production
```

### ⚙️ CCR服务管理

```bash
# 启动CCR服务
ccode ccr start

# 查看服务状态
ccode ccr status

# 重启服务（配置更新后）
ccode ccr restart

# 停止服务
ccode ccr stop

# 查看服务日志
ccode ccr logs
```

### 📊 Provider管理

```bash
# 列出所有Providers
ccode provider list

# 添加新Provider
ccode provider add myProvider

# 查看Provider详情
ccode provider show myProvider

# 编辑Provider
ccode provider edit myProvider

# 删除Provider
ccode provider remove myProvider
```

## 📋 命令参考

### 🔄 统一接口命令

支持`--group direct|ccr`参数的通用命令：

```bash
# 列出配置
ccode list [--group direct|ccr]

# 添加配置
ccode add <name> [--group direct|ccr]

# 设置默认配置  
ccode use <name> [--group direct|ccr]

# 启动claude
ccode run [name] [--group direct|ccr]

# 删除配置
ccode remove <name> [--group direct|ccr]
```

### 🚀 CCR快捷命令

专门针对CCR模式的便捷命令：

```bash
ccode add-ccr <name>      # 添加CCR配置
ccode list-ccr            # 列出CCR配置
ccode use-ccr <name>      # 设置默认CCR配置
ccode run-ccr [name]      # 启动CCR配置
ccode remove-ccr <name>   # 删除CCR配置
```

### ⚙️ CCR服务命令

```bash
ccode ccr start           # 启动CCR服务
ccode ccr stop            # 停止CCR服务
ccode ccr restart         # 重启CCR服务
ccode ccr status          # 查看服务状态
ccode ccr logs            # 查看服务日志
```

### 📊 Provider命令

```bash
ccode provider list       # 列出Providers
ccode provider add <name> # 添加Provider
ccode provider show <name># 查看Provider详情
ccode provider edit <name># 编辑Provider
ccode provider remove <name># 删除Provider
```

## 📁 配置文件

### 配置存储位置
- **Linux/macOS**: `~/.config/ccode/config.json`
- **Windows**: `%APPDATA%/ccode/config.json`
- **CCR配置**: `~/.claude-code-router/config.json`

### 配置文件结构

```json
{
  "version": "2.0",
  "groups": {
    "direct": {
      "default_profile": "myapi",
      "profiles": {
        "myapi": {
          "ANTHROPIC_AUTH_TOKEN": "your-token",
          "ANTHROPIC_BASE_URL": "https://api.example.com",
          "description": "我的API服务",
          "created_at": "2025-07-31T10:00:00Z"
        }
      }
    },
    "ccr": {
      "default_profile": "production", 
      "profiles": {
        "production": {
          "description": "生产环境CCR配置",
          "providers": [...],
          "router": {...},
          "created_at": "2025-07-31T10:00:00Z"
        }
      }
    }
  }
}
```

## 🔧 工作原理

### Direct模式
1. 读取Direct配置中的token和base_url
2. 设置环境变量：`ANTHROPIC_AUTH_TOKEN`、`ANTHROPIC_BASE_URL`
3. 启动claude程序

### CCR模式  
1. 生成CCR配置文件到`~/.claude-code-router/config.json`
2. 启动CCR服务（监听localhost:3456）
3. 设置环境变量指向CCR代理
4. Claude请求通过CCR智能路由到最适合的模型

### 智能路由策略

CCR根据请求特征自动选择模型：

- **默认任务** → `default`配置的模型
- **后台任务** → 高性价比的`background`模型
- **推理任务** → 强推理能力的`think`模型  
- **长上下文** → 大窗口的`longContext`模型（超过阈值时）
- **网络搜索** → 支持搜索的`webSearch`模型

## 🎯 使用场景

### 个人开发者
- Direct模式：简单API切换，快速上手
- CCR模式：多模型测试，成本优化

### 团队协作
- 标准化多环境配置（开发/测试/生产）
- 智能路由降低API成本
- 统一的配置管理和分享

### 企业用户
- 多Provider容灾和负载均衡
- 精细化的成本控制
- 合规和安全的配置管理

## ⚠️ 重要说明

### 兼容性
- **向后兼容**：现有Direct模式配置无需修改
- **配置迁移**：自动从v1.0配置格式升级到v2.0
- **CCR依赖**：CCR模式需要npm环境，但会自动管理依赖

### 系统要求
- **官方支持**：Ubuntu 22.04 LTS（CI/CD标准环境）
- **兼容性测试**：Windows、macOS、其他Linux发行版
- **运行时要求**：现代Linux发行版，glibc 2.31+

### 安全注意事项
- API密钥加密存储（计划中）
- 配置文件权限控制
- CCR服务默认仅监听localhost

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
- **异步运行时**: tokio (CCR服务管理)
- **HTTP客户端**: reqwest (CCR API交互)

### 依赖管理
- **序列化**: serde + serde_json
- **目录处理**: dirs (跨平台)
- **时间处理**: chrono
- **错误处理**: anyhow
- **系统信息**: sysinfo

### 质量保证
- **测试覆盖**: 单元测试 + 集成测试 (7个核心测试)
- **代码质量**: Zero warnings (clippy + rustfmt)
- **代码行数**: 3,122 行精简高效代码
- **安全扫描**: cargo-audit 自动检查
- **CI/CD**: GitHub Actions 全平台构建

## 🛠️ 开发

### 项目结构

```
src/
├── main.rs           # CLI入口和命令路由
├── commands.rs       # 命令实现逻辑
├── config.rs         # 配置数据结构和管理
├── ccr_config.rs     # CCR配置文件管理
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

**最后更新**: 2025-07-31 | **架构版本**: v2.0（双模式架构）