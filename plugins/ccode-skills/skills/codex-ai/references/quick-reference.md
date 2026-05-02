# Codex-AI 快速参考

常用 MCP 调用模版与速查表。任务执行一律走 MCP 工具；CLI 命令只用于本地环境诊断。

> 说明：本文中出现的 `model` 值（如 `gpt-5.3-codex` / `gpt-5.4`）仅为示例，请按当前 Codex 实际可用版本替换；Codex MCP server 以实际 `config.toml` 的 `model` 字段为准。

## MCP 工具调用速查

### 发起新会话

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "<任务描述>",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "xhigh"},
    "approval-policy": "never",
    "sandbox": "workspace-write"
  }
}
```

### 继续会话（多轮对话）

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "<上一次返回的 threadId>",
    "prompt": "<后续问题>"
  }
}
```

> 旧版字段 `session_id` 已被 `threadId` 取代，仍保留向后兼容；新代码统一用 `threadId`。

## 按场景选模板

### 场景 A · 代码审查（review / code review / 审查代码）

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "审查以下代码变更，聚焦潜在问题、性能与安全：\n\n<git diff 输出>",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "xhigh"},
    "sandbox": "read-only"
  }
}
```

要点：
- 审查场景优先 `sandbox: "read-only"`，避免 Codex 改动文件。
- 先用 Bash 获取 `git diff` 或 `git diff --cached`，再把内容粘贴进 `prompt`。

### 场景 B · 简单任务（简单重构 / 单一目标算法 / 文档生成）

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "<任务描述>",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "high"}
  }
}
```

### 场景 C · 复杂任务（复杂算法 / 架构评审 / 性能优化 / 多约束权衡）

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "<任务描述>",
    "model": "gpt-5.4",
    "config": {"model_reasoning_effort": "xhigh"},
    "sandbox": "workspace-write"
  }
}
```

## 模型选择快速决策

| 场景 | 推荐模型 | 特点 |
|------|---------|------|
| 代码审查、简单重构、单一目标算法 | **gpt-5.3-codex** | 响应快、质量稳、成本低 |
| 复杂算法设计、架构评审、性能优化、多约束权衡 | **gpt-5.4** | 深度推理、处理多维约束 |

**快速判断规则**：
- 核心逻辑 < 10 行且目标单一 → `gpt-5.3-codex`
- 涉及系统级设计或 3+ 约束权衡 → `gpt-5.4`
- 不确定时优先 `gpt-5.4`

## 关键参数速查

| 参数 | 说明 | 常用值 |
|------|------|--------|
| `prompt` | 任务描述（必填） | 结构化文本 |
| `model` | 模型名 | `gpt-5.3-codex` / `gpt-5.4` |
| `config.model_reasoning_effort` | 推理强度 | `xhigh`（推荐）/ `high` / `medium` |
| `approval-policy` | shell 命令审批 | `never`（自动化）/ `on-failure` / `on-request` / `untrusted` |
| `sandbox` | 文件系统权限 | `read-only`（审查）/ `workspace-write`（开发）/ `danger-full-access`（谨慎） |
| `cwd` | 工作目录 | 绝对路径，通常省略 |

## 常见问题速查

| 现象 | 诊断思路 | 解决方向 |
|------|---------|---------|
| MCP 工具找不到 | `codex-mcp-tool` 插件未加载 | 检查 `.claude-plugin/marketplace.json` 和 `external_plugins/codex-mcp-tool` |
| MCP 调用返回底层错误 | Codex CLI 未安装或未登录 | 运行 `scripts/check-codex-mcp.sh` 诊断 |
| 响应超时 | 任务过大、`xhigh` 推理过重 | 拆分任务；临时降到 `high` |
| 输出过于泛泛 | prompt 未给约束 | 补齐指标、边界、期望格式 |
| 模型选择不当 | 简单任务用了 `gpt-5.4`，或相反 | 用 `codex-reply` 切会话并调整模型 |

## 本地环境诊断（仅 CLI）

MCP 工具内部通过 Codex CLI 工作。若 MCP 调用频繁失败，可临时用 CLI 确认本地环境健康：

```bash
# 1) 检查 Codex CLI 是否就位
codex --version

# 2) 查看当前配置
codex config show

# 3) 跑一个最小任务，确认能通
codex exec -m gpt-5.3-codex "return the string 'ok'"
```

也可直接执行技能自带脚本：

```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/codex-ai/scripts/check-codex-mcp.sh"
```

> 上述 CLI 命令仅用于诊断。任务执行一律通过 MCP 工具调用，避免在技能流程里直接执行 `codex exec`。

## 使用场景速查

| 场景 | 触发关键词 | 模型 | 建议 sandbox |
|------|----------|------|-------------|
| 代码审查 | review, code review, 审查代码 | gpt-5.3-codex | read-only |
| 简单重构 | 重构, refactor | gpt-5.3-codex | workspace-write |
| 算法设计 | 算法, 数据结构 | gpt-5.3-codex / gpt-5.4 | workspace-write |
| 架构分析 | 架构评审, 系统设计, 扩展性 | gpt-5.4 | workspace-write |
| 性能优化 | 瓶颈, 性能调优, p99 | gpt-5.4 | workspace-write |

---

**关联文档**：
- 技能定义：[../SKILL.md](../SKILL.md)
- 用户入口：[../README.md](../README.md)
- 完整技术参考：[api-reference.md](api-reference.md)
- 完整示例：[../examples/review-workflow.md](../examples/review-workflow.md)
