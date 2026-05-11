---
name: codex-ai
description: 当用户要求"code review / 代码审查 / review 未提交变更"，或提到"算法设计 / 复杂算法 / 分布式算法 / 并发控制"、"架构分析 / 架构评审 / 系统设计 / 扩展性分析"、"性能优化 / 瓶颈分析 / p99 / 性能调优"等复杂技术任务（核心逻辑 >10 行、多约束权衡、系统级设计）时使用本技能，通过 Codex MCP 工具调起外部模型协作分析。
allowed-tools:
  - mcp__plugin_codex-ai_codex-ai__codex
  - mcp__plugin_codex-ai_codex-ai__codex-reply
  - Bash
---

# Codex-AI 协作技能

通过 `codex-ai` 外部插件调用 Codex，处理需要深度推理的代码审查、算法设计、架构分析与性能优化任务。任务执行一律通过 MCP 工具调用，避免直接执行 `codex exec`。

> 说明：本文中出现的 `model` 值（如 `gpt-5.3-codex` / `gpt-5.4`）仅为示例，请按当前 Codex 实际可用版本替换；建议在 Codex 的 `config.toml` 固定模型，MCP 调用层仅在需要覆盖时显式传入。

## 何时使用

触发此技能的典型场景：

- **代码审查**：review、code review、审查代码、审查未提交变更
- **算法设计**：复杂算法（核心逻辑 >10 行）、数据结构设计、并发/分布式算法
- **架构分析**：架构评审、系统设计、扩展性分析、重构方案
- **性能优化**：瓶颈分析、性能调优、p99 延迟、吞吐量问题

## 何时不使用

- 简单任务（<10 行代码、基本语法）——直接生成即可
- 文档查询——使用 Context7
- 简单调试（读日志、定位空指针）——用常规工具
- 一次性的代码生成——无需深度推理

## 工作流程

1. **识别任务类型**：代码审查 / 算法设计 / 架构分析 / 性能优化
2. **选择模型**：简单任务 → `gpt-5.3-codex`；复杂任务 → `gpt-5.4`（详见 [references/api-reference.md#模型选择详解](references/api-reference.md#模型选择详解)）
3. **准备上下文**：用 Bash 收集 `git diff` / 代码片段 / 性能指标 / 约束条件
4. **调用 Codex**：通过 `mcp__plugin_codex-ai_codex-ai__codex` 发起新会话（保留返回的 `threadId`）
5. **追问细节**（可选）：用 `codex-reply` + `threadId` 继续同一会话
6. **格式化输出**：用下文标准模板把结果呈现给用户

### 输出格式模板

```
📊 分析结果
- 任务类型: <代码审查 / 算法设计 / 架构分析 / 性能优化>
- 使用模型: <gpt-5.3-codex / gpt-5.4>

📝 Codex 建议
<结构化分析，按 Codex 原输出整理>

💡 下一步行动
<对用户的可执行建议>
```

## MCP 工具调用

### 发起新会话

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {
    "prompt": "<任务描述，包含目标、约束、期望输出格式>",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "xhigh"},
    "approval-policy": "never",
    "sandbox": "read-only"
  }
}
```

参数要点：
- **代码审查**用 `sandbox: "read-only"`，避免 Codex 意外改动文件。
- **设计 / 开发**用 `sandbox: "workspace-write"`，允许 Codex 写入当前工作区。
- **`approval-policy: "never"`**：自动化场景避免审批阻塞；交互式调试可用 `on-failure`。
- **模型名**：按当前 Codex 实际可用版本替换（见文首说明），不确定时优先更强的那一档。

### 续聊

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex-reply",
  "parameters": {
    "threadId": "<上一次返回的 threadId>",
    "prompt": "<追问内容>"
  }
}
```

**完整参数与更多示例**：[references/api-reference.md#mcp-工具完整参考](references/api-reference.md#mcp-工具完整参考)

## 模型选择（速览）

- **gpt-5.3-codex**：代码审查、简单重构、单一目标算法（响应快、成本低）
- **gpt-5.4**：复杂算法、架构评审、性能优化、多约束权衡（深度推理）

**决策树与完整对照**：[references/api-reference.md#模型选择详解](references/api-reference.md#模型选择详解)

## 错误处理

| 症状 | 优先检查 | 详细方案 |
|------|---------|---------|
| 工具列表找不到 `codex-ai` | `marketplace.json` 与 `external_plugins/codex-ai` | [references/api-reference.md#诊断流程](references/api-reference.md#诊断流程) |
| MCP 调用返回底层错误 | Codex CLI 安装 / 登录 | 运行 `scripts/check-codex-mcp.sh` |
| 输出泛泛 | prompt 缺约束 | 用 `AskUserQuestion` 补齐指标、边界 |
| 响应超时 | 任务过大 / `xhigh` 过重 | 拆任务 / 临时降到 `high` |

## 附加资源

### 参考文件（references/）

按需加载，避免一次性塞满上下文：

- **[references/quick-reference.md](references/quick-reference.md)** — 常用 MCP 调用模版、模型与参数速查表
- **[references/api-reference.md](references/api-reference.md)** — 完整参数、场景示例、模型详解、诊断流程、FAQ

### 示例（examples/）

- **[examples/review-workflow.md](examples/review-workflow.md)** — 端到端代码审查工作流（git diff 采集 → 首轮 MCP → codex-reply 续聊 → 格式化输出）

### 脚本（scripts/）

- **[scripts/check-codex-mcp.sh](scripts/check-codex-mcp.sh)** — 一键诊断 Codex CLI 与 MCP server 状态

### 用户入口

- **[README.md](README.md)** — 面向使用者的快速入门
