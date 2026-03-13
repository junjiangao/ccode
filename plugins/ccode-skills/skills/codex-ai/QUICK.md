# Codex-AI 快速参考

快速查找常用命令和常见问题的速查表。

## 常用命令模版

### 代码审查
```bash
codex review --uncommitted -m gpt-5.3-codex -c 'model_reasoning_effort="xhigh"'
```

### 简单任务（代码审查、简单重构、文档生成）
```bash
codex exec -m gpt-5.3-codex -c 'model_reasoning_effort="high"' "<任务描述>"
```

### 复杂任务（算法设计、架构评审、性能优化）
```bash
codex exec -m gpt-5.4 -c 'model_reasoning_effort="xhigh"' "<任务描述>"
```

## 模型选择快速决策

| 场景 | 推荐模型 | 特点 |
|------|---------|------|
| 代码审查、简单重构、单一目标算法 | **gpt-5.3-codex** | 快速响应、高质量、低成本 |
| 复杂算法设计、架构评审、性能优化、多约束权衡 | **gpt-5.4** | 深度推理、处理复杂约束 |

**快速判断规则**：
- 任务 <10 行核心逻辑、单一目标 → 使用 `gpt-5.3-codex`
- 任务涉及系统级设计、多约束需权衡 → 使用 `gpt-5.4`

## MCP 工具调用速查

### 发起新会话
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "<任务描述>",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "xhigh"}
  }
}
```

### 继续会话
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "<之前的 threadId>",
    "prompt": "<后续问题>"
  }
}
```

## 常见问题速查

| 问题 | 诊断 | 解决方案 |
|------|------|---------|
| 工具调用失败 | Codex CLI 未安装 | `codex --version` 检查安装 |
| 响应超时 | 任务过于复杂 | 简化任务描述或切换到更快的模型 |
| 输出质量差 | 模型选择不当 | 复杂任务切换到 `gpt-5.4` |
| 格式不符合预期 | prompt 不清晰 | 在描述中添加具体约束和期望输出格式 |

## 环境检查三步法

### Step 1: 基础检查
```bash
codex --version  # 检查安装
codex config show  # 检查配置
```

### Step 2: 测试简单任务
```bash
codex exec -m gpt-5.3-codex "解释 1+1=2 的原理"
```

### Step 3: 测试复杂任务
```bash
codex exec -m gpt-5.4 -c 'model_reasoning_effort="xhigh"' "设计一个简单的 LRU 缓存算法"
```

## 关键参数速查

| 参数 | 说明 | 推荐值 |
|------|------|--------|
| `model` | 模型选择 | `gpt-5.3-codex` (简单) / `gpt-5.4` (复杂) |
| `model_reasoning_effort` | 推理强度 | `xhigh` (推荐) / `high` / `medium` |
| `approval-policy` | 命令审批策略 | `never` (自动) / `on-failure` |
| `sandbox` | 访问权限 | `read-only` (审查) / `workspace-write` (开发) |

## 使用场景速查

| 场景 | 命令前缀 | 示例 |
|------|----------|------|
| 代码审查 | `codex review --uncommitted` | 审查未提交变更 |
| 算法设计 | `codex exec -m gpt-5.4` | 设计分布式限流 |
| 架构分析 | `codex exec -m gpt-5.4` | 分析微服务扩展性 |
| 性能优化 | `codex exec -m gpt-5.4` | 优化 API 延迟 |

---

**更多详细信息**：
- 完整工作流程：查看 [README.md](README.md)
- 技术参考：查看 [REFERENCE.md](REFERENCE.md)
- 技能定义：查看 [SKILL.md](SKILL.md)
