# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

`ccode` 是一个用 Rust 编写的命令行配置管理工具，专为 `claude` CLI 和 `claude-code-router` (ccr) 设计。它采用双模式架构，简化不同配置环境的切换。

### 🎯 核心架构

- **Direct 模式**：传统的简单API配置方式（向后兼容）。
  - 直接配置 `ANTHROPIC_AUTH_TOKEN` 和 `ANTHROPIC_BASE_URL`（必需）。
  - 可选配置 `ANTHROPIC_MODEL` 和 `ANTHROPIC_SMALL_FAST_MODEL` 实现精确模型控制。
  - 适合单一API服务的简单切换需求。
  - **参数透传**：支持直接将参数透传给 `claude` 命令。

- **Router 模式**：通过管理 `RouterProfile` 来支持 `claude-code-router` 的复杂路由配置。
  - **Provider 管理**：支持管理不同的后端服务（如 DeepSeek, Qwen 等）。
  - **路由规则**：为不同场景（如默认、后台、思考等）配置不同的模型路由。
  - **配置同步**：自动将 `ccode` 中的路由配置同步到 `ccr` 的配置文件中。
  - **精确更新**：更新配置时只修改变动节点，而非重写整个文件。

### ⚠️ 重要说明

- `ccode` **仅管理配置**，不包含 `ccr` 的服务管理功能（如 `start`/`stop`）。
- `Router` 模式依赖用户**自行安装和管理** `ccr` 工具。

## 开发命令

### 构建和测试
```bash
# 开发构建
cargo build

# 生产构建
cargo build --release

# 运行测试
cargo test
```

### 代码质量检查
```bash
# 代码格式化 (提交前必须)
cargo fmt

# 代码质量检查 (要求零警告)
cargo clippy -- -D warnings

# 安全漏洞扫描
cargo audit

# CI流程完整检查
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### 安装和运行
```bash
# 从源码编译并安装到系统
cargo install --path .

# 或者直接运行
cargo run -- <subcommand>
```

## 项目架构

### 技术栈
- **语言**：Rust 2024 Edition (最低要求 Rust 1.70+)
- **CLI框架**：clap 4.x (derive API)
- **序列化**：serde + serde_json
- **系统信息**：sysinfo
- **目录处理**：dirs (跨平台)
- **时间处理**：chrono
- **错误处理**：anyhow

### 核心模块结构

```
src/
├── main.rs          # CLI入口，命令路由和参数解析
├── commands.rs      # 所有命令的具体实现逻辑
├── config.rs        # ccode配置数据结构和管理
├── ccr_config.rs    # ccr配置文件(config.json)的管理
├── error.rs         # 统一错误处理
└── lib.rs           # 库入口，模块导出
```

### 配置系统架构

- **ccode 配置**: `~/.config/ccode/config.json`
- **ccr 配置**: `~/.claude-code-router/config.json` (由 `ccode` 自动管理)

`ccode` 读取自身的配置文件，并根据 `Router` 模式的配置去精确更新 `ccr` 的配置文件。

## 命令组织模式

### 统一接口命令 (支持 `--group` 参数)
- `list [--group direct|router]` - 列出指定组配置
- `add <name> [--group direct|router]` - 添加配置到指定组
- `use <name> [--group direct|router]` - 设置指定组默认配置
- `run [name] [--group direct|router] [<claude_args>...]` - 运行指定组配置，支持透传参数给claude（仅Direct模式）
- `remove <name> [--group direct|router]` - 删除指定组配置

### Router 模式快捷命令
- `add-ccr <name>` - 快速添加RouterProfile
- `list-ccr` - 列出所有RouterProfile
- `use-ccr <name>` - 设置默认RouterProfile
- `run-ccr [name]` - 使用指定RouterProfile启动 (调用外部`ccr`命令)
- `remove-ccr <name>` - 删除RouterProfile

### Provider 管理命令
- `provider list` - 列出所有Providers
- `provider add <name>` - 添加新Provider
- `provider remove <name>` - 删除Provider
- `provider show <name>` - 显示Provider详情
- `provider edit <name>` - 编辑Provider配置

## Direct 模式环境变量配置

### 🔧 支持的环境变量

#### 必需环境变量
- **`ANTHROPIC_AUTH_TOKEN`**: API认证令牌
  - 用于API服务的身份验证
  - 所有Direct配置都必须设置此字段
  
- **`ANTHROPIC_BASE_URL`**: API基础URL
  - 指定API服务的访问地址
  - 支持官方API和第三方兼容API

#### 可选环境变量
- **`ANTHROPIC_MODEL`**: 指定默认使用的模型
  - 精确控制claude使用的AI模型
  - 例如：`claude-3-5-sonnet-20241022`
  - 为空时使用claude的默认模型选择

- **`ANTHROPIC_SMALL_FAST_MODEL`**: 指定快速响应场景的模型  
  - 用于需要快速响应的轻量级任务
  - 例如：`claude-3-haiku-20240307`
  - 为空时使用claude的默认快速模型选择

### 💡 使用场景

#### 基础配置（仅必需变量）
```bash
ccode add basic-api
# 输入Token和URL，跳过可选字段
# 使用claude的默认模型选择策略
```

#### 精确模型控制（包含可选变量）
```bash
ccode add precise-api
# 输入Token和URL
# 设置主模型：claude-3-5-sonnet-20241022
# 设置快速模型：claude-3-haiku-20240307
# 实现精确的模型控制
```

### 📋 配置特性

- **向后兼容**：现有配置无需修改，自动兼容
- **渐进配置**：用户可根据需要选择性设置可选变量
- **条件注入**：只有设置的环境变量才会传递给claude命令
- **模型优化**：支持针对不同场景的模型优化选择

## 参数透传功能

### 概述
`ccode` 支持将额外参数透传给 `claude` 命令，该功能仅在 **Direct 模式** 下可用。

### 使用方式
支持两种参数透传方式：

1. **直接透传**（推荐用于无冲突参数）
2. **使用 `--` 分隔符**（用于可能冲突的参数）

```bash
# 直接透传（适用于大多数情况）
ccode run [name] [--group direct] <claude_args>...

# 使用 -- 分隔符（避免参数冲突）
ccode run [name] [--group direct] -- <claude_args>...

# 示例
ccode run myapi --version                        # 直接透传 ✅
ccode run myapi code --project myapp             # 直接透传 ✅  
ccode run myapi -- --help                       # 使用分隔符避免冲突 ✅
ccode run myapi --help                          # ❌ 会显示ccode帮助而非claude帮助
```

### 功能特性
- **双模式支持**：支持直接透传和 `--` 分隔符两种方式
- **智能冲突处理**：自动识别参数冲突并在提示中说明解决方案
- **完整透传**：支持所有 `claude` 命令的参数和选项
- **模式限制**：仅在 Direct 模式下生效，CCR 模式会忽略透传参数并显示警告

### 实现原理
1. 使用 `trailing_var_arg = true` 解析尾随参数，支持两种使用方式
2. **直接透传**：参数直接被 clap 收集为尾随参数
3. **`--` 分隔符**：clap 自动识别并正确处理分隔符后的参数
4. **冲突检测**：当参数与 ccode 自身参数冲突时，建议使用 `--` 分隔符
5. 在 Direct 模式下将收集的参数附加到 `claude` 命令执行

## 开发注意事项

### 代码质量要求
- **零警告**: `cargo clippy -- -D warnings`
- **强制格式化**: `cargo fmt`
- **安全扫描**: `cargo audit`

### 代码重构和质量改进

#### 🏗️ 架构设计原则
- **DRY原则**：通过抽象化消除重复代码，提高代码可维护性
- **单一责任原则**：每个函数专注单一功能，提升代码清晰度
- **抽象化设计**：创建通用函数处理相似逻辑模式

#### ✨ 最新重构改进 
- **输入处理优化**：统一的 `read_optional_input()` 函数处理所有可选输入
- **显示逻辑封装**：`DirectProfile::display_optional_fields()` 方法统一显示逻辑
- **代码行数减少**：消除约30行重复代码，提升维护效率
- **测试覆盖完善**：所有新功能都有相应的单元测试覆盖

#### 🎯 维护性提升
- **统一修改点**：添加新字段只需在一处修改，避免遗漏
- **错误风险降低**：减少重复代码降低维护时的错误风险
- **扩展性改善**：为未来功能扩展建立良好的代码模式

### Git提交流程要求
**IMPORTANT: 提交代码前必须执行格式化**
```bash
# 每次git提交前必须执行以下命令
cargo fmt
```
此举是为了确保代码风格统一，避免CI构建失败。

### 错误处理模式
使用 `anyhow::Result<T>` 作为统一的错误返回类型（别名为 `AppResult<T>`），所有错误通过 `AppError` 枚举统一处理。
