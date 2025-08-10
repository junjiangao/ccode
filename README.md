# ccode 🚀

**Claude Code 配置管理工具** - 支持双模式配置的Claude环境快速切换工具

[![CI Status](https://github.com/junjiangao/ccode/workflows/CI/badge.svg)](https://github.com/junjiangao/ccode/actions)
[![Release](https://github.com/junjiangao/ccode/workflows/Release/badge.svg)](https://github.com/junjiangao/ccode/actions)
[![Version](https://img.shields.io/github/v/release/junjiangao/ccode?include_prereleases)](https://github.com/junjiangao/ccode/releases)
[![License](https://img.shields.io/github/license/junjiangao/ccode)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgray)](https://github.com/junjiangao/ccode/releases)

## ✨ 核心特性

### 🔄 双模式架构
- **🎯 Direct模式**：简单的API配置，直接启动claude
- **🛠️ Router模式**：通过RouterProfile管理复杂路由配置

### 🌟 主要功能
- 📋 **配置管理**：支持多配置存储和快速切换
- 🔀 **路由配置**：管理RouterProfile，支持不同场景的模型路由
- 🎛️ **Provider管理**：管理claude-code-router的provider配置
- 🔄 **配置同步**：自动同步CCR配置文件，确保配置信息实时一致
- ⚡ **精确更新**：精确更新配置节点，避免重写整个配置文件
- 📱 **交互式操作**：友好的命令行交互界面
- 🌐 **跨平台支持**：Windows、macOS、Linux

### 🛠️ 工作模式
- **Direct模式**：传统的token+base_url配置方式，直接启动claude程序
- **Router模式**：管理RouterProfile配置，通过外部`ccr`命令启动路由功能

## 🚀 快速开始

### 📋 系统要求

- **Rust**: 1.70+（如需从源码编译）
- **Claude CLI**: 已安装claude命令行工具
- **ccr工具**: Router模式需要安装claude-code-router工具

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

### 🛠️ Router模式（路由配置）

适合需要管理复杂路由配置的场景，依赖外部ccr工具。

#### 添加Provider
```bash
ccode provider add deepseek
```

按提示配置Provider信息：
- API Base URL
- API Key  
- 支持的模型列表
- Provider类型

#### 添加RouterProfile
```bash
ccode add-ccr production
```

交互式配置路由规则：
- default: 默认路由
- background: 后台任务路由
- think: 推理任务路由
- longContext: 长上下文路由
- webSearch: 网络搜索路由

#### 使用Router配置
```bash
# 列出RouterProfile
ccode list-ccr

# 设置默认RouterProfile
ccode use-ccr production

# 启动claude（通过ccr工具）
ccode run-ccr production
```

## 📋 命令参考

### 🔄 统一命令

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

### 🛠️ Router模式快捷命令

专门针对Router模式的便捷命令：

```bash
ccode add-ccr <name>      # 添加RouterProfile
ccode list-ccr            # 列出RouterProfile
ccode use-ccr <name>      # 设置默认RouterProfile
ccode run-ccr [name]      # 启动RouterProfile（通过ccr工具）
ccode remove-ccr <name>   # 删除RouterProfile
```

### 📊 Provider管理命令

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
- **CCR配置**: `~/.claude-code-router/config.json`（由ccode管理）

### ccode配置文件结构

```json
{
  "version": "2.0",
  "default_group": "direct",
  "default_profile": {
    "direct": "myapi",
    "router": "production"
  },
  "groups": {
    "direct": {
      "myapi": {
        "ANTHROPIC_AUTH_TOKEN": "your-token",
        "ANTHROPIC_BASE_URL": "https://api.example.com",
        "description": "我的API服务",
        "created_at": "2025-07-31T10:00:00Z"
      }
    },
    "router": {
      "production": {
        "name": "production",
        "router": {
          "default": "deepseek,deepseek-chat",
          "background": "qwen,qwen-plus",
          "think": "deepseek,deepseek-reasoner",
          "longContext": "qwen,qwen-max",
          "longContextThreshold": 60000,
          "webSearch": "qwen,qwen-plus"
        },
        "description": "生产环境路由配置",
        "created_at": "2025-07-31T10:00:00Z"
      }
    }
  }
}
```

### CCR配置文件结构

**文件位置**: `~/.claude-code-router/config.json`（由ccode自动管理）

```json
{
  "providers": [
    {
      "name": "deepseek",
      "api_base_url": "https://api.deepseek.com/chat/completions",
      "api_key": "sk-xxx",
      "models": ["deepseek-chat", "deepseek-reasoner"],
      "provider_type": "deepseek"
    },
    {
      "name": "qwen",
      "api_base_url": "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
      "api_key": "sk-xxx", 
      "models": ["qwen-plus", "qwen-max"],
      "provider_type": "qwen"
    }
  ],
  "router": {
    "default": "deepseek,deepseek-chat",
    "background": "qwen,qwen-plus",
    "think": "deepseek,deepseek-reasoner",
    "longContext": "qwen,qwen-max",
    "longContextThreshold": 60000,
    "webSearch": "qwen,qwen-plus"
  },
  "transformer": {
    "use": ["deepseek"],
    "deepseek-chat": {"use": ["tooluse"]}
  }
}
```

## 🔧 工作原理

### Direct模式
1. 读取Direct配置中的token和base_url
2. 设置环境变量：`ANTHROPIC_AUTH_TOKEN`、`ANTHROPIC_BASE_URL`
3. 启动claude程序

### Router模式
1. **配置同步**：每次命令执行前自动同步CCR配置文件状态
2. **读取RouterProfile**：从ccode配置中读取路由规则
3. **精确配置应用**：将RouterProfile精确应用到CCR配置文件的Router节点
4. **启动路由**：调用外部`ccr code`命令启动路由功能

### 配置管理架构

```
┌─────────────────┐    精确管理    ┌──────────────────────┐
│ ccode配置        │ ─────────────→ │ CCR配置文件          │
│ ~/.config/ccode  │    配置同步    │ ~/.claude-code-router │
├─────────────────┤                ├──────────────────────┤
│ router组:        │                │ providers: []        │
│ • RouterProfile  │                │ router: {}           │
│ • 路由规则       │                │ transformer: {}      │
│ • 元数据         │                │                      │
└─────────────────┘                └──────────────────────┘
         │                                    │
         │ ccode命令                          │ ccr工具
         ▼                                    ▼
┌─────────────────┐                ┌──────────────────────┐
│ 配置管理         │                │ 路由执行             │
│ • add-ccr       │                │ • ccr code           │
│ • list-ccr      │                │ • 智能路由           │
│ • provider管理   │                │ • API转换            │
│ • 配置同步       │                │                      │
└─────────────────┘                └──────────────────────┘
```

## 🎯 使用场景

### 个人开发者
- **Direct模式**：简单API切换，快速上手
- **Router模式**：管理多个API服务的路由配置

### 团队协作
- 标准化配置管理（开发/测试/生产）
- 统一的RouterProfile配置和分享
- 集中化的Provider管理

### 高级用户
- 复杂的路由规则配置
- 多Provider的配置管理
- 与claude-code-router的深度集成

## ⚠️ 重要说明

### 系统依赖
- **claude CLI**：必须预先安装claude命令行工具
- **ccr工具**：Router模式需要安装claude-code-router
- **配置文件**：ccode仅管理配置，不包含服务管理功能

### 兼容性
- **向后兼容**：现有Direct模式配置无需修改
- **配置迁移**：自动从v1.0配置格式升级到v2.0
- **外部依赖**：依赖外部ccr工具进行路由功能

### 限制说明
- ccode不包含CCR服务管理功能（start/stop/restart等）
- Router模式需要用户自行安装和管理ccr工具
- 配置文件管理功能仅限于RouterProfile和Provider

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
- **异步运行时**: tokio (配置管理)
- **HTTP客户端**: reqwest (外部API交互)

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

**最后更新**: 2025-08-10 | **架构版本**: v0.2.0（配置管理工具）