# Codex MCP 参考指南

完整的技术参考文档，包含集成指南、协作模式和 MCP 工具规范。

---

## 第一部分：集成指南

### 集成流程

```
接收建议 → 验证逻辑 → 计划集成 → 分阶段实施 → 测试验证 → 监控测量
```

### 阶段 1: 验证

#### 1.1 逻辑验证
检查清单：
- [ ] 算法正确性已验证（手动追踪或证明）
- [ ] 覆盖边界情况
- [ ] 确认复杂度分析
- [ ] 记录所有假设

#### 1.2 兼容性检查
- [ ] 语言版本兼容性
- [ ] 库依赖可用性
- [ ] 平台兼容性（OS、架构）
- [ ] 识别现有代码集成点
- [ ] 评估破坏性变更

兼容性矩阵：
| 方面 | Codex 假设 | 你的环境 | 兼容？ | 所需操作 |
|------|-----------|----------|--------|----------|
| 语言 | Rust 1.75 | Rust 1.70 | ✗ | 升级或适配 |
| 库 | tokio 1.35 | tokio 1.28 | ⚠️ | 测试兼容性 |
| 平台 | Unix | 跨平台 | ✗ | 添加 Windows 支持 |

#### 1.3 性能验证
```rust
#[cfg(test)]
mod codex_validation {
    use super::*;
    use std::time::Instant;

    #[test]
    fn validate_codex_performance() {
        let input = generate_test_data(10_000);
        let start = Instant::now();
        let result = codex_suggested_function(&input);
        let duration = start.elapsed();

        // 验证声称的性能
        assert!(duration.as_millis() < 100, "应 <100ms 完成");
        assert!(result.is_correct(), "结果必须正确");
    }
}
```

### 阶段 2: 集成策略

#### 风险评估
| 风险级别 | 标准 | 缓解策略 |
|----------|------|----------|
| **低** | 非关键代码，易回滚 | 直接实施 |
| **中** | 影响多个组件 | 功能标志 + 分阶段推出 |
| **高** | 关键路径，难回滚 | 影子模式 + 广泛测试 |
| **关键** | 财务/安全影响 | 完全审计 + 渐进迁移 |

#### 策略选择

**直接集成**（低风险）：
```rust
fn old_implementation() -> Result<Output> {
    // Codex 的改进实现
    new_implementation()
}
```

**并行实现**（中等风险）：
```rust
fn process_data(input: &Input) -> Result<Output> {
    if feature_flag_enabled() {
        new_implementation(input)
    } else {
        old_implementation(input)
    }
}
```

**影子模式**（高风险）：
```rust
fn process_data(input: &Input) -> Result<Output> {
    let result = old_implementation(input);

    tokio::spawn(async move {
        match new_implementation(input).await {
            Ok(shadow_result) => {
                if result != shadow_result {
                    log::warn!("影子实现不同: {:?}", shadow_result);
                }
                metrics::record_shadow_latency();
            }
            Err(e) => {
                log::error!("影子实现失败: {}", e);
            }
        }
    });

    result
}
```

### 阶段 3: 测试策略

**测试金字塔**：
```
        ┌─────────────┐
        │   手动      │  ← 复杂场景
        │   测试      │
        ├─────────────┤
        │  集成      │  ← API/组件测试
        │   测试      │
        ├─────────────┤
        │    单元     │  ← 算法正确性
        │   测试      │
        └─────────────┘
```

**必须测试**：
- [ ] 单元测试（所有绿色）
- [ ] 集成测试
- [ ] 性能基准（满足目标）
  - [ ] 延迟改进 __%
  - [ ] 吞吐量增加 __%
  - [ ] 内存使用在限制内
- [ ] 负载测试（30+ 分钟）
- [ ] 错误处理验证

### 阶段 4: 监控

**关键指标**：
```markdown
- **延迟**: p50, p95, p99, max
- **吞吐量**: requests/second
- **错误率**: errors/total requests
- **资源使用**: CPU, memory, disk I/O
```

**警报配置**：
```yaml
alerts:
  - name: LatencyIncrease
    condition: p99_latency > baseline * 1.5
    action: notify_team

  - name: ErrorRateSpike
    condition: error_rate > baseline * 2
    action: auto_rollback
```

---

## 第二部分：协作模式

### 常用模式速查

| 任务类型 | 最佳模式 | 关键成功因素 |
|----------|----------|--------------|
| 新算法 | 迭代优化 | 明确约束 |
| 系统扩展 | 扩展规划 | 当前指标 |
| 性能问题 | 度量-分析-优化 | 分析数据 |
| 代码审查 | 重构提案 | 质量目标 |
| 架构选择 | 权衡矩阵 | 评估标准 |

### 核心模式

#### 1. 迭代优化
**场景**：设计多约束的复杂算法
```markdown
阶段1: "这里是朴素解决方案。请分析其复杂度。"
阶段2: "O(n²) 太慢，建议 O(n log n) 的优化。"
阶段3: "能否将空间复杂度从 O(n) 降至 O(1)？"
```

#### 2. 瓶颈分析
**场景**：系统性能在负载下退化
```markdown
**当前架构**: [图表]
**指标**:
- 95th 百分位延迟: 2s
- CPU 利用率: 80%
- 内存: 4GB/8GB

**问题**: 瓶颈在哪里？如何优化？
```

#### 3. 权衡矩阵
**场景**：存在多种架构选项
```markdown
**选项**:
1. 微服务
2. 模块化单体
3. 事件驱动

**标准**: 性能、可维护性、团队规模、时间线

**问题**: 为我们的约束创建权衡矩阵。
```

### 反模式

❌ **模糊问题陈述**
```markdown
错误: "我的代码很慢。怎么修复？"

正确: "processData() 函数处理 10K 记录需要 5s。
分析显示 80% 时间在嵌套循环中。
目标: <500ms。这是代码..."
```

❌ **缺少上下文**
```markdown
错误: "审查这个函数的 bug。" [仅函数代码]

正确: "审查这个认证函数：
- 用于公共 API
- 处理 1K req/min
- 对安全性至关重要
- 必须使用 JWT 令牌"
```

---

## 第三部分：MCP 工具规范

### 标准结构

```json
{
  "name": "tool_name",
  "description": "此工具的功能和使用时机",
  "inputSchema": {
    "type": "object",
    "properties": {
      "param1": {
        "type": "string",
        "description": "参数描述"
      }
    },
    "required": ["param1"]
  }
}
```

### 字段要求

**name**（必需）：
- 格式：snake_case 或 kebab-case
- 在服务器内必须唯一
- 示例：`"analyze_code"`, `"run-benchmark"`

**description**（推荐）：
- 包括：功能、时机、返回值
- 示例：`"在分析指标时使用。执行 SQL 查询。返回 JSON。"`

**inputSchema**（必需）：
- JSON Schema（Draft 7+）
- 必须指定：type、properties、required 字段

### 端点

#### 1. 列出工具 (`tools/list`)
```json
请求: {"method": "tools/list"}

响应:
{
  "tools": [
    {"name": "tool1", "description": "...", "inputSchema": {...}},
    {"name": "tool2", "description": "...", "inputSchema": {...}}
  ]
}
```

#### 2. 调用工具 (`tools/call`)
```json
请求:
{
  "method": "tools/call",
  "params": {
    "name": "tool_name",
    "arguments": {"param1": "value1"}
  }
}

响应:
{
  "content": [{"type": "text", "text": "结果..."}],
  "isError": false
}
```

### 工具类别

#### 代码分析
```json
{
  "name": "analyze_code",
  "description": "执行静态分析。识别 bug、性能问题、安全漏洞。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "code": {"type": "string"},
      "language": {"type": "string", "enum": ["rust", "python"]},
      "checks": {"type": "array", "items": {"type": "string"}}
    },
    "required": ["code", "language"]
  }
}
```

#### 性能测试
```json
{
  "name": "run_benchmark",
  "description": "执行性能基准测试。Codex 建议优化时使用。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "function_name": {"type": "string"},
      "iterations": {"type": "integer", "minimum": 1}
    },
    "required": ["function_name", "iterations"]
  }
}
```

#### 测试执行
```json
{
  "name": "run_tests",
  "description": "执行测试套件。验证实现时使用。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "test_path": {"type": "string"},
      "coverage": {"type": "boolean"}
    },
    "required": ["test_path"]
  }
}
```

### 最佳实践

#### 1. 工具发现
当 Codex 推荐工具时：
1. 列出可用工具：`tools/list`
2. 验证工具存在
3. 检查 inputSchema 要求
4. 准备参数

#### 2. 参数准备
**错误**：缺少必需参数
```json
{"name": "analyze_code", "arguments": {"code": "..."}}
```

**正确**：所有必需参数
```json
{
  "name": "analyze_code",
  "arguments": {
    "code": "fn main() {...}",
    "language": "rust"
  }
}
```

#### 3. 错误处理
```rust
match call_mcp_tool("analyze_code", args) {
    Ok(result) => send_to_codex(result),
    Err(e) if e.is_invalid_args() => {
        let fixed = fix_arguments(args);
        call_mcp_tool("analyze_code", fixed)?
    }
    Err(e) => report_tool_failure(e)
}
```

### 安全与权限

#### 权限配置
在交接中指定允许的工具：

```markdown
## 可用 MCP 工具

**允许**:
- code_analyzer: 是（只读）
- run_benchmark: 是（安全测试）

**不允许**:
- execute_query: 否（不需要数据访问）
- system_command: 否（安全限制）
```

#### 安全约束
```json
{
  "tool_constraints": {
    "code_analyzer": {
      "max_code_size": 10000,
      "allowed_languages": ["rust", "python"]
    }
  }
}
```

---

## 快速参考

### MCP 工具调用检查表
```
□ 工具存在 (tools/list)
□ inputSchema 已审查
□ 必需参数已准备
□ 权限已确认
□ 错误处理已就绪
□ 结果解释计划
```

### 集成检查清单
```
验证:
□ 算法正确性
□ 兼容性
□ 性能声明

规划:
□ 风险评估
□ 集成策略
□ 回滚计划

实施:
□ 功能分支
□ 隔离测试
□ 分阶段集成
□ 文档更新
```

### 常用模板

#### 标准任务交接
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

#### 算法设计
```markdown
**问题陈述**: [描述问题]
**输入格式**: [数据结构与约束]
**输出要求**: [期望结果格式]
**性能约束**:
- 时间复杂度: O(?)
- 空间复杂度: O(?)
**测试用例**:
1. 输入: [...] → 期望: [...]
2. 边界情况: [...] → 期望: [...]
```

#### 架构评审
```markdown
**系统概览**: [高层描述]
**组件**:
- 组件 A: [用途和交互]
- 组件 B: [用途和交互]
**当前挑战**:
1. [挑战 1]
2. [挑战 2]
**扩展要求**:
- 当前负载: [指标]
- 预期增长: [预测]
**分析问题**:
1. [具体架构关注点]
2. [性能优化机会]
```

---

## 版本信息

**版本**: 2.0 | **更新**: 2025-11-05

合并了以下文档：
- INTEGRATION_GUIDE.md (v1.0)
- CODEX_PATTERNS.md (v1.0)
- MCP_TOOLS_SPEC.md (v1.0)

---

返回 [SKILL.md](SKILL.md) 以获取主要文档。
