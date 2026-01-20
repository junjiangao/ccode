# Codex-AI 完整技术参考

本文档提供 MCP 工具调用的完整参数参考、详细示例和故障排查指南。

## 📑 目录

- [MCP 工具完整参考](#mcp-工具完整参考)
- [完整工作流程示例](#完整工作流程示例)
- [详细场景示例](#详细场景示例)
- [模型选择详解](#模型选择详解)
- [配置选项详解](#配置选项详解)
- [错误处理完整指南](#错误处理完整指南)
- [最佳实践](#最佳实践)
- [常见问题 FAQ](#常见问题-faq)
- [故障排查](#故障排查)

---

## MCP 工具完整参考

### 工具 1: mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex

发起新的 Codex 会话。

#### 参数详解

| 参数 | 类型 | 必需 | 说明 | 可选值 | 默认值 |
|------|------|------|------|--------|--------|
| `prompt` | string | 是 | 任务描述 | - | - |
| `model` | string | 否 | 模型选择 | `gpt-5.2-codex`, `gpt-5.2` | - |
| `config` | object | 否 | 配置覆盖 | - | - |
| `config.model_reasoning_effort` | string | 否 | 推理强度 | `xhigh`, `high`, `medium` | `xhigh` |
| `approval-policy` | string | 否 | 审批策略 | `untrusted`, `on-failure`, `on-request`, `never` | `never` |
| `sandbox` | string | 否 | 沙箱模式 | `read-only`, `workspace-write`, `danger-full-access` | `workspace-write` |
| `cwd` | string | 否 | 工作目录 | 文件路径 | 当前目录 |

#### 调用示例

##### 场景 1: 代码审查

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "审查以下代码变更,关注潜在问题、性能和安全性:\n\n[git diff 输出]",
    "model": "gpt-5.2-codex",
    "config": {
      "model_reasoning_effort": "xhigh"
    },
    "approval-policy": "never",
    "sandbox": "workspace-write"
  }
}
```

**返回示例**:
```json
{
  "threadId": "thread-abc123",
  "structuredContent": {
    "result": "代码审查报告...",
    "issues": [
      {
        "file": "src/main.rs",
        "line": 45,
        "issue": "潜在空指针异常",
        "suggestion": "添加 null 检查"
      }
    ]
  }
}
```

##### 场景 2: 简单算法设计

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "设计一个 LRU 缓存,容量 1000 项,支持并发访问",
    "model": "gpt-5.2-codex",
    "config": {
      "model_reasoning_effort": "xhigh"
    },
    "approval-policy": "never",
    "sandbox": "workspace-write"
  }
}
```

##### 场景 3: 复杂算法设计

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "设计一个分布式限流算法:\n- 处理 10K req/s\n- 每用户 100 req/min\n- 延迟 <1ms\n\n要求:\n- 完整数据结构设计\n- 并发控制策略\n- 性能分析",
    "model": "gpt-5.2",
    "config": {
      "model_reasoning_effort": "xhigh"
    },
    "approval-policy": "never",
    "sandbox": "workspace-write"
  }
}
```

##### 场景 4: 架构分析

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "分析当前微服务架构,从 1K 扩展到 10K req/sec:\n- 当前: API Gateway → 5个服务 → PostgreSQL + Redis\n- 挑战: 数据库 p99 瓶颈、服务紧耦合\n- 建议改进方案",
    "model": "gpt-5.2",
    "config": {
      "model_reasoning_effort": "xhigh"
    },
    "approval-policy": "never",
    "sandbox": "workspace-write"
  }
}
```

##### 场景 5: 性能优化

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "优化以下函数性能:\n- 当前延迟: p99 = 500ms\n- 目标: p99 < 100ms\n- 分析热点并提供优化方案\n\n[代码]",
    "model": "gpt-5.2",
    "config": {
      "model_reasoning_effort": "xhigh"
    },
    "approval-policy": "never",
    "sandbox": "workspace-write"
  }
}
```

### 工具 2: mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply

继续现有 Codex 会话。

#### 参数详解

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `threadId` | string | 是 | 从上次调用返回的线程 ID |
| `prompt` | string | 是 | 后续问题或指令 |

#### 调用示例

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "thread-abc123",
    "prompt": "请详细解释第二个优化方案的实现细节"
  }
}
```

---

## 完整工作流程示例

### 工作流 1: 代码审查完整流程

**步骤 1**: 获取 git diff (Bash 工具)
```json
{
  "name": "Bash",
  "parameters": {
    "command": "git diff --cached",
    "description": "获取暂存区代码变更"
  }
}
```

**步骤 2**: 调用 Codex 审查 (MCP 工具)
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "审查以下代码变更:\n\n[步骤1的输出]",
    "model": "gpt-5.2-codex",
    "config": {
      "model_reasoning_effort": "xhigh"
    }
  }
}
```

**步骤 3**: 格式化输出给用户

**步骤 4** (可选): 追问细节
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "[步骤2返回的threadId]",
    "prompt": "请解释第一个问题的修复方案"
  }
}
```

### 工作流 2: 算法设计完整流程

**步骤 1**: 调用 Codex 设计算法
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "设计分布式限流算法:\n- 10K req/s\n- 100 req/min per user\n- <1ms 延迟",
    "model": "gpt-5.2"
  }
}
```

**步骤 2**: 请求详细实现
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "[步骤1的threadId]",
    "prompt": "提供 Rust 完整实现"
  }
}
```

**步骤 3**: 性能验证讨论
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "[同一threadId]",
    "prompt": "如何验证性能满足要求?"
  }
}
```

---

## 详细场景示例

### 场景 3: 架构分析

**示例: 系统扩展设计**

**输入**:
```
分析当前微服务架构,从 1K 扩展到 10K req/sec:
- 当前: API Gateway → 5个服务 → PostgreSQL + Redis
- 挑战: 数据库 p99 瓶颈、服务紧耦合
```

**MCP 工具调用**:
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "分析当前微服务架构,从 1K 扩展到 10K req/sec:\n- 当前: API Gateway → 5个服务 → PostgreSQL + Redis\n- 挑战: 数据库 p99 瓶颈、服务紧耦合\n- 建议改进方案",
    "model": "gpt-5.2",
    "config": {
      "model_reasoning_effort": "xhigh"
    },
    "approval-policy": "never",
    "sandbox": "workspace-write"
  }
}
```

**输出示例**:
```
架构扩展分析
===========

## 当前瓶颈分析
1. 数据库 p99 瓶颈:
   - 原因: 单点写入、N+1 查询
   - 影响: 限制整体吞吐量

2. 服务紧耦合:
   - 原因: 同步调用链过长
   - 影响: 级联故障风险

## 改进方案

### 短期优化 (1-2 周)
1. 数据库优化:
   - 添加读副本 (读写分离)
   - 优化慢查询 (添加索引)
   - 引入连接池

2. 缓存策略:
   - 热点数据缓存 (Redis)
   - 查询结果缓存

### 中期重构 (1-2 月)
1. 服务解耦:
   - 引入消息队列 (Kafka/RabbitMQ)
   - 异步处理非关键路径

2. 数据库分片:
   - 按用户 ID 分片
   - 使用 Citus 或自建分片

### 预期效果
- 吞吐量: 1K → 10K req/s
- p99 延迟: 500ms → <100ms
- 可用性: 99.9% → 99.99%
```

### 场景 4: 性能优化

**示例: 函数性能优化**

**输入**:
```
优化以下函数性能:
- 当前延迟: p99 = 500ms
- 目标: p99 < 100ms
- 分析热点并提供优化方案

[代码]
```

**MCP 工具调用**:
```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "优化以下函数性能:\n- 当前延迟: p99 = 500ms\n- 目标: p99 < 100ms\n- 分析热点并提供优化方案\n\n[代码]",
    "model": "gpt-5.2",
    "config": {
      "model_reasoning_effort": "xhigh"
    },
    "approval-policy": "never",
    "sandbox": "workspace-write"
  }
}
```

**输出示例**:
```
性能优化分析
===========

## 热点分析
1. 数据库查询 (占 80% 时间)
   - N+1 查询问题
   - 缺少索引

2. JSON 序列化 (占 15% 时间)
   - 大对象序列化开销

## 优化方案

### 优化 1: 批量查询
- 当前: 循环中单次查询
- 优化: 使用 IN 查询批量获取
- 预期: 减少 70% 数据库时间

### 优化 2: 添加索引
- 字段: user_id, created_at
- 预期: 查询时间从 100ms → 5ms

### 优化 3: 优化序列化
- 使用 serde 的 skip_serializing_if
- 减少不必要的字段序列化
- 预期: 减少 50% 序列化时间

## 预期效果
- p99 延迟: 500ms → 80ms
- 吞吐量: 提升 5x
```

---

## 模型选择详解

### gpt-5.2-codex

**定位**: 快速高效的日常开发助手

**性能特点**:
- 响应速度: 快速 (<10秒)
- 推理深度: 1-2 层
- 代码质量: 高质量
- 成本: 较低

**适用场景**:
1. 代码审查 (提交前自查、PR 预检)
2. 简单重构 (提取函数、重命名变量)
3. 文档生成 (API 文档、代码注释)
4. 常规 bug 修复 (空指针、逻辑错误)
5. 单一目标算法 (LRU 缓存、二分查找)

**不适用场景**:
- 多约束权衡
- 系统级架构设计
- 复杂性能优化

### gpt-5.2

**定位**: 深度推理的复杂问题解决专家

**性能特点**:
- 响应速度: 较慢 (20-60秒)
- 推理深度: 3+ 层
- 问题解决: 处理复杂约束
- 成本: 较高

**适用场景**:
1. 复杂算法设计 (分布式算法、并发控制)
2. 架构评审 (扩展性分析、重构建议)
3. 性能优化 (瓶颈分析、系统级优化)
4. 多约束问题 (多目标权衡)
5. 深度推理 (根因分析、方案对比)

**不适用场景**:
- 简单任务 (浪费时间和成本)

### 模型选择决策树

```
任务描述
    │
    ├─ 单一明确目标?
    │   ├─ 是 → gpt-5.2-codex
    │   └─ 否 → 继续判断
    │
    ├─ 多个约束需要权衡?
    │   ├─ 是 → gpt-5.2
    │   └─ 否 → 继续判断
    │
    ├─ 涉及系统级设计?
    │   ├─ 是 → gpt-5.2
    │   └─ 否 → 继续判断
    │
    ├─ 需要深度推理?
    │   ├─ 是 → gpt-5.2
    │   └─ 否 → gpt-5.2-codex
```

---

## 配置选项详解

### model_reasoning_effort

控制模型推理强度,影响输出质量和响应时间。

**可选值**:

- **xhigh** (推荐): 最高推理强度,最佳质量,响应较慢
- **high**: 高推理强度,良好质量,中等速度
- **medium**: 中等推理强度,基本质量,较快速度

**使用建议**:
- 默认使用 `xhigh` 确保质量
- 仅在时间极度敏感的简单任务使用 `medium`

### approval-policy

控制 shell 命令执行的审批策略。

**可选值**:

- **untrusted** (默认): 不受信任的命令需要审批
- **on-failure**: 失败时需要审批
- **on-request**: 根据请求审批
- **never**: 从不需要审批 (谨慎使用)

### sandbox

控制执行环境的访问权限。

**可选值**:

- **read-only**: 只读访问 (代码审查推荐)
- **workspace-write**: 可写工作区 (算法设计、重构推荐)
- **danger-full-access**: 完全访问 (谨慎使用)

---

## 错误处理完整指南

### 错误场景 1: 工具调用失败

**症状**: MCP 工具返回错误或超时

**可能原因**:
- Codex CLI 未正确安装
- 配置错误
- 网络问题

**诊断步骤**:
1. 检查 Codex CLI 版本
2. 验证配置
3. 测试网络连接

**解决方案**:
1. 检查安装: `codex --version`
2. 验证配置: `codex config show`
3. 简化任务描述
4. 重试请求

### 错误场景 2: 任务描述不清晰

**症状**: Codex 输出不符合预期或过于泛泛

**解决方案**:
1. 使用 `AskUserQuestion` 询问补充信息:
   - 具体性能指标
   - 约束条件
   - 边界情况
2. 重新构建更详细的 prompt
3. 再次调用 MCP 工具

### 错误场景 3: 模型选择不当

**症状**: 简单任务使用 gpt-5.2 导致响应慢,或复杂任务使用 gpt-5.2-codex 质量不足

**解决方案**:
1. 重新评估任务复杂度
2. 使用 `codex-reply` 继续会话并切换模型
3. 向用户说明模型选择原因

---

## 最佳实践

### Prompt 编写最佳实践

1. **明确目标**: 清晰描述期望的输出
2. **提供约束**: 性能指标、资源限制、技术栈
3. **结构化输入**: 使用标题、列表组织信息
4. **提供示例**: 给出期望的输出格式示例
5. **边界情况**: 说明特殊情况和边界条件

**示例对比**:

❌ 不好的 prompt:
```
"优化这段代码"
```

✅ 好的 prompt:
```
"优化以下函数性能:

## 当前代码
[代码]

## 性能指标
- 当前: p99 = 500ms
- 目标: p99 < 100ms

## 约束
- 保持 API 兼容
- 内存 <1GB
- 并发安全

## 期望输出
- 热点分析
- 优化方案 (至少 2 个)
- 优化后代码
- 预期性能提升
"
```

### 多轮对话最佳实践

使用 `codex-reply` 进行深入讨论:

```json
// 第一轮
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "设计限流算法...",
    "model": "gpt-5.2"
  }
}

// 第二轮 - 追问细节
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "[上次返回的threadId]",
    "prompt": "详细解释 Token Bucket 的实现"
  }
}

// 第三轮 - 请求代码
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "[同一threadId]",
    "prompt": "提供 Rust 完整实现"
  }
}
```

### 上下文传递最佳实践

在 prompt 中传递丰富上下文:

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "优化性能:\n\n## 当前代码\n[代码片段]\n\n## 性能指标\n- p50: 50ms\n- p99: 500ms\n\n## 目标\n- p99 < 100ms\n\n## 约束\n- 保持 API 兼容\n- 内存 <1GB",
    "model": "gpt-5.2"
  }
}
```

---

## 常见问题 FAQ

**Q1: 如何选择合适的模型?**

A: 参考决策树:
- 单一目标、简单任务 → gpt-5.2-codex
- 多约束、复杂任务 → gpt-5.2
- 不确定时,优先使用 gpt-5.2

**Q2: model_reasoning_effort 应该设置为多少?**

A: 推荐 `xhigh` 确保质量。仅在时间极度敏感的简单任务使用 `medium`。

**Q3: 如何处理超时?**

A:
1. 简化任务描述
2. 拆分为多个小任务
3. 检查网络连接
4. 重试请求

**Q4: 可以多轮对话吗?**

A: 可以。使用 `mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply` 继续会话。

**Q5: 如何验证建议的正确性?**

A:
1. 理解建议的原理
2. 在测试环境验证
3. 编写单元测试
4. 进行性能测试
5. 代码审查

---

## 故障排查

### 诊断检查清单

- [ ] Codex CLI 已正确安装
- [ ] 配置正确
- [ ] 网络连接正常
- [ ] 任务描述清晰
- [ ] 模型选择合适
- [ ] 参数配置正确

### 常见问题快速索引

| 问题 | 可能原因 | 解决方案 |
|------|---------|---------|
| 工具调用失败 | CLI 未安装 | 安装 Codex CLI |
| 响应超时 | 任务过于复杂 | 拆分任务 |
| 输出质量差 | 模型选择不当 | 切换到 gpt-5.2 |
| 格式不符合预期 | prompt 不清晰 | 提供详细约束 |

### 调试技巧

1. **启用详细日志**: 查看完整的 MCP 工具调用和响应
2. **测试简单任务**: 确认基本功能正常
3. **对比示例**: 参考文档中的示例调整参数
4. **逐步调试**: 从简单任务开始,逐步增加复杂度

---

**文档版本**: v2.0
**最后更新**: 2025-01-20
