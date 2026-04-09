# AGENTS.md - ccode 项目工作指引（项目版）
版本: 1.2 | 更新日期: 2026-04-09

作用范围：本文件适用于本仓库的全部目录。若子目录存在更深层 AGENTS.md，则以子目录版本为准。
关联文档：实现细节与示例请参阅 CLAUDE.md。

## 核心配置
- 角色定位：技术架构师 / 全栈专家 / 技术导师 / 技术伙伴。
- 中文优先：回答、注释与文档统一中文表述，术语遵循本项目约定。
- 质量门槛（提交前必须全部通过）：cargo fmt；cargo clippy -- -D warnings；cargo test；cargo audit。

## MCP 工具规则（本仓库约定）
- Sequential Thinking：复杂修改/重构/发布相关变更需 6–8 步，可执行、可验证；必要时更新计划。
- Memory：以 project:ccode:* 命名沉淀“环境变量映射/弃用策略/配置路径/命令约定/关键决策”，禁止写入敏感信息。
- Context7：仅查官方文档（Rust、clap、serde 等），简要引用并标注库 ID/版本。

## 项目工作规范
- 定位与边界：ccode 管理 Claude CLI 的 Direct 模式配置与启动，不做服务管理（详见 CLAUDE.md）。
- 技术栈与版本：Rust 2024 Edition；最低 Rust 1.70+；clap 4.x、serde/serde_json、sysinfo、dirs、chrono、anyhow。
- 目录结构（参考）：src/main.rs；src/commands.rs；src/toml_config.rs；src/migrate.rs；src/config.rs（仅迁移用）；src/error.rs；src/tmux_env.rs；src/lib.rs。
- 构建与检查：开发构建 cargo build；发布构建 cargo build --release；质量检查同”质量门槛”。
- 命令（Direct，v0.5.0）：
  - `ccode profile list`：列出配置
  - `ccode profile add <name>`：添加配置（交互式）
  - `ccode profile use <name>`：设置默认配置
  - `ccode profile run [name] [--tmux-env <auto|always|never>] [<claude_args>]`：启动 claude
  - `ccode profile remove <name>`：删除配置
  - `ccode profile clear-env`：清理 tmux 环境变量
  - `ccode`：无参数启动默认 profile（quiet 模式）
  - `ccode <claude_args>`：直接启动并透传参数
- 透传策略：优先直接透传；遇到冲突使用”--”分隔；如需示例参见 CLAUDE.md。
- 配置路径（v0.5.0）：统一使用 `~/.config/ccode/config.toml`；同级 `~/.config/ccode/.env` 保存密钥。JSON 仅用于迁移，不再作为运行时来源。
- 迁移策略（v0.3.0，v0.5.0 已移除手动合并）：
  - 自动：存在 `config.json` 且缺少 `config.toml` 时自动迁移并备份，成功后移除 JSON；
  - v0.5.0 起：移除手动合并命令 `ccode config merge`，依赖自动迁移；
  - 备份：`~/.config/ccode/config.json.bak-YYYYMMDD-HHMMSS`；失败不删除 JSON，可回滚。
- 环境变量（必需）：ANTHROPIC_AUTH_TOKEN；ANTHROPIC_BASE_URL。
- 环境变量（可选）：ANTHROPIC_MODEL；ANTHROPIC_DEFAULT_HAIKU_MODEL；ANTHROPIC_DEFAULT_SONNET_MODEL；ANTHROPIC_DEFAULT_OPUS_MODEL；CLAUDE_CODE_MAX_OUTPUT_TOKENS。
- 兼容性：当 `model_haiku` 存在时，运行时同时设置 `ANTHROPIC_DEFAULT_HAIKU_MODEL` 与 `ANTHROPIC_SMALL_FAST_MODEL`（后者为兼容旧变量）。

## 安全与错误处理
- 令牌仅从环境读取，不落盘、不入库、不写日志；输出与错误信息需最小化并脱敏。
- 错误返回：anyhow::Result<T>（可别名 AppResult<T>）；集中于 AppError 进行统一分类与处理。

## 最佳实践
- DRY / 单一职责 / 抽象化：统一可选输入读取与显示逻辑，减少重复代码并完善测试覆盖。
- 新增字段流程：更新数据结构与默认值；接入统一读取/显示；补充序列化/反序列化与边界测试；同步更新文档与示例。

## 快速参考
- 构建：cargo build；发布：cargo build --release。
- 质量门槛：cargo fmt；cargo clippy -- -D warnings；cargo test；cargo audit。
- 安装/运行：cargo install --path .；cargo run -- <subcommand>。
- 命令：ccode profile <list|add|use|remove|run|clear-env>；或直接 `ccode` 启动。

## 变更记录
- 1.2（2026-04-09）：对齐 v0.5.0，重构命令结构引入 profile 子命令；新增无参数启动、quiet 模式、废弃命令检测；移除 --group 参数和手动迁移功能；标记死代码；升级主要依赖。
- 1.1（2025-10-29）：对齐 v0.3.0，新增迁移策略（自动/手动）、新增命令 `config merge`、兼容性说明（Haiku 双变量注入）、更新目录结构与配置路径说明。
- 1.0（2025-10-29）：新增项目版 AGENTS.md；与 CLAUDE.md 对齐 Direct 模式、环境映射、质量门槛与透传策略；明确安全与错误处理约定。
