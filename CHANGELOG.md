# Changelog

本项目遵循 Keep a Changelog 的书写方式，并尽量遵循 语义化版本号（SemVer）。

## [v0.4.1] - 2026-03-01

重要改动
- 新增
  - `ccode run` 新增参数 `--tmux-env <auto|always|never>`（默认 `auto`），用于 Claude Code Team/`--tmux` 场景下的环境变量同步策略控制。
  - 新增命令 `ccode tmux clear-env`，用于手动清理 tmux 会话中的 `ANTHROPIC_*` 与 `CLAUDE_CODE_MAX_OUTPUT_TOKENS` 相关环境变量。

- 修复
  - 修复在 tmux server 已存在时，后续 `claude` 实例可能丢失 `ANTHROPIC_AUTH_TOKEN` / `ANTHROPIC_BASE_URL` 等变量的问题。
  - `run` 在触发 tmux 策略时会临时合并 `update-environment`，确保新 pane/window 能继承所需变量，并在 `ccode run` 结束后恢复原配置。

## [v0.3.0] - 2025-10-29

重要改动
- 新增
  - 自动迁移：当存在 `~/.config/ccode/config.json` 且不存在 `config.toml` 时，运行任意 `ccode` 命令将自动迁移到 TOML，并在成功后删除 JSON；同时生成备份 `config.json.bak-YYYYMMDD-HHMMSS`。
  - 手动合并：新增命令 `ccode config merge`，当 `config.toml` 与 `config.json` 并存时，将 JSON 合并进 TOML（同名 profile 跳过），成功后删除 JSON。
  - 兼容性：当配置项 `model_haiku` 存在时，运行时同时注入环境变量 `ANTHROPIC_DEFAULT_HAIKU_MODEL` 与 `ANTHROPIC_SMALL_FAST_MODEL`（后者为兼容旧变量）。
  - 文档：README 新增“迁移指南（config.json → config.toml）”；更新 CLAUDE.md（v0.3.0 架构、迁移策略、命令）；AGENTS.md 升级为 1.1（新增迁移策略/命令与兼容性说明）。
  - 记忆：新增 `docs/memory/ccode-memory.json` 作为 Memory 种子，沉淀项目关键约定与关系。

- 变更
  - 运行时配置统一为 `~/.config/ccode/config.toml` + 同级 `~/.config/ccode/.env`；JSON 不再作为运行时来源，仅用于迁移。
  - 程序启动时进行自动迁移检测；若 TOML 与 JSON 并存，仅提示手动处理（不自动覆盖）。
  - README 版本与日期更新为 v0.3.0（2025-10-29）；新增“版本变更”章节。

- 弃用
  - `ANTHROPIC_SMALL_FAST_MODEL` 环境变量已弃用；但在设置了 `model_haiku` 时仍会同时设置该变量以保持向后兼容。

- 修复/质量
  - 通过 `cargo fmt`、`cargo clippy -D warnings` 与 `cargo test` 校验；修复 `src/migrate.rs` 的 clippy 告警（无用的 `mut`、默认后再赋值、无用 `format!`、未读取的赋值等）。

- 安全
  - 令牌仅从环境读取，落盘至 `~/.config/ccode/.env`（由 `env_key` 决定）；日志与错误信息最小化并脱敏。

迁移说明
- JSON → TOML 自动迁移：成功后移除 JSON，并在同目录生成备份；如需回滚，将备份重命名为 `config.json` 即可。
- 手动合并：`ccode config merge`；同名 profile 跳过，新增 profile 写入 TOML；若 TOML 未设置默认而 JSON 设置了默认，则迁移默认值。

## [v0.2.0] - 2025-08-10

- 变更
  - 引入 `~/.config/ccode/config.toml` + `.env` 新架构；`list/add/use/remove/run` 全量基于 TOML。
  - 支持参数透传与 `--` 分隔；环境变量映射至 `ANTHROPIC_*` 与 `CLAUDE_CODE_MAX_OUTPUT_TOKENS`。

- 移除
  - Router/Provider 相关功能从用户文档中移除（代码保留少量兼容与注释片段，不作为对外能力）。

——

提示
- 若 `Cargo.toml` 版本号未同步为 0.3.0，请在发布前更新以保持与文档一致。
