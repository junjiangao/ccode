# ccode 🚀

**Claude Code 环境切换工具** - 一个用于快速切换不同API服务配置并启动claude程序的命令行工具

[![CI Status](https://github.com/junjiangao/ccode/workflows/CI/badge.svg)](https://github.com/junjiangao/ccode/actions)
[![Release](https://github.com/junjiangao/ccode/workflows/Release/badge.svg)](https://github.com/junjiangao/ccode/actions)
[![Version](https://img.shields.io/github/v/release/junjiangao/ccode?include_prereleases)](https://github.com/junjiangao/ccode/releases)
[![License](https://img.shields.io/github/license/junjiangao/ccode)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgray)](https://github.com/junjiangao/ccode/releases)

## ✨ 特性

- 🔧 **多配置管理**：支持管理多个API服务配置（如anyrouter、instcopilot等）
- 🔄 **一键切换**：快速切换不同的API环境
- 🚀 **自动启动**：设置环境变量后自动启动claude程序
- 💻 **交互式操作**：友好的命令行交互界面
- 🎯 **默认配置**：支持设置和管理默认配置
- 🛡️ **配置验证**：自动验证URL格式，支持各种第三方API token
- 🌐 **跨平台**：支持Windows、macOS、Linux

## 📦 安装

### 🚀 预编译二进制文件（推荐）

从 [GitHub Releases](https://github.com/junjiangao/ccode/releases) 下载对应平台的预编译二进制文件：

```bash
# Linux (Debian 10+)
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

# Windows
# 下载 ccode-windows-x86_64.exe 并放到 PATH 中
```

### 🔨 从源码编译

```bash
git clone <repository-url>
cd ccode
cargo build --release
```

编译完成后，可执行文件位于 `target/release/ccode`

### 添加到PATH

```bash
# Linux/macOS
export PATH="$PATH:/path/to/ccode/target/release"

# 或者复制到系统目录
sudo cp target/release/ccode /usr/local/bin/
```

## 🚀 快速开始

### 1. 添加第一个配置

```bash
ccode add anyrouter
```

按提示输入：
- ANTHROPIC_AUTH_TOKEN: `your-api-token-here`
- ANTHROPIC_BASE_URL: `https://anyrouter.top`
- 描述（可选）: `AnyRouter API服务`

### 2. 查看配置

```bash
ccode list
```

### 3. 启动claude

```bash
# 使用默认配置启动
ccode run

# 使用指定配置启动
ccode run anyrouter
```

## 📋 命令参考

### `ccode list`
列出所有已配置的API服务

```bash
$ ccode list
📋 可用配置：

🔧 anyrouter (默认)
   📍 URL: https://anyrouter.top
   🔑 Token: your-token...xyz
   📝 描述: AnyRouter API服务
   📅 创建: 2025-07-27 15:30:00 UTC

🔧 instcopilot
   📍 URL: https://instcopilot-api.com
   🔑 Token: your-token...abc
   📝 描述: InstCopilot API服务
   📅 创建: 2025-07-27 15:35:00 UTC
```

### `ccode add <name>`
交互式添加新配置

```bash
$ ccode add instcopilot
🔧 添加新配置: instcopilot

🔑 请输入 ANTHROPIC_AUTH_TOKEN (支持各种第三方API格式): your-api-token
📍 请输入 ANTHROPIC_BASE_URL (如: https://api.anthropic.com): https://instcopilot-api.com
📝 请输入描述 (可选，直接回车跳过): InstCopilot API服务

✅ 配置 'instcopilot' 添加成功！
```

### `ccode use <name>`
设置默认配置

```bash
$ ccode use instcopilot
✅ 已将 'instcopilot' 设为默认配置
```

### `ccode run [name]`
启动claude程序

```bash
# 使用默认配置
$ ccode run
🚀 使用配置 'anyrouter' 启动 claude...
📍 API URL: https://anyrouter.top

# 使用指定配置
$ ccode run instcopilot
🚀 使用配置 'instcopilot' 启动 claude...
📍 API URL: https://instcopilot-api.com
```

### `ccode remove <name>`
删除配置

```bash
$ ccode remove oldconfig
⚠️  确定要删除配置 'oldconfig' 吗？(y/N): y
✅ 配置 'oldconfig' 已删除
```

## 📁 配置文件

配置文件自动保存在系统配置目录：

- **Linux/macOS**: `~/.config/ccode/config.json`
- **Windows**: `%APPDATA%/ccode/config.json`

### 配置文件格式

```json
{
  "version": "1.0",
  "default": "anyrouter",
  "profiles": {
    "anyrouter": {
      "ANTHROPIC_AUTH_TOKEN": "your-api-token",
      "ANTHROPIC_BASE_URL": "https://anyrouter.top",
      "description": "AnyRouter API服务",
      "created_at": "2025-07-27 15:30:00 UTC"
    },
    "instcopilot": {
      "ANTHROPIC_AUTH_TOKEN": "your-another-token",
      "ANTHROPIC_BASE_URL": "https://instcopilot-api.com",
      "description": "InstCopilot API服务",
      "created_at": "2025-07-27 15:35:00 UTC"
    }
  }
}
```

### 手动编辑配置

你可以直接编辑配置文件来批量添加配置，但建议使用 `ccode add` 命令以确保格式正确。

## 🔧 工作原理

ccode通过设置环境变量来让claude程序使用不同的API服务：

1. **读取配置**：从配置文件中读取指定的配置
2. **设置环境变量**：
   - `ANTHROPIC_AUTH_TOKEN`: 认证令牌
   - `ANTHROPIC_BASE_URL`: API基础URL
3. **启动claude**：使用设置的环境变量启动claude程序

## ⚠️ 注意事项

- 确保claude程序已安装并在PATH中
- 支持各种第三方API token格式，无格式限制
- URL必须以 `http://` 或 `https://` 开头
- 首次添加的配置会自动设为默认配置
- 删除默认配置时会自动选择其他配置作为新默认

## 📊 构建状态

| 平台 | 状态 | 说明 |
|------|------|------|
| **持续集成** | [![CI Status](https://github.com/junjiangao/ccode/workflows/CI/badge.svg)](https://github.com/junjiangao/ccode/actions) | 代码质量检查、测试、安全扫描 (Debian 10) |
| **自动发布** | [![Release](https://github.com/junjiangao/ccode/workflows/Release/badge.svg)](https://github.com/junjiangao/ccode/actions) | 跨平台二进制文件构建与发布 (Linux使用Debian 10) |
| **Linux (Debian 10)** | ✅ 支持 | 主要测试环境 + 发布构建环境 |
| **Windows** | ✅ 支持 | 跨平台兼容测试 |
| **macOS** | ✅ 支持 | Intel + Apple Silicon |

### 🔧 技术栈

- **语言**: Rust 2024 Edition
- **最低版本**: Rust 1.70+
- **测试覆盖**: 11 个单元测试
- **代码质量**: Zero warnings (clippy + rustfmt)
- **安全扫描**: cargo-audit 自动检查

## 🛠️ 开发

### 项目结构

```
src/
├── main.rs          # CLI入口和命令路由
├── config.rs        # 配置文件管理
├── commands.rs      # 命令实现
└── error.rs         # 错误处理
```

### 依赖项

- `serde` + `serde_json`: JSON序列化
- `clap`: 命令行参数解析
- `dirs`: 跨平台目录处理
- `chrono`: 时间戳处理
- `anyhow`: 错误处理

### 编译

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test
```

### 🧪 质量检查

本地运行与CI相同的检查：

```bash
# 代码格式检查
cargo fmt --check

# 代码质量检查（零警告）
cargo clippy -- -D warnings

# 运行所有测试
cargo test

# 安全漏洞扫描
cargo install cargo-audit
cargo audit

# 完整的CI检查流程
cargo fmt --check && \
cargo clippy -- -D warnings && \
cargo test && \
cargo build --release
```

### 🔄 CI/CD 流程

项目使用 GitHub Actions 实现自动化：

- **🔍 持续集成**: 每次 push 和 PR 都会触发
  - 代码格式检查 (rustfmt)
  - 代码质量检查 (clippy)
  - 单元测试执行
  - 跨平台构建验证 (Debian 10, Windows, macOS)
  - 安全漏洞扫描 (cargo-audit)

- **🚀 自动发布**: git tag 推送时触发
  - 多平台二进制构建 (5个目标平台，Linux使用Debian 10)
  - GitHub Releases 自动创建
  - 源码归档和资产上传

- **🔧 依赖管理**: Dependabot 自动维护
  - 每周检查 Rust 依赖更新
  - GitHub Actions 版本更新

## 📄 许可证

本项目采用 [LICENSE](LICENSE) 许可证。

## 🤝 贡献

欢迎提交Issue和Pull Request！

---

**最后更新**: 2025-07-29