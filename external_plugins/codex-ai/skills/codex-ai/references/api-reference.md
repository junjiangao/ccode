# Codex-AI 完整技术参考

MCP 工具调用的完整参数参考、调用示例、诊断流程与最佳实践。

> 说明：本文中出现的 `model` 值（如 `gpt-5.3-codex` / `gpt-5.4`）仅为示例，请按当前 Codex 实际可用版本替换；建议在 Codex 的 `config.toml` 固定模型，MCP 调用层仅在需要覆盖时显式传入。

## 📑 目录

- [MCP 工具完整参考](#mcp-工具完整参考)
- [完整工作流程示例](#完整工作流程示例)
- [详细场景示例](#详细场景示例)
- [模型选择详解](#模型选择详解)
- [配置选项详解](#配置选项详解)
- [诊断流程](#诊断流程)
- [错误处理指南](#错误处理指南)
- [最佳实践](#最佳实践)
- [常见问题 FAQ](#常见问题-faq)

---

## MCP 工具完整参考

Codex-AI 技能由 `codex-ai` 外部插件提供（位于 `external_plugins/codex-ai`）。该插件暴露两个 MCP 工具；任务执行**一律**通过它们调用 Codex，避免直接执行 `codex exec`。

### 工具 1：`mcp__plugin_codex-ai_codex-ai__codex`

发起新的 Codex 会话。

#### 参数详解

| 参数 | 类型 | 必需 | 说明 | 可选值 | 默认值 |
|------|------|------|------|--------|--------|
| `prompt` | string | 是 | 任务描述 | — | — |
| `model` | string | 否 | 模型选择 | `gpt-5.3-codex` / `gpt-5.4` 等 | 由 MCP server `config.toml` 决定 |
| `config` | object | 否 | 覆盖 Codex 运行时配置 | — | — |
| `config.model_reasoning_effort` | string | 否 | 推理强度 | `xhigh` / `high` / `medium` | `xhigh` |
| `approval-policy` | string | 否 | shell 命令审批策略 | `untrusted` / `on-failure` / `on-request` / `never` | `never` |
| `sandbox` | string | 否 | 沙箱模式 | `read-only` / `workspace-write` / `danger-full-access` | `workspace-write` |
| `cwd` | string | 否 | 工作目录（绝对路径） | — | 当前进程 cwd |
| `base-instructions` | string | 否 | 覆盖默认系统指令 | — | — |
| `developer-instructions` | string | 否 | 注入开发者角色消息 | — | — |
| `profile` | string | 否 | Codex 配置 profile | — | — |

> 模型名仅为示例（详见文首说明）；建议在 Codex 的 `config.toml` 固定模型，MCP 调用层只显式传需要覆盖的参数。

#### 返回结构（节选）

```json
{
  "threadId": "thread-abc123",
  "structuredContent": { "result": "...", "issues": [ ... ] }
}
```

保留 `threadId` 即可用 `codex-reply` 续聊。

#### 调用示例

##### 场景 1：代码审查（只读沙箱）

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {
    "prompt": "审查以下代码变更，关注潜在问题、性能和安全性:\n\n[git diff 输出]",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "xhigh"},
    "approval-policy": "never",
    "sandbox": "read-only"
  }
}
```

##### 场景 2：简单算法设计

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {
    "prompt": "设计一个 LRU 缓存，容量 1000 项，支持并发访问",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "xhigh"},
    "sandbox": "workspace-write"
  }
}
```

##### 场景 3：复杂算法设计

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {
    "prompt": "设计一个分布式限流算法:\n- 处理 10K req/s\n- 每用户 100 req/min\n- 延迟 <1ms\n\n要求:\n- 完整数据结构设计\n- 并发控制策略\n- 性能分析",
    "model": "gpt-5.4",
    "config": {"model_reasoning_effort": "xhigh"},
    "sandbox": "workspace-write"
  }
}
```

##### 场景 4：架构分析

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {
    "prompt": "分析当前微服务架构，从 1K 扩展到 10K req/sec:\n- 当前: API Gateway → 5 个服务 → PostgreSQL + Redis\n- 挑战: 数据库 p99 瓶颈、服务紧耦合\n- 建议改进方案",
    "model": "gpt-5.4",
    "config": {"model_reasoning_effort": "xhigh"},
    "sandbox": "read-only"
  }
}
```

##### 场景 5：性能优化

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {
    "prompt": "优化以下函数性能:\n- 当前延迟: p99 = 500ms\n- 目标: p99 < 100ms\n- 分析热点并提供优化方案\n\n[代码]",
    "model": "gpt-5.4",
    "config": {"model_reasoning_effort": "xhigh"},
    "sandbox": "workspace-write"
  }
}
```

### 工具 2：`mcp__plugin_codex-ai_codex-ai__codex-reply`

继续现有 Codex 会话。

#### 参数详解

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `threadId` | string | 是 | 上一次调用返回的线程 ID |
| `prompt` | string | 是 | 后续问题或指令 |
| `conversationId` | string | 否 | 向后兼容的旧字段，**新代码不要使用** |

#### 调用示例

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex-reply",
  "parameters": {
    "threadId": "thread-abc123",
    "prompt": "请详细解释第二个优化方案的实现细节"
  }
}
```

---

## 完整工作流程示例

> 完整的「git diff 采集 → MCP 首轮 → 续聊 → 输出」样例见 [../examples/review-workflow.md](../examples/review-workflow.md)。

### 工作流 1：代码审查

1. Bash 收集上下文：`git diff --cached`
2. MCP `codex` 首轮：把 diff 粘进 `prompt`，`sandbox: read-only`
3. 格式化输出给用户
4. （可选）`codex-reply` 追问细节

### 工作流 2：算法设计

1. MCP `codex` 首轮：描述约束、期望输出
2. `codex-reply`：「提供 Rust 完整实现」
3. `codex-reply`：「如何验证性能？」

---

## 详细场景示例

### 架构分析 · 输入/输出样例

**输入**：

```
分析当前微服务架构，从 1K 扩展到 10K req/sec:
- 当前: API Gateway → 5 个服务 → PostgreSQL + Redis
- 挑战: 数据库 p99 瓶颈、服务紧耦合
```

**典型输出框架**：

```
架构扩展分析
============

## 当前瓶颈分析
1. 数据库 p99：单点写入、N+1 查询
2. 服务紧耦合：同步调用链过长，级联故障风险

## 改进方案

### 短期优化（1–2 周）
- 读副本 / 慢查询索引 / 连接池
- 热点缓存、查询结果缓存

### 中期重构（1–2 月）
- 消息队列解耦
- 数据库分片（按 user_id / Citus）

### 预期效果
- 吞吐量：1K → 10K req/s
- p99：500ms → <100ms
- 可用性：99.9% → 99.99%
```

### 性能优化 · 输入/输出样例

**输入**：

```
优化以下函数性能:
- 当前 p99 = 500ms；目标 p99 < 100ms
- 分析热点并提供优化方案
[代码]
```

**典型输出框架**：

```
性能优化分析
============

## 热点分析
- 数据库查询占 80%（N+1）
- JSON 序列化占 15%

## 优化方案
1. 批量查询（IN）：-70% DB 时间
2. 索引（user_id, created_at）：100ms → 5ms
3. serde `skip_serializing_if`：-50% 序列化

## 预期效果
- p99：500ms → 80ms
- 吞吐量：+5x
```

---

## 模型选择详解

### gpt-5.3-codex

- **定位**：快速高效的日常开发助手
- **响应速度**：<10 秒
- **推理深度**：1–2 层
- **成本**：较低
- **适用**：代码审查、简单重构、文档生成、常规 bug 修复、单一目标算法
- **不适用**：多约束权衡、系统级架构、复杂性能优化

### gpt-5.4

- **定位**：深度推理的复杂问题解决专家
- **响应速度**：20–60 秒
- **推理深度**：3+ 层
- **成本**：较高
- **适用**：复杂算法、架构评审、性能瓶颈、多目标权衡、根因分析
- **不适用**：简单任务（浪费时间与成本）

### 决策树

```
任务描述
  ├─ 单一明确目标？       → 是 → gpt-5.3-codex
  ├─ 多约束需要权衡？     → 是 → gpt-5.4
  ├─ 涉及系统级设计？     → 是 → gpt-5.4
  └─ 需要深度推理？       → 是 → gpt-5.4 / 否 → gpt-5.3-codex
```

---

## 配置选项详解

### `config.model_reasoning_effort`

控制推理强度，影响输出质量与响应时间。

- `xhigh`（推荐）：最高质量，响应较慢
- `high`：高质量，中等速度
- `medium`：基本质量，较快速度

建议默认 `xhigh`，仅在时间极敏感的小任务降为 `medium`。

### `approval-policy`

控制 shell 命令执行的审批策略。

- `untrusted`：不受信任的命令需要审批
- `on-failure`：失败时审批
- `on-request`：按请求审批
- `never`：从不审批（本技能默认；自动化场景使用）

### `sandbox`

控制执行环境的文件系统权限。

- `read-only`：只读，代码审查优先
- `workspace-write`：可写当前工作区，开发/重构场景默认
- `danger-full-access`：完整权限，仅在受控环境使用

---

## 诊断流程

> 本节的 CLI 命令**只用于本地环境诊断**。日常任务执行一律通过 MCP 工具调用。

### Step 1：MCP 工具是否注册成功？

最快的确认方式是直接发起一次极简 MCP 调用：

```json
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {
    "prompt": "echo ok",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "medium"},
    "sandbox": "read-only"
  }
}
```

若工具列表里根本找不到这个名字，说明 `codex-ai` 插件没被 Claude Code 加载：检查 `.claude-plugin/marketplace.json` 是否列出了 `codex-ai` 条目、`external_plugins/codex-ai` 目录是否存在。

### Step 2：Codex CLI 本地可用性

若 MCP 工具存在但调用返回底层错误，用 CLI 直接探针：

```bash
codex --version
codex config show
```

预期：输出版本号与有效配置。失败 → 重新安装 Codex CLI / 重跑 `codex login` / 检查 `~/.config/codex/config.toml`。

也可一键跑技能自带脚本：

```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/codex-ai/scripts/check-codex-mcp.sh"
```

### Step 3：端到端回归

```bash
# 最小任务
codex exec -m gpt-5.3-codex "return the string 'ok'"
```

若以上 CLI 能工作但 MCP 调用依旧失败 → 问题在 MCP server 与 Codex 之间的桥接，查看 `codex-ai` 进程输出或重启 Claude Code。

---

## 错误处理指南

### 错误 1：MCP 工具调用失败

- **症状**：工具返回错误或超时
- **常见原因**：Codex CLI 未安装 / 登录失效 / 网络
- **处理**：走「诊断流程」三步；拿到明确报错后对应修复；临时可简化 `prompt` 或降 `model_reasoning_effort`。

### 错误 2：任务描述不清晰

- **症状**：输出泛泛、结论不可执行
- **处理**：用 `AskUserQuestion` 补齐指标 / 约束 / 边界情况；重构 prompt 后再次 `codex` 或 `codex-reply`。

### 错误 3：模型选择不当

- **症状**：简单任务用 `gpt-5.4` 导致慢；或复杂任务用 `gpt-5.3-codex` 导致粗糙
- **处理**：重新评估复杂度；同会话内用 `codex-reply` 追加「请切换到 <新模型> 重新回答」难以生效时，建议开启新 `codex` 会话。

---

## 最佳实践

### Prompt 编写

1. **明确目标**：清晰说明期望输出
2. **提供约束**：性能指标、资源限制、技术栈
3. **结构化输入**：标题、列表组织信息
4. **给出格式样例**：期望输出结构
5. **边界情况**：特殊 case / 错误处理

**对比**：

❌ `优化这段代码`

✅
```
优化以下函数性能:

## 当前代码
[代码]

## 指标
- 当前 p99 = 500ms；目标 p99 < 100ms

## 约束
- API 兼容；内存 <1GB；并发安全

## 期望输出
- 热点分析
- 至少 2 个优化方案
- 优化后代码
- 预期性能提升
```

### 多轮对话

```json
// 1) 首轮
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex",
  "parameters": {"prompt": "设计限流算法...", "model": "gpt-5.4"}
}

// 2) 追问
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex-reply",
  "parameters": {"threadId": "<threadId>", "prompt": "详细解释 Token Bucket 实现"}
}

// 3) 请求落地代码
{
  "name": "mcp__plugin_codex-ai_codex-ai__codex-reply",
  "parameters": {"threadId": "<threadId>", "prompt": "提供 Rust 完整实现"}
}
```

### 上下文传递

在 `prompt` 里塞足够背景：当前代码、指标、约束、目标。不要依赖 Codex 去"猜"上下文——每个 MCP 会话的 Codex 都是干净的新实例，除非你传 `threadId`。

---

## 常见问题 FAQ

**Q1：如何选择合适的模型？**
→ 单一目标 → `gpt-5.3-codex`；多约束 / 系统级 / 深度推理 → `gpt-5.4`；拿不准就 `gpt-5.4`。

**Q2：`model_reasoning_effort` 设多少？**
→ 默认 `xhigh`；只有时间极敏感时才降到 `medium`。

**Q3：响应超时怎么办？**
→ 拆任务 / 简化 prompt / 重试；若频繁超时检查网络与 Codex 账号限额。

**Q4：可以多轮对话吗？**
→ 可以，用 `codex-reply` + `threadId`。

**Q5：如何验证 Codex 建议的正确性？**
→ 理解原理 → 测试环境复现 → 单元 / 性能测试 → 代码审查。

**Q6：技能会自动改我的代码吗？**
→ 默认**不会**直接改。除非把 `sandbox` 设成 `workspace-write` 并让 Codex 执行写文件的操作；审查场景请务必用 `read-only`。

---

**关联文档**：
- 技能定义：[../SKILL.md](../SKILL.md)
- 用户入口：[../README.md](../README.md)
- 快速参考：[quick-reference.md](quick-reference.md)
- 完整示例：[../examples/review-workflow.md](../examples/review-workflow.md)
- 诊断脚本：[../scripts/check-codex-mcp.sh](../scripts/check-codex-mcp.sh)
