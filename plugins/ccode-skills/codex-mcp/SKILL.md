---
name: codex-mcp
description: 调用 Codex 进行深度分析、复杂逻辑设计和代码审查。使用场景：>10行核心逻辑、架构设计、性能优化、关键代码审查。提供标准的 MCP 工具调用接口和协作模板。
---

# Codex MCP 协作

本技能通过标准 MCP（Model Context Protocol）工具调用接口，协作 Codex 处理复杂技术任务。

## 使用场景

### 触发条件
满足以下任一条件时激活：
- 需要设计或优化 >10 行核心逻辑的算法
- 请求架构评审、性能优化或安全审计
- 涉及数学证明、复杂问题求解
- 描述包含 "深度分析"、"复杂逻辑"、"算法设计"、"架构审查" 等关键词

### 典型应用
```markdown
"设计一个并发数据处理算法处理 1000 events/sec"
"优化这个状态机实现，需要代码审查"
"请分析系统架构，支持从 1K 扩展到 10K req/sec"
```

## MCP 工具调用方法

### 工具初始化

**MCP 工具名称**：`mcp__codex-mcp-tool__codex`

**开启会话**（必须设置固定参数）：

#### 默认模型：gpt-5-codex
适用于大多数复杂技术任务和分析工作：

**完整工具调用示例**：
```json
{
  "name": "mcp__codex-mcp-tool__codex",
  "parameters": {
    "model": "gpt-5-codex",
    "sandbox": "danger-full-access",
    "approval-policy": "on-failure",
    "prompt": "<需求描述或任务说明>",
    "cwd": "<可选：工程路径>"
  }
}
```

#### 高级模型：gpt-5
适用于特别复杂的任务或特殊指定场景：

**完整工具调用示例**：
```json
{
  "name": "mcp__codex-mcp-tool__codex",
  "parameters": {
    "model": "gpt-5",
    "sandbox": "danger-full-access",
    "approval-policy": "on-failure",
    "prompt": "<需求描述或任务说明>",
    "cwd": "<可选：工程路径>"
  }
}
```

**返回值**：`{ conversationId: "<string>", ... }`

### 工具调用参数

#### 必选参数
- `model`: 模型选择，支持以下选项：
  - **"gpt-5-codex"**（默认）- 适用于大多数复杂技术任务、算法设计、架构分析
  - **"gpt-5"** - 适用于特别复杂的任务或用户特殊指定，需要更强的推理能力
- `prompt`: 任务描述，支持中英文

**模型选择指南**：
- 使用 **gpt-5-codex**：>90% 的协作场景（默认推荐）
- 使用 **gpt-5**：遇到高度复杂的多约束优化、超大规模系统设计、或用户明确指定时
- `sandbox`: 沙盒模式
  - `"read-only"` - 仅读取权限
  - `"workspace-write"` - 可写入工作区
  - `"danger-full-access"` - 完全访问权限
- `approval-policy`: 命令审批策略
  - `"untrusted"` - 无需审批
  - `"on-failure"` - 失败时审批
  - `"on-request"` - 按需审批
  - `"never"` - 从不审批

#### 可选参数
- `cwd`: 工作目录
- `base-instructions`: 基础指令
- `compact-prompt`: 紧凑提示模式
- `developer-instructions`: 开发者指令
- `config`: 配置对象覆盖

### 继续对话

**MCP 工具名称**：`mcp__codex-mcp-tool__codex-reply`

**完整工具调用示例**：
```json
{
  "name": "mcp__codex-mcp-tool__codex-reply",
  "parameters": {
    "conversationId": "<上步返回的 conversationId>",
    "prompt": "<补充问题或新指令>"
  }
}
```

⚠️ **会话管理**：保存返回的 `conversationId`，会话失效时需重新初始化。

## 协作模板

### 标准任务模板

```markdown
## 任务给 Codex

**背景**: [项目/系统简要描述]
**目标**: [清晰的目标陈述]
**约束**: [性能/安全/兼容性要求]
**当前状态**: [现有实现或尝试]

### 具体问题:
1. [第一个问题]
2. [第二个问题]

### 期望交付:
- [ ] 设计文档/伪代码
- [ ] 实现策略
- [ ] 测试用例
- [ ] 性能分析
```

### 算法设计请求
```markdown
## 算法设计

**问题描述**: [问题陈述]
**输入格式**: [数据结构与约束]
**输出要求**: [期望结果格式]
**性能约束**:
- 时间复杂度: O(?)
- 空间复杂度: O(?)
**测试用例**:
1. 输入: [...] → 期望: [...]
2. 边界情况: [...] → 期望: [...]
```

### 架构评审请求
```markdown
## 架构评审

**系统概览**: [高层描述]
**组件**:
- 组件A: [用途和交互]
- 组件B: [用途和交互]
**当前挑战**:
1. [挑战1]
2. [挑战2]
**扩展要求**:
- 当前负载: [指标]
- 预期增长: [预测]
**分析问题**:
1. [具体架构关注点]
2. [性能优化机会]
```

## 最佳实践

### ✅ 推荐做法
- 提供完整的上下文（包括约束和边界情况）
- 共享现有代码模式和约定
- 明确性能要求
- 询问具体、有针对性的问题
- 在实施前验证 Codex 的建议

### ❌ 避免事项
- 假设 Codex 了解你的项目结构
- 跳过测试 Codex 提出的解决方案
- 不理解推理过程就实施
- 忘记检查与现有系统的兼容性

## 参考文档

- [REFERENCE.md](REFERENCE.md) - 完整集成指南、模式速查和 MCP 工具规范
- [HANDOFF_CHECKLIST.md](HANDOFF_CHECKLIST.md) - 任务交接检查清单
