# CLAUDE.md

版本: v0.5.1 | 更新日期: 2026-05-07

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**`ccode` 是一个双重性质的项目**：

1. **Rust CLI 工具**：用于管理 `claude` CLI 的 Direct 模式配置与快速切换
2. **Claude Code 插件仓库**：提供系统通知、智能提交等扩展功能

两个系统独立运行但互补协作：CLI 工具负责配置管理与启动，插件系统扩展 Claude Code 的功能。

### 🎯 CLI 工具核心架构（仅 Direct）

- 通过 `config.toml` 指定 `base_url` 与 `env_key`（从同级 `.env` 或系统环境读取 token）。
- 可选配置 `model` 与家族模型：`model_haiku`、`model_sonnet`、`model_opus`、`model_subagent`（对应 `ANTHROPIC_DEFAULT_*` 与 `CLAUDE_CODE_SUBAGENT_MODEL` 环境变量）。
- 兼容性：当 `model_haiku` 存在时，同时设置 `ANTHROPIC_DEFAULT_HAIKU_MODEL` 与（兼容）`ANTHROPIC_SMALL_FAST_MODEL` 两个环境变量。
- 顶层裸 `ccode`（无参数或后跟 `claude` 参数）直接使用默认 profile 启动 `claude`；profile 管理全部归入 `ccode profile <子命令>`。
- 支持参数透传到 `claude` 命令。

### 🔌 插件系统概览

- **ccode-notify**：桌面通知插件（Notification/Stop hooks）
- **git-commit**：独立技能插件（v1.0.0），智能 Git 提交助手
- **codex-ai**：外部插件（位于 `external_plugins/codex-ai`），合并了 Codex MCP 工具与 codex-ai 技能，通过 `mcp__plugin_codex-ai_codex-ai__codex` / `codex-reply` 提供代码审查、算法设计和架构分析能力

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
- **序列化**：serde + serde_json + `toml`（运行时配置）
- **环境加载**：`dotenvy`（读取 `~/.config/ccode/.env`）
- **系统信息**：sysinfo
- **目录处理**：dirs (跨平台)
- **时间处理**：chrono
- **错误处理**：anyhow

### CLI 工具模块结构

```
src/
├── main.rs          # CLI入口，命令路由（`ccode profile` 子命令 + 顶层透传 + 启动自动迁移钩子）
├── commands.rs      # 所有命令的具体实现逻辑（list/add/use/run/remove/clear-env，含环境冲突检测）
├── toml_config.rs   # TOML 格式配置与 .env 读取/持久化
├── tmux_env.rs      # tmux 环境同步与清理（Team/--tmux 场景）
├── migrate.rs       # JSON→TOML 自动迁移实现（启动时触发）
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
│   ├── hooks-handlers/
│   │   ├── notify-interaction.py  # 交互提醒脚本
│   │   └── notify-stop.py         # 会话停止提醒脚本
│   └── README.md
│
└── git-commit/                # Git 提交助手插件（独立版本管理）
    ├── .claude-plugin/
    │   └── plugin.json        # 插件元数据
    └── skills/
        └── git-commit/
            ├── SKILL.md       # 技能定义
            ├── README.md      # 面向使用者的快速入门
            ├── references/    # 规范/模板/安全红线
            ├── examples/      # 端到端示例
            └── scripts/       # 预检脚本

external_plugins/
└── codex-ai/                  # Codex AI 插件（合并 codex-ai 技能 + MCP 工具）
    ├── .claude-plugin/
    │   └── plugin.json        # 插件元数据
    ├── .mcp.json              # MCP server 配置
    └── skills/
        └── codex-ai/
            ├── SKILL.md       # 技能定义
            ├── README.md      # 面向使用者的快速入门
            ├── references/    # 完整参考（api-reference.md / quick-reference.md）
            ├── examples/      # 端到端示例（review-workflow.md）
            └── scripts/       # 辅助脚本（check-codex-mcp.sh）
```

### 插件注册

插件通过 `.claude-plugin/marketplace.json` 注册（节选实际结构）：

```json
{
  "name": "ccode-plugins",
  "plugins": [
    {
      "name": "ccode-notify",
      "version": "1.0.0",
      "source": "./plugins/ccode-notify",
      "category": "tools"
    },
    {
      "name": "git-commit",
      "version": "1.0.0",
      "source": "./plugins/git-commit",
      "strict": false,
      "skills": [
        "./skills/git-commit"
      ]
    },
    {
      "name": "codex-ai",
      "version": "1.0.0",
      "source": "./external_plugins/codex-ai",
      "category": "tools",
      "strict": false,
      "skills": [
        "./skills/codex-ai"
      ]
    }
  ]
}
```

注意：`git-commit` 与 `codex-ai` 均使用 `skills` 字段显式列出子技能路径，每个插件职责单一、可独立版本管理。

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

### Skills 插件：git-commit

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

### Skills 插件：codex-ai

**用途**：通过 MCP 工具调用 Codex 处理复杂技术任务（不再使用 Bash 直接调用 `codex` CLI）。

**触发场景**：
- 代码审查（review、code review）
- 复杂算法设计（>10行核心逻辑）
- 架构分析与评审（系统扩展、架构重构）
- 性能优化（瓶颈分析、性能调优）

**MCP 工具调用**（由 `external_plugins/codex-ai` 提供，合并了原 codex-ai 技能与 codex-mcp-tool MCP 工具）：
- `mcp__plugin_codex-ai_codex-ai__codex`：启动新会话，返回 `threadId`（或旧字段 `session_id`）
- `mcp__plugin_codex-ai_codex-ai__codex-reply`：携带 `threadId` 续聊

**调用示例**（开启新会话）：
```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {
    "model": "gpt-5.3-codex",
    "sandbox": "workspace-write",
    "approval-policy": "on-failure",
    "prompt": "review the uncommitted diff for correctness and style"
  }
}
```

**模型选择**：
- **gpt-5.3-codex**：简单任务（代码审查、简单重构、文档生成）
- **gpt-5.4**：复杂任务（复杂算法、架构评审、性能优化、多约束问题）
- 通过 `config` 字段可进一步调整推理强度，如 `{"model_reasoning_effort": "xhigh"}`

### 配置系统架构（v0.3.0）

- 运行时配置：`~/.config/ccode/config.toml`（同级 `~/.config/ccode/.env` 保存密钥）。
- 迁移策略：
  - 自动迁移：存在 `config.json` 且不存在 `config.toml` 时，启动任意 `ccode` 命令都会触发 `migrate::auto_migrate_if_needed()`，迁移并备份，成功后移除 `config.json`。
  - 备份路径：`~/.config/ccode/config.json.bak-YYYYMMDD-HHMMSS`；失败不会删除 JSON，可手动回滚。
  - JSON 不再作为运行时来源，仅用于迁移；旧版 `ccode config merge` 子命令已废弃。

## 命令组织模式

### 顶层入口

| 调用形式 | 行为 |
| --- | --- |
| `ccode` | 使用默认 profile 直接启动 `claude`（无参） |
| `ccode <claude_args...>` | 使用默认 profile 启动 `claude`，尾随参数透传 |
| `ccode profile <子命令>` | 进入 profile 管理命名空间（由 clap 解析） |
| `ccode --help` / `-h` | 先打印 ccode 简介，再透传 `--help` 给 `claude` |
| `ccode --version` / `-v` / `-V` | 先打印 `ccode vX.Y.Z`，再透传给 `claude` |

### `ccode profile` 子命令一览（仅 Direct）

- `ccode profile list` - 列出所有可用配置
- `ccode profile add <name>` - 交互式添加新配置
- `ccode profile use <name>` - 设置默认配置
- `ccode profile remove <name>` - 删除配置
- `ccode profile run [name] [--tmux-env auto|always|never] [-- <claude_args>...]` - 启动并透传参数到 `claude`
- `ccode profile clear-env` - 清理 tmux 中 ccode 相关环境变量

### 废弃命令兼容提示

以下旧的顶层命令已废弃，执行时会打印迁移提示并以退出码 1 结束（来源：`src/main.rs` 中 `DEPRECATED_COMMANDS`）：

| 旧命令 | 新写法 |
| --- | --- |
| `ccode list` / `add` / `use` / `remove` / `run` | `ccode profile <同名子命令>` |
| `ccode tmux ...` | `ccode profile clear-env` |
| `ccode config ...` | 已取消，JSON→TOML 迁移在每次启动时自动执行 |

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
- **`CLAUDE_CODE_SUBAGENT_MODEL`**: 子代理覆盖模型（对应 `model_subagent`）
- `ANTHROPIC_SMALL_FAST_MODEL`（已弃用）：为兼容，程序在 `model_haiku` 存在时仍会同时设置该变量。

兼容性说明：当 `model_haiku` 存在时，运行时会同时注入 `ANTHROPIC_DEFAULT_HAIKU_MODEL` 与 `ANTHROPIC_SMALL_FAST_MODEL` 两个变量。

### 💡 使用场景

#### 基础配置（仅必需变量）
```bash
ccode profile add basic-api
# 输入Token和URL，跳过可选字段
# 使用claude的默认模型选择策略
```

#### 精确模型控制（包含可选变量）
```bash
ccode profile add precise-api
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
`ccode` 支持把尾随参数透传给 `claude` 命令。两种入口均支持：

1. **顶层 `ccode <claude_args>`**：使用默认 profile，直接透传（最省事的日常用法）
2. **`ccode profile run [name] -- <claude_args>`**：指定 profile 并透传（需要切 profile 或传递与 ccode 冲突的参数时使用）

### 使用方式

```bash
# 顶层透传（使用默认 profile）
ccode --version                                   # 先打印 ccode 版本再透传
ccode code --project myapp                        # 启动 claude 并打开项目

# 通过 profile run 指定配置
ccode profile run myapi                           # 仅切换 profile，不传参数
ccode profile run myapi -- --help                 # 透传 --help 给 claude
ccode profile run myapi -- code --project myapp   # 透传多个参数
```

说明：
- `-- ` 分隔符用于避开 `ccode profile run` 自身的参数（例如 `--tmux-env`）冲突。
- `ccode profile run --help` 会显示 `run` 子命令的帮助，不是 claude 的帮助；若要看 claude 的帮助请写 `ccode -- --help` 或 `ccode profile run <name> -- --help`。

### 功能特性
- **两种入口**：顶层 ccode 透传 + `profile run -- ...`
- **智能冲突处理**：参数与 ccode 冲突时提示使用 `--` 分隔符
- **完整透传**：支持所有 `claude` 命令的参数和选项

### 实现原理
1. `ProfileCommands::Run` 使用 `trailing_var_arg = true` 与 `allow_hyphen_values = true` 收集 `claude_args`
2. 顶层 `ccode` 无参数时调用 `cmd_run(None, TmuxEnvMode::Auto, vec![], true)`；有参数且非 `profile` 时，整段参数作为 `claude_args` 透传
3. 冲突检测：当参数与 `profile run` 自身参数冲突时，提示改用 `--` 分隔符
4. 在启动 `claude` 时将收集的参数追加到命令行

## tmux / Team 模式兼容

- 背景：Claude Code 在 `--tmux`/Team 工作流中，新 pane/window 会从 tmux 会话环境继承变量；若 tmux `update-environment` 未包含 `ANTHROPIC_*`，后续实例可能丢失密钥与 URL。
- 方案：`ccode profile run` 在 `--tmux-env=auto`（默认）下，检测到 `--tmux`/`--worktree` 或当前处于 tmux 会话时，临时补齐 tmux `update-environment`，并在命令结束后自动恢复原值。
- 清理：`ccode profile clear-env` 可手动清除 tmux 会话中的相关环境变量。

## 环境变量冲突检测

`ccode` 在启动 `claude` 前会自动检测环境变量冲突，确保配置正确生效。

### 检测范围

- **进程环境变量**：shell 或系统设置的环境变量
- **settings.json**：`~/.claude/settings.json` 中的 `env` 节点
- **自定义配置文件**：通过 `--settings` 参数指定的配置文件

### 检测的环境变量

- `ANTHROPIC_AUTH_TOKEN`
- `ANTHROPIC_BASE_URL`
- `ANTHROPIC_MODEL`
- `ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `ANTHROPIC_SMALL_FAST_MODEL`
- `ANTHROPIC_DEFAULT_SONNET_MODEL`
- `ANTHROPIC_DEFAULT_OPUS_MODEL`
- `CLAUDE_CODE_MAX_OUTPUT_TOKENS`
- `CLAUDE_CODE_SUBAGENT_MODEL`

### 冲突警告示例

```
⚠️  检测到环境变量冲突：
   以下环境变量可能会覆盖 ccode 的配置：

   📌 进程环境变量：
   - ANTHROPIC_AUTH_TOKEN=ut-46ff9***
   - ANTHROPIC_BASE_URL=https://ai.uniontech.com/api/anthropic

   📄 settings.json (/tmp/test_settings.json):
   - CLAUDE_CODE_MAX_OUTPUT_TOKENS=99999

   💡 建议解决方案：
      1. 进程环境变量：使用 'unset' 命令清除（如：unset ANTHROPIC_AUTH_TOKEN）
      2. settings.json：编辑文件移除上述环境变量配置
      3. ccode 将继续执行，但上述变量可能会覆盖配置
```

### 使用示例

```bash
# 使用默认配置检测冲突
ccode profile run uniontech

# 使用自定义 settings.json 并检测冲突
ccode profile run uniontech -- --settings /path/to/settings.json

# 静默模式（不显示冲突警告）
ccode profile run --quiet uniontech
```

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
├── SKILL.md              # 技能定义（必需，作为精简入口）
├── README.md             # 面向使用者的快速入门
├── references/           # 完整技术参考，按主题拆分（按需加载）
├── examples/             # 端到端示例工作流（按需加载）
└── scripts/              # 辅助脚本（诊断/预检等，按需调用）
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
- **SKILL.md**：Claude Code 读取的核心定义（精简，作为入口）
- **README.md**：用户快速入门指南
- **references/**：完整技术参考，按主题拆分为多个 Markdown（渐进披露）
- **examples/**：端到端示例工作流
- **scripts/**：辅助脚本（诊断、预检等）

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
- **自动迁移（唯一路径）**：每次启动 `ccode` 时 `auto_migrate_if_needed()` 检测；存在 `config.json` 且无 `config.toml` 时触发
- **备份机制**：`config.json.bak-YYYYMMDD-HHMMSS`
- **失败保护**：任一步失败都不删除原 JSON，可手动回滚
- **废弃**：旧的 `ccode config merge` 子命令已移除（`DEPRECATED_COMMANDS` 中对 `config` 给出提示）

### 2. Hooks 注入模式（ccode-notify）
- 通过 `hooks.json` 注册事件监听
- 使用 `${CLAUDE_PLUGIN_ROOT}` 动态路径
- 脚本失败不中断主流程（以 0 退出）
- 参考 `explanatory-output-style` 的设计思路

### 3. Skills 定义模式（git-commit / codex-ai）
- **SKILL.md**：Claude Code 读取的技能定义（保持精简）
- **README.md**：用户快速入门
- **references/** / **examples/** / **scripts/**：按需加载的完整参考、示例与脚本（渐进披露）
- **YAML Front Matter**：`name`、`description`，必要时附 `allowed-tools`
- 每个技能作为独立插件管理，职责单一、可独立版本控制

### 4. 命名空间隔离（Memory）
- 格式：`project:<repo>:<category>:<identifier>`
- 使用 kebab-case
- 避免跨项目污染
- 示例：`project:ccode:commit-convention`

## 迁移指引（摘要）

- 启动任意 `ccode` 命令时 `migrate::auto_migrate_if_needed()` 自动检测；仅在存在 `config.json` 且缺少 `config.toml` 时触发。
- 成功后：写入 `config.toml` + 同级 `.env`，原 `config.json` 被重命名为 `config.json.bak-YYYYMMDD-HHMMSS`。
- 失败时：保留原 JSON 不动，打印 `⚠️ 自动迁移失败: ...` 以便用户人工介入。
- 旧的 `ccode config merge` 已废弃，无需也不能手动触发。
