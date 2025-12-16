# CLAUDE.md

版本: v0.4.0 | 更新日期: 2025-11-17

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**`ccode` 是一个双重性质的项目**：

1. **Rust CLI 工具**：用于管理 `claude` CLI 的 Direct 模式配置与快速切换
2. **Claude Code 插件仓库**：提供系统通知、智能提交等扩展功能

两个系统独立运行但互补协作：CLI 工具负责配置管理与启动，插件系统扩展 Claude Code 的功能。

### 🎯 CLI 工具核心架构（仅 Direct）

- 通过 `config.toml` 指定 `base_url` 与 `env_key`（从同级 `.env` 或系统环境读取 token）。
- 可选配置 `model` 与家族模型：`model_haiku`、`model_sonnet`、`model_opus`（对应 `ANTHROPIC_DEFAULT_*` 环境变量）。
- 兼容性：当 `model_haiku` 存在时，同时设置 `ANTHROPIC_DEFAULT_HAIKU_MODEL` 与（兼容）`ANTHROPIC_SMALL_FAST_MODEL` 两个环境变量。
- 支持参数透传到 `claude` 命令。

### 🔌 插件系统概览

- **ccode-notify**：桌面通知插件（Notification/Stop hooks）
- **ccode-skills**：技能集合插件
  - `codex-ai`：通过 Codex CLI 进行代码审查、算法设计和架构分析
  - `git-commit`：智能 Git 提交助手

### ⚠️ 重要说明

- CLI 工具仅管理配置与启动，不包含服务管理功能。
- 插件通过 `.claude-plugin/marketplace.json` 注册到 Claude Code。

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

### CLI 工具模块结构

```
src/
├── main.rs          # CLI入口，命令路由和参数解析（包含启动自动迁移钩子）
├── commands.rs      # 所有命令的具体实现逻辑（含 ccode config merge）
├── toml_config.rs   # TOML 格式配置与 .env 读取/持久化
├── migrate.rs       # JSON→TOML 迁移/合并实现（自动/手动）
├── config.rs        # 旧 JSON 结构（仅迁移使用）
├── error.rs         # 统一错误处理
└── lib.rs           # 库入口，模块导出
```

## 插件系统架构

### 插件组织结构

```
plugins/
├── ccode-notify/              # 桌面通知插件
│   ├── .claude-plugin/
│   │   └── plugin.json        # 插件元数据
│   ├── hooks/
│   │   └── hooks.json         # Notification & Stop hooks 配置
│   └── hooks-handlers/
│       ├── notify-interaction.py  # 交互提醒脚本
│       └── notify-stop.py         # 会话停止提醒脚本
│
└── ccode-skills/              # 技能集合插件
    ├── .claude-plugin/
    │   └── plugin.json        # 插件元数据
    ├── codex-ai/              # Codex CLI 协作技能
    │   ├── SKILL.md           # 技能定义（Claude Code 读取）
    │   ├── README.md          # 快速入门
    │   └── REFERENCE.md       # 完整参考
    │
    └── git-commit/            # Git 提交助手技能
        ├── SKILL.md           # 技能定义
        ├── README.md          # 快速入门
        └── REFERENCE.md       # 完整参考
```

### 插件注册

插件通过 `.claude-plugin/marketplace.json` 注册：

```json
{
  "plugins": [
    {
      "name": "ccode-notify",
      "path": "plugins/ccode-notify"
    },
    {
      "name": "ccode-skills",
      "path": "plugins/ccode-skills"
    }
  ]
}
```

### Hooks 插件：ccode-notify

**功能**：
- **Notification Hook**：Claude 等待用户输入时发送桌面通知
- **Stop Hook**：会话停止时根据原因（complete/user_stop/error）发送不同通知

**技术实现**：
- 基于 `notify-send`（libnotify）
- 脚本超时 5 秒，失败不中断会话
- 零配置，即装即用

**依赖安装**：
```bash
# Debian/Ubuntu
sudo apt-get install -y libnotify-bin

# Arch
sudo pacman -S libnotify

# Fedora
sudo dnf install libnotify
```

### Skills 插件：ccode-skills

#### 技能 1：codex-ai

**用途**：通过 Bash 直接调用 Codex CLI 处理复杂技术任务

**触发场景**：
- 代码审查（review、code review）
- 复杂算法设计（>10行核心逻辑）
- 架构分析与评审（系统扩展、架构重构）
- 性能优化（瓶颈分析、性能调优）

**核心命令**：
```bash
# 代码审查
codex review --uncommitted -m gpt-5.1-codex-max -c model_reasoning_effort=high

# 简单任务（代码审查、简单重构）
codex exec -m gpt-5.1-codex-max -c model_reasoning_effort=high "<任务描述>"

# 复杂任务（算法设计、架构评审、性能优化）
codex exec -m gpt-5.2 -c model_reasoning_effort=high "<任务描述>"
```

**模型选择**：
- **gpt-5.1-codex-max**：简单任务（代码审查、简单重构、文档生成）
- **gpt-5.2**：复杂任务（复杂算法、架构评审、性能优化、多约束问题）

#### 技能 2：git-commit

**用途**：智能化 Git 提交工作流

**完整工作流程**（8 步）：
1. 检查暂存区（`git status --porcelain`、`git diff --cached --stat`）
2. 用户确认（暂存区为空时使用 `AskUserQuestion` 询问提交范围）
3. 简单代码审查（语法错误、调试代码、敏感信息）
4. 分析提交历史（`git log --format="%s" -20`）
5. 读取项目规范（从 Memory 查询 `project:<repo>:commit-convention`）
6. 生成提交信息（使用 `mcp__sequential-thinking__sequentialthinking`）
7. 展示摘要（变更统计 + 提交信息 + 审查结果）
8. 执行提交（用户确认后执行 `git commit`）

**Memory 命名规范**：
```
project:<repo>:commit-convention
```
- 使用 `project:` 前缀确保命名空间隔离
- `<repo>` 替换为实际仓库名（如 `ccode`）

### 配置系统架构（v0.3.0）

- 运行时配置：`~/.config/ccode/config.toml`（同级 `~/.config/ccode/.env` 保存密钥）。
- 迁移策略：
  - 自动迁移：存在 `config.json` 且不存在 `config.toml` 时，启动任意命令后自动迁移并备份，成功后移除 `config.json`。
  - 手动迁移：当两者并存时，执行 `ccode config merge` 进行合并（同名 profile 跳过），成功后移除 `config.json`。
  - 备份路径：`~/.config/ccode/config.json.bak-YYYYMMDD-HHMMSS`；失败不会删除 JSON，可手动回滚。
  - JSON 不再作为运行时来源，仅用于迁移。

## 命令组织模式

### 命令一览（仅 Direct）
- `list [--group direct]` - 列出配置
- `add <name> [--group direct]` - 添加配置
- `use <name> [--group direct]` - 设置默认配置
- `run [name] [--group direct] [<claude_args>...]` - 启动并透传参数到 `claude`
- `remove <name> [--group direct]` - 删除配置
 - `config merge` - 将旧版 `config.json` 合并/迁移到 `config.toml`（成功后移除 JSON）

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
- **`ANTHROPIC_MODEL`**: 默认模型（对应 `model`）
- **`ANTHROPIC_DEFAULT_HAIKU_MODEL`**: Haiku 系列（对应 `model_haiku`）
- **`ANTHROPIC_DEFAULT_SONNET_MODEL`**: Sonnet 系列（对应 `model_sonnet`）
- **`ANTHROPIC_DEFAULT_OPUS_MODEL`**: Opus 系列（对应 `model_opus`）
- **`CLAUDE_CODE_MAX_OUTPUT_TOKENS`**: 最大输出 token（对应 `max_tokens`）
- `ANTHROPIC_SMALL_FAST_MODEL`（已弃用）：为兼容，程序在 `model_haiku` 存在时仍会同时设置该变量。

兼容性说明：当 `model_haiku` 存在时，运行时会同时注入 `ANTHROPIC_DEFAULT_HAIKU_MODEL` 与 `ANTHROPIC_SMALL_FAST_MODEL` 两个变量。

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
- **两种透传方式**：支持直接透传和 `--` 分隔符
- **智能冲突处理**：自动识别参数冲突并在提示中说明解决方案
- **完整透传**：支持所有 `claude` 命令的参数和选项
  

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

## 插件开发指引

### Hooks 插件开发

**基本结构**：
```
plugin-name/
├── .claude-plugin/
│   └── plugin.json          # 插件元数据
├── hooks/
│   └── hooks.json           # Hooks 配置
└── hooks-handlers/
    └── handler-script.py    # 处理脚本
```

**plugin.json 示例**：
```json
{
  "name": "plugin-name",
  "version": "1.0.0",
  "description": "插件描述",
  "type": "hooks"
}
```

**hooks.json 配置**：
```json
{
  "hooks": {
    "Notification": [{
      "hooks": [{
        "type": "command",
        "command": "${CLAUDE_PLUGIN_ROOT}/hooks-handlers/handler.py",
        "timeout": 5
      }]
    }]
  }
}
```

**设计原则**：
- 脚本超时设置合理（建议 5 秒）
- 失败不中断主流程（以 0 退出）
- 使用 `${CLAUDE_PLUGIN_ROOT}` 动态路径
- 轻量、非侵入、失败安全

### Skills 插件开发

**基本结构**：
```
skill-name/
├── SKILL.md              # 技能定义（必需）
├── README.md             # 快速入门
└── REFERENCE.md          # 完整参考
```

**SKILL.md 格式**：
```markdown
---
name: skill-name
description: 技能简短描述
---

# 技能名称

## 使用场景
- 场景 1
- 场景 2

## 工作流程
1. 步骤 1
2. 步骤 2

## MCP 工具调用
\`\`\`json
{
  "name": "mcp__tool__name",
  "parameters": {...}
}
\`\`\`
```

**文档组织**：
- **SKILL.md**：Claude Code 读取的核心定义
- **README.md**：用户快速入门指南
- **REFERENCE.md**：完整技术参考文档

### 插件注册流程

1. 在 `.claude-plugin/marketplace.json` 添加插件：
```json
{
  "plugins": [
    {
      "name": "your-plugin",
      "path": "plugins/your-plugin"
    }
  ]
}
```

2. 测试插件加载：
```bash
# 启动 Claude Code 并验证插件加载
claude code
```

3. 验证功能：
- Hooks 插件：触发对应事件验证
- Skills 插件：使用技能触发词验证

## 关键架构模式

### 1. 配置迁移模式（CLI 工具）
- **自动迁移**：检测到 `config.json` 且无 `config.toml` 时自动迁移
- **手动合并**：`ccode config merge` 合并同名 profile
- **备份机制**：`config.json.bak-YYYYMMDD-HHMMSS`
- **失败保护**：任一步失败不删除原文件

### 2. Hooks 注入模式（ccode-notify）
- 通过 `hooks.json` 注册事件监听
- 使用 `${CLAUDE_PLUGIN_ROOT}` 动态路径
- 脚本失败不中断主流程（以 0 退出）
- 参考 `explanatory-output-style` 的设计思路

### 3. Skills 定义模式（ccode-skills）
- **SKILL.md**：Claude Code 读取的技能定义
- **README.md**：用户快速入门
- **REFERENCE.md**：完整参考文档
- **YAML Front Matter**：`name` 和 `description` 元数据

### 4. 命名空间隔离（Memory）
- 格式：`project:<repo>:<category>:<identifier>`
- 使用 kebab-case
- 避免跨项目污染
- 示例：`project:ccode:commit-convention`

## 迁移指引（摘要）

- 自动：检测到 `config.json` 且缺少 `config.toml` 时自动迁移并备份，成功后删除 JSON。
- 手动：`ccode config merge`；同名跳过；迁移默认 profile；写入 `.env`；成功后删除 JSON；失败不删除。
