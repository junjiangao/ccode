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

### 🎯 Direct 模式（新配置：config.toml）

自 v0.2.0 起，ccode 使用 `~/.config/ccode/config.toml` 进行配置管理。

#### 字段映射（旧 → 新）
- ANTHROPIC_SMALL_FAST_MODEL（弃用） → ANTHROPIC_DEFAULT_HAIKU_MODEL → `model_haiku`
- ANTHROPIC_DEFAULT_HAIKU_MODEL → `model_haiku`
- ANTHROPIC_DEFAULT_OPUS_MODEL → `model_opus`
- ANTHROPIC_DEFAULT_SONNET_MODEL → `model_sonnet`
- ANTHROPIC_MODEL → `model`
- ANTHROPIC_AUTH_TOKEN → 通过 `env_key` 指定的环境变量读取（同级 `.env` 或系统环境）
- ANTHROPIC_BASE_URL → `base_url`
- CLAUDE_CODE_MAX_OUTPUT_TOKENS → `max_tokens`

#### 配置文件位置
- Linux/macOS: `~/.config/ccode/config.toml`
- Windows: `%APPDATA%/ccode/config.toml`

#### config.toml 示例
```toml
default = "uos-minimax2"

[profiles.uos-minimax2]
name = "uos-minimax2"
base_url = "https://api.anthropic.com"
env_key  = "uos_minimax2_key"       # 对应配置目录下 .env 中的变量名
model = "claude-3-5-sonnet-latest"
model_sonnet = "claude-3-5-sonnet-20241022"
model_haiku  = "claude-3-haiku-20240307"
model_opus   = "claude-3-opus-latest"
max_tokens   = "32000"
comment = "公司内部API"

[profiles.anyrouter]
name = "anyrouter"
base_url = "https://api.example.com"
env_key  = "anyrouter_key"
```

同级 `.env` 示例（可选，优先于系统环境加载）：
```env
# 文件路径：~/.config/ccode/.env
uos_minimax2_key="sk-xxx..."
anyrouter_key="sk-yyy..."
```

#### 使用 TOML 配置
```bash
# 列出配置（自动识别 TOML）
ccode list

# 运行（未指定 name 时使用 default）
ccode run                      # 使用 default
ccode run uos-minimax2         # 指定 profile

# 透传参数
ccode run uos-minimax2 --version
ccode run uos-minimax2 -- --help
```

#### 添加配置（交互式）
```bash
ccode add myapi
# 按提示依次输入：
# 1) ANTHROPIC_BASE_URL（如 https://api.anthropic.com）
# 2) ANTHROPIC_AUTH_TOKEN（密钥值，工具会保存到配置目录 .env）
```

`ccode add/use/remove` 支持直接写入 `config.toml` 并使用同级 `.env` 保存密钥。

<!-- Router/Provider 功能已移除 -->

## 📋 命令参考

### 🔄 命令

```bash
# 列出配置（使用 config.toml）
ccode list

# 添加/设置默认/删除（写入 config.toml）
ccode add <name>
ccode use <name>
ccode remove <name>

# 启动 claude（根据 config.toml 映射环境变量）
ccode run [name] [<claude_args>...]

# 示例：
# ccode run myapi --version                    # 直接透传
# ccode run myapi code                         # 透传子命令
# ccode run myapi -- --help                    # 使用 -- 分隔符避免冲突

# 删除配置
ccode remove <name> [--group direct]
```

<!-- Router/Provider 快捷命令已移除 -->

## 📁 配置文件

### 配置存储位置（新）
- **Linux/macOS**: `~/.config/ccode/config.toml`
- **Windows**: `%APPDATA%/ccode/config.toml`

请在上述路径创建并维护 `config.toml` 与同级 `.env`。

<!-- CCR 配置文件相关章节已移除 -->

## 🔧 工作原理

### Direct 模式（TOML）
1. 解析 `~/.config/ccode/config.toml`，选取指定或默认 `profile`
2. 加载同级 `.env`（若存在），并回落至系统环境
3. 设置环境变量：
   - `ANTHROPIC_AUTH_TOKEN` ← 由 `env_key` 指向的值
   - `ANTHROPIC_BASE_URL` ← `base_url`
   - `ANTHROPIC_MODEL` ← `model`（可选）
   - `ANTHROPIC_DEFAULT_HAIKU_MODEL` ← `model_haiku`（可选）
   - `ANTHROPIC_DEFAULT_SONNET_MODEL` ← `model_sonnet`（可选）
   - `ANTHROPIC_DEFAULT_OPUS_MODEL` ← `model_opus`（可选）
   - `CLAUDE_CODE_MAX_OUTPUT_TOKENS` ← `max_tokens`（可选）
4. 透传参数并启动 `claude`

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
- **配置方式**：采用 `config.toml`；旧版 `config.json` 不再作为管理入口
- **命令迁移**：`add/use/remove/list/run` 全部基于 `config.toml`

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
[必填项说明]
- 每个 profile 必填：`name`、`base_url`、`env_key`
- 其他字段（`model*`、`max_tokens`、`comment`）均为可选
