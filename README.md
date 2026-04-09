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
default = "minimax-m2"

[profiles.minimax-m2]
name = "minimax-m2"
base_url = "https://api.anthropic.com"
env_key  = "minimax-m2_key"       # 对应配置目录下 .env 中的变量名
model = "claude-3-5-sonnet-latest"
model_sonnet = "claude-3-5-sonnet-20241022"
model_haiku  = "claude-3-haiku-20240307"
model_opus   = "claude-3-opus-latest"
max_tokens   = "32000"
comment = "测试站点"

[profiles.anyrouter]
name = "anyrouter"
base_url = "https://api.example.com"
env_key  = "anyrouter_key"
```

同级 `.env` 示例（可选，优先于系统环境加载）：
```env
# 文件路径：~/.config/ccode/.env
minimax-m2_key="sk-xxx..."
anyrouter_key="sk-yyy..."
```

#### 使用 TOML 配置
```bash
# 列出配置
ccode profile list

# 运行（未指定 name 时使用 default）
ccode                        # 无参数，使用默认 profile
ccode <claude_args>          # 直接透传参数到 claude
ccode run minimax-m2         # 指定 profile
ccode profile run minimax-m2 # 等同于上方

# 透传参数
ccode --version              # 显示 ccode 版本（quiet 模式）
ccode run minimax-m2 --version
ccode run minimax-m2 -- --help

# tmux 环境同步策略（team/--tmux 场景）
ccode run minimax-m2 --tmux-env auto -- --worktree my-branch --tmux
```

#### 添加配置（交互式）
```bash
ccode profile add myapi
# 按提示依次输入：
# 1) ANTHROPIC_BASE_URL（如 https://api.anthropic.com）
# 2) ANTHROPIC_AUTH_TOKEN（密钥值，工具会保存到配置目录 .env）
```

#### 其他 profile 管理命令
```bash
ccode profile use <name>        # 设置默认配置
ccode profile remove <name>     # 删除配置
ccode profile clear-env         # 清理 tmux 中 ccode 相关环境变量
```

<!-- Router/Provider 功能已移除 -->

## 🔁 迁移指南（config.json → config.toml）

自 v0.2.0 起，配置文件统一迁移为 `~/.config/ccode/config.toml`，密钥改为从同级 `~/.config/ccode/.env` 读取。

### 自动迁移（开箱即用）
- 触发条件：存在 `~/.config/ccode/config.json` 且不存在 `config.toml`。
- 行为：运行任意 `ccode` 命令时自动迁移；迁移成功后删除 `config.json`，并生成备份。
- 备份文件：`~/.config/ccode/config.json.bak-YYYYMMDD-HHMMSS`
- 失败保护：任一步失败不会删除 `config.json`，备份仍保留。

### 手动迁移/合并（推荐命令）

当同时存在 `config.toml` 与 `config.json` 时，不会自动处理，建议使用合并命令：

```bash
# 将旧版 JSON 合并/迁移到 TOML，并在成功后移除 JSON
ccode config merge
```

合并策略与输出说明：
- 同名 profile 跳过（保留现有 TOML 的版本），新增 profile 自动写入。
- 默认 profile：若 TOML 未设置且 JSON 有默认，则迁移默认值。
- 输出示例（简化）：
```
✅ 迁移完成：共 3，迁移 2，跳过 1
🆕 已创建 config.toml           # 或 🔗 已合并到现有 config.toml
🎯 默认配置: demo
🗄️  已备份旧 JSON: /home/user/.config/ccode/config.json.bak-20251029-114233
🧹 已移除 config.json
```

### 字段与密钥迁移规则
- JSON 字段映射到 TOML：
  - `ANTHROPIC_BASE_URL` → `base_url`
  - `ANTHROPIC_MODEL` → `model`
  - `ANTHROPIC_SMALL_FAST_MODEL`（弃用）→ `model_haiku`
  - 其他家族模型、最大输出等见上文“字段映射”。
- 密钥迁移：
  - `ANTHROPIC_AUTH_TOKEN` 的值迁移到同级 `.env`，变量名由 profile 名推导：`{profile}_key`（非字母数字改为下划线、小写，并确保以 `_key` 结尾）。
  - TOML 中仅保存 `env_key`，运行时从 `.env` 或系统环境读取。

`.env` 迁移后示例：
```env
# 文件：~/.config/ccode/.env
myapi_key="sk-xxx..."
anyrouter_key="sk-yyy..."
```

### 还原与回滚
- 从备份还原：
```bash
mv ~/.config/ccode/config.json.bak-YYYYMMDD-HHMMSS ~/.config/ccode/config.json
```
- 如需完全回滚到 JSON 流程，可临时移除 `config.toml`；但不建议长期使用旧格式。

### 迁移后自检
```bash
ccode profile list           # 检查 profile 与默认项
cat ~/.config/ccode/config.toml | sed -n '1,120p'
cat ~/.config/ccode/.env | sed -n '1,80p'
ccode run demo --version
```

### ⚠️ 命令结构变更说明（v0.5.0）

v0.5.0 引入了新的命令结构以提升用户体验。旧的扁平式命令已升级为分层式：

| 旧命令（v0.4.x） | 新命令（v0.5.0） | 说明 |
|------------------|------------------|------|
| `ccode list` | `ccode profile list` | 列出配置 |
| `ccode add <name>` | `ccode profile add <name>` | 添加配置 |
| `ccode use <name>` | `ccode profile use <name>` | 设置默认配置 |
| `ccode remove <name>` | `ccode profile remove <name>` | 删除配置 |
| `ccode run` | `ccode` 或 `ccode profile run` | 无参数直接启动 |
| `ccode tmux clear-env` | `ccode profile clear-env` | 清理 tmux 环境 |
| `ccode config merge` | - | 手动迁移功能已移除 |

**重要提示**：
- 使用旧命令时会显示友好的迁移提示
- 直接使用 `ccode` 即可启动默认 profile（推荐新用法）
- `ccode` 后直接跟参数会透传给 claude（如 `ccode --version`）

### 手动迁移（已移除）

自 v0.5.0 起，`ccode config merge` 手动迁移功能已移除。如需从旧版 JSON 迁移：
- 保留旧版用户可使用 v0.4.x 版本手动迁移后再升级
- 新用户建议直接使用 TOML 格式创建配置

## 📋 命令参考

### 🔄 命令

```bash
# 配置管理（profile 子命令）
ccode profile list                    # 列出所有配置
ccode profile add <name>              # 添加新配置（交互式）
ccode profile use <name>              # 设置默认配置
ccode profile remove <name>           # 删除配置

# 启动 claude
ccode                                 # 无参数，使用默认 profile（quiet 模式）
ccode <claude_args>                   # 直接启动并透传参数
ccode profile run [name]              # 明确指定 profile 启动
ccode profile run [name] --tmux-env <auto|always|never>  # 指定 tmux 同步策略

# 示例：
ccode profile run myapi --version                      # 直接透传
ccode profile run myapi code                           # 透传子命令
ccode profile run myapi -- --help                      # 使用 -- 分隔符避免冲突
ccode profile run myapi --tmux-env always -- --worktree feat-a --tmux

# tmux 环境变量清理
ccode profile clear-env               # 清理 tmux 中 ccode 相关环境变量
```

### 📋 输出字段说明（v0.5.0）
- `base_url`：目标 API 基础地址
- `env_key`：从同级 `~/.config/ccode/.env` 或系统环境读取的变量名
- `model` / `model_haiku` / `model_sonnet` / `model_opus`：模型或家族模型
- `max_tokens`：最大输出 tokens（可选）
- `说明`：对该配置的人类可读备注

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
3. **检测环境变量冲突**（重要）：
   - 检查进程环境变量中的 `ANTHROPIC_*` 和 `CLAUDE_CODE_*` 变量
   - 检查 `~/.claude/settings.json` 中的 `env` 节点
   - 支持 `--settings` 参数指定的自定义配置文件
   - 发现冲突时显示警告，建议用户清理
4. 设置环境变量：
   - `ANTHROPIC_AUTH_TOKEN` ← 由 `env_key` 指向的值
   - `ANTHROPIC_BASE_URL` ← `base_url`
   - `ANTHROPIC_MODEL` ← `model`（可选）
   - `ANTHROPIC_DEFAULT_HAIKU_MODEL` ← `model_haiku`（可选）
   - `ANTHROPIC_SMALL_FAST_MODEL` ← `model_haiku`（兼容性：已弃用但仍设置）
   - `ANTHROPIC_DEFAULT_SONNET_MODEL` ← `model_sonnet`（可选）
   - `ANTHROPIC_DEFAULT_OPUS_MODEL` ← `model_opus`（可选）
   - `CLAUDE_CODE_MAX_OUTPUT_TOKENS` ← `max_tokens`（可选）
5. 透传参数并启动 `claude`

### 环境变量冲突检测

`ccode` 会在启动 `claude` 前自动检测环境变量冲突，确保您的配置正确生效。

**检测范围：**
- 进程环境变量（shell 或系统设置的变量）
- `~/.claude/settings.json` 中的 `env` 节点
- 通过 `--settings` 参数指定的自定义配置文件

**检测的环境变量：**
- `ANTHROPIC_AUTH_TOKEN`
- `ANTHROPIC_BASE_URL`
- `ANTHROPIC_MODEL`
- `ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `ANTHROPIC_SMALL_FAST_MODEL`
- `ANTHROPIC_DEFAULT_SONNET_MODEL`
- `ANTHROPIC_DEFAULT_OPUS_MODEL`
- `CLAUDE_CODE_MAX_OUTPUT_TOKENS`

**冲突警告示例：**
```
⚠️  检测到环境变量冲突：
   以下环境变量可能会覆盖 ccode 的配置：

   📌 进程环境变量：
   - ANTHROPIC_AUTH_TOKEN=sk-ant-t***
   - ANTHROPIC_BASE_URL=https://ai.uniontech.com/api/anthropic

   📄 settings.json (/tmp/test_settings.json):
   - CLAUDE_CODE_MAX_OUTPUT_TOKENS=99999

   💡 建议解决方案：
      1. 进程环境变量：使用 'unset' 命令清除（如：unset ANTHROPIC_AUTH_TOKEN）
      2. settings.json：编辑文件移除上述环境变量配置
      3. ccode 将继续执行，但上述变量可能会覆盖配置
```

**使用方式：**
```bash
# 正常启动，自动检测冲突
ccode profile run uniontech

# 使用自定义 settings.json
ccode profile run uniontech -- --settings /path/to/settings.json

# 静默模式（不显示冲突警告）
ccode profile --quiet run uniontech
```

### tmux / Team 模式环境变量同步

当 `claude` 使用 `--tmux`（通常伴随 `--worktree`）时，新 pane/window 继承的是 tmux 会话环境。
`ccode run` 新增 `--tmux-env` 参数用于控制同步策略：

- `auto`（默认）：检测到 `--tmux` / `--worktree` 参数，或当前就在 tmux 会话中时，临时补齐 tmux `update-environment`。
- `always`：每次 `run` 都尝试进行 tmux 同步处理。
- `never`：禁用 tmux 同步处理。

示例：

```bash
ccode run myapi --tmux-env auto -- --worktree team-a --tmux
ccode run myapi --tmux-env never -- --worktree team-a --tmux
```

安全说明：
- `ccode` 不会把 token 值作为 tmux 命令参数传递；
- tmux 会话环境仍可能保留变量名对应的值，必要时执行 `ccode tmux clear-env` 清理。

<!-- Router 模式工作原理已移除 -->

<!-- 架构图（CCR 集成）已移除 -->

## 📝 版本变更

### v0.5.0（2026-04-09）
- **变更（破坏性）**：重构命令结构，引入 `profile` 子命令统一管理配置。
- **废弃命令**：`list`/`add`/`use`/`remove`/`config`/`tmux` 已废弃，使用时会显示友好提示。
- **新增**：无参数直接启动（`ccode`）、quiet 模式（减少冗余输出）、废弃命令检测与提示。
- **新增**：环境变量冲突检测功能，支持进程环境和 `settings.json` 检测，避免配置不生效。
- **优化**：移除 `--group` 参数和手动迁移功能、标记死代码、升级主要依赖、简化命令实现。
- **文档**：更新命令参考、版本变更说明，新增环境变量冲突检测章节。

### v0.4.0（2026-03-01）
- 新增：`ccode run --tmux-env <auto|always|never>`，修复 `claude --tmux`/Team 场景后续实例可能丢失 `ANTHROPIC_*` 环境变量的问题。
- 新增：`ccode tmux clear-env`，用于手动清理 tmux 会话中的相关环境变量。
- 行为：默认 `auto` 策略，仅在 tmux/worktree 相关场景触发环境同步。

### v0.3.0（2025-10-29）
- 新增：自动迁移功能（存在 `config.json` 且无 `config.toml` 时自动迁移并备份）。
- 新增：`ccode config merge` 主动迁移/合并旧版 JSON；成功后移除 `config.json`，保留备份。
- 兼容：当 `model_haiku` 已设置时，同时注入环境变量 `ANTHROPIC_DEFAULT_HAIKU_MODEL` 与 `ANTHROPIC_SMALL_FAST_MODEL`（后者为兼容旧变量）。
- 文档：新增“迁移指南”，更新“工作原理/环境变量映射”。

### v0.2.0（2025-08-10）
- 变更：配置改为 `~/.config/ccode/config.toml` + `.env` 的新架构。
- 功能：`list/add/use/remove/run` 全量基于 TOML；支持参数透传与 `--` 分隔。

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

**最后更新**: 2026-04-09 | **架构版本**: v0.5.0（配置管理工具）
[必填项说明]
- 每个 profile 必填：`name`、`base_url`、`env_key`
- 其他字段（`model*`、`max_tokens`、`comment`）均为可选
