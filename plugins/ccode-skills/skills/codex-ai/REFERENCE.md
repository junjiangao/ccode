# Codex-AI 完整参考

本文档提供 Codex CLI 的完整命令参考、模型详解、配置选项和高级用法。

## 📑 目录

- [命令参考](#命令参考)
  - [codex review](#codex-review)
  - [codex exec](#codex-exec)
- [模型详解](#模型详解)
  - [gpt-5.1-codex-max](#gpt-51-codex-max)
  - [gpt-5.2](#gpt-52)
  - [模型选择决策树](#模型选择决策树)
- [配置选项](#配置选项)
  - [model_reasoning_effort](#model_reasoning_effort)
  - [其他配置参数](#其他配置参数)
- [高级用法](#高级用法)
  - [自定义审查指令](#自定义审查指令)
  - [输出到文件](#输出到文件)
  - [自动执行模式](#自动执行模式)
  - [指定工作目录](#指定工作目录)
  - [复杂场景示例](#复杂场景示例)
- [故障排查](#故障排查)
  - [常见问题](#常见问题)
  - [错误处理](#错误处理)
  - [调试技巧](#调试技巧)
- [性能优化](#性能优化)
  - [模型选择策略](#模型选择策略)
  - [参数调优建议](#参数调优建议)
- [参考资源](#参考资源)

---

## 📋 命令参考

### codex review

代码审查命令，用于分析代码变更并提供改进建议。

#### 基本语法

```bash
codex review [OPTIONS] [CUSTOM_INSTRUCTION]
```

#### 常用选项

| 选项 | 说明 | 示例 |
|------|------|------|
| `--uncommitted` | 审查未提交的变更 | `codex review --uncommitted` |
| `--base <BRANCH>` | 对比基准分支 | `codex review --base main` |
| `--commit <SHA>` | 审查特定提交 | `codex review --commit abc123` |
| `-m <MODEL>` | 指定模型 | `-m gpt-5.1-codex-max` |
| `-c <KEY=VALUE>` | 配置覆盖 | `-c 'model_reasoning_effort="xhigh"'` |

#### 使用示例

**审查未提交的变更**
```bash
codex review --uncommitted -m gpt-5.1-codex-max -c 'model_reasoning_effort="xhigh"'
```

**审查 PR 分支**
```bash
codex review --base main -m gpt-5.1-codex-max -c 'model_reasoning_effort="xhigh"'
```

**审查特定提交**
```bash
codex review --commit a1b2c3d -m gpt-5.1-codex-max -c 'model_reasoning_effort="xhigh"'
```

**自定义审查指令**
```bash
codex review "关注线程安全和内存泄漏问题" -m gpt-5.1-codex-max -c 'model_reasoning_effort="xhigh"'
```

#### 审查重点

Codex 会自动检查以下方面：
- **代码质量**：可读性、可维护性、代码风格
- **潜在问题**：空指针、资源泄漏、并发问题
- **性能**：算法复杂度、不必要的计算
- **安全**：SQL 注入、XSS、敏感信息泄露
- **最佳实践**：设计模式、错误处理、测试覆盖

### codex exec

执行任意技术任务的通用命令。

#### 基本语法

```bash
codex exec [OPTIONS] <TASK_DESCRIPTION>
```

#### 常用选项

| 选项 | 说明 | 示例 |
|------|------|------|
| `-m <MODEL>` | 指定模型 | `-m gpt-5.2` |
| `-c <KEY=VALUE>` | 配置覆盖 | `-c 'model_reasoning_effort="xhigh"'` |
| `-C <DIR>` | 工作目录 | `-C /path/to/project` |
| `--full-auto` | 自动执行模式 | `--full-auto` |
| `-o <FILE>` | 输出到文件 | `-o result.md` |

#### 使用示例

**简单任务**
```bash
codex exec -m gpt-5.1-codex-max -c 'model_reasoning_effort="xhigh"' "设计一个 LRU 缓存算法"
```

**复杂任务**
```bash
codex exec -m gpt-5.2 -c 'model_reasoning_effort="xhigh"' "设计一个分布式限流系统：
- 支持 10K req/s
- 每用户 100 req/min
- 延迟 <1ms
- 支持水平扩展"
```

**指定工作目录**
```bash
codex exec -m gpt-5.2 -c 'model_reasoning_effort="xhigh"' -C /path/to/project "分析当前项目的架构瓶颈"
```

**自动执行模式**
```bash
codex exec -m gpt-5.2 -c 'model_reasoning_effort="xhigh"' --full-auto "重构 auth 模块，提取公共逻辑"
```

**输出到文件**
```bash
codex exec -m gpt-5.2 -c 'model_reasoning_effort="xhigh"' -o architecture-analysis.md "分析系统架构并提供改进建议"
```

## 🤖 模型详解

### gpt-5.1-codex-max

**定位**：快速、高效的日常开发助手

#### 特点和性能

- **响应速度**：快速（通常 <10 秒）
- **推理能力**：适合单一目标任务
- **代码质量**：高质量的代码生成
- **成本**：相对较低

#### 适用场景

1. **代码审查**
   - 提交前的代码自查
   - PR 审查前的预检
   - 代码质量检查

2. **简单重构**
   - 提取函数或类
   - 重命名变量
   - 简化逻辑

3. **文档生成**
   - API 文档
   - 代码注释
   - README 文件

4. **常规 bug 修复**
   - 空指针错误
   - 逻辑错误
   - 边界条件处理

5. **单一目标算法**
   - LRU 缓存
   - 二分查找
   - 排序算法

#### 性能指标

- **适合任务复杂度**：低到中等
- **代码行数**：<100 行核心逻辑
- **约束条件**：1-2 个主要约束
- **推理深度**：1-2 层

#### 成本考虑

- 适合高频使用
- 日常开发任务的首选
- 成本效益比高

### gpt-5.2

**定位**：深度推理的复杂问题解决专家

#### 特点和性能

- **响应速度**：较慢（通常 20-60 秒）
- **推理能力**：强大的深度推理
- **问题解决**：处理复杂约束和权衡
- **成本**：相对较高

#### 适用场景

1. **复杂算法设计**
   - 分布式系统算法
   - 并发控制算法
   - 多约束优化问题

2. **架构评审**
   - 系统扩展性分析
   - 架构重构建议
   - 技术选型评估

3. **性能优化**
   - 性能瓶颈分析
   - 深度性能调优
   - 系统级优化

4. **多约束问题**
   - 需要权衡多个目标
   - 复杂的业务逻辑
   - 系统级设计决策

5. **深度推理任务**
   - 根因分析
   - 方案对比评估
   - 风险评估

#### 性能指标

- **适合任务复杂度**：中等到高
- **代码行数**：>100 行核心逻辑
- **约束条件**：3+ 个约束需要权衡
- **推理深度**：3+ 层深度分析

#### 成本考虑

- 适合关键决策
- 复杂问题的首选
- 需要权衡成本和质量

### 模型选择决策树

```
任务描述
    │
    ├─ 单一明确目标？
    │   ├─ 是 → gpt-5.1-codex-max
    │   └─ 否 → 继续判断
    │
    ├─ 多个约束条件需要权衡？
    │   ├─ 是 → gpt-5.2
    │   └─ 否 → 继续判断
    │
    ├─ 涉及系统级设计？
    │   ├─ 是 → gpt-5.2
    │   └─ 否 → 继续判断
    │
    ├─ 需要深度推理分析？
    │   ├─ 是 → gpt-5.2
    │   └─ 否 → gpt-5.1-codex-max
```

## ⚙️ 配置选项

### model_reasoning_effort

控制模型的推理强度，影响输出质量和响应时间。

#### 可选值

**high（推荐）**
- **推理强度**：最高
- **输出质量**：最佳
- **响应时间**：较慢
- **适用场景**：所有任务（默认推荐）

**medium**
- **推理强度**：中等
- **输出质量**：良好
- **响应时间**：中等
- **适用场景**：时间敏感的简单任务

**low**
- **推理强度**：最低
- **输出质量**：基本
- **响应时间**：最快
- **适用场景**：极简单的任务或快速原型

#### 使用建议

```bash
# 推荐：所有任务使用 high
codex exec -m gpt-5.2 -c 'model_reasoning_effort="xhigh"' "任务描述"

# 时间敏感：简单任务可用 medium
codex exec -m gpt-5.1-codex-max -c 'model_reasoning_effort="medium"' "简单重构"

# 不推荐：low 仅用于极简单场景
codex exec -m gpt-5.1-codex-max -c 'model_reasoning_effort="low"' "生成简单注释"
```

### 其他配置参数

#### 工作目录配置

```bash
# 指定项目根目录
codex exec -C /path/to/project -m gpt-5.2 "分析项目架构"
```

#### 输出配置

```bash
# 输出到文件
codex exec -o output.md -m gpt-5.2 "生成架构文档"

# 输出格式（Markdown）
codex exec -m gpt-5.2 "生成文档" > output.md
```

#### 自动执行配置

```bash
# 自动执行模式（谨慎使用）
codex exec --full-auto -m gpt-5.2 "重构代码"
```

## 🔧 高级用法

### 自定义审查指令

针对特定关注点进行代码审查。

**示例 1：关注安全问题**
```bash
codex review "重点检查 SQL 注入、XSS 和敏感信息泄露" \
  -m gpt-5.1-codex-max \
  -c 'model_reasoning_effort="xhigh"'
```

**示例 2：关注性能**
```bash
codex review "分析性能瓶颈，关注算法复杂度和不必要的计算" \
  -m gpt-5.2 \
  -c 'model_reasoning_effort="xhigh"'
```

**示例 3：关注并发安全**
```bash
codex review "检查线程安全、竞态条件和死锁风险" \
  -m gpt-5.2 \
  -c 'model_reasoning_effort="xhigh"'
```

### 输出到文件

将分析结果保存到文件以便后续参考。

**架构分析**
```bash
codex exec \
  -m gpt-5.2 \
  -c 'model_reasoning_effort="xhigh"' \
  -o architecture-analysis.md \
  "分析当前微服务架构，提供扩展性改进建议"
```

**性能优化报告**
```bash
codex exec \
  -m gpt-5.2 \
  -c 'model_reasoning_effort="xhigh"' \
  -o performance-optimization.md \
  "分析系统性能瓶颈，提供优化方案"
```

### 自动执行模式

**⚠️ 警告**：自动执行模式会直接修改代码，使用前请确保：
- 代码已提交到 Git
- 在测试分支上操作
- 理解可能的风险

**示例：自动重构**
```bash
codex exec \
  --full-auto \
  -m gpt-5.2 \
  -c 'model_reasoning_effort="xhigh"' \
  "重构 auth 模块，提取公共逻辑到 utils"
```

### 指定工作目录

在不同项目之间切换时指定工作目录。

**分析特定项目**
```bash
codex exec \
  -C /path/to/project-a \
  -m gpt-5.2 \
  -c 'model_reasoning_effort="xhigh"' \
  "分析项目架构"
```

**对比多个项目**
```bash
# 项目 A
codex exec -C /path/to/project-a -m gpt-5.2 "分析架构" > project-a.md

# 项目 B
codex exec -C /path/to/project-b -m gpt-5.2 "分析架构" > project-b.md
```

### 复杂场景示例

**场景 1：分布式系统设计**
```bash
codex exec \
  -m gpt-5.2 \
  -c 'model_reasoning_effort="xhigh"' \
  "设计一个分布式任务调度系统：

要求：
- 支持 10K 任务/秒
- 任务优先级调度
- 故障自动恢复
- 水平扩展
- 延迟 <100ms

约束：
- 使用 Redis 作为消息队列
- PostgreSQL 存储任务状态
- Kubernetes 部署

输出：
- 架构设计图
- 核心组件说明
- 数据流程
- 扩展策略"
```

**场景 2：性能优化方案**
```bash
codex exec \
  -m gpt-5.2 \
  -c 'model_reasoning_effort="xhigh"' \
  "优化 API 性能：

当前状态：
- p50: 50ms
- p99: 500ms
- p999: 2000ms
- QPS: 1000

目标：
- p99 < 100ms
- QPS > 5000

瓶颈：
- 数据库查询慢
- N+1 查询问题
- 缺少缓存

提供：
- 瓶颈分析
- 优化方案
- 预期效果
- 实施步骤"
```

## ❓ 故障排查

### 常见问题

**Q1: Codex 响应很慢怎么办？**

A: 检查以下几点：
1. 是否使用了 gpt-5.2（响应较慢是正常的）
2. 任务描述是否过于复杂
3. 网络连接是否稳定
4. 考虑使用 gpt-5.1-codex-max 处理简单任务

**Q2: Codex 的建议不符合预期？**

A: 改进方法：
1. 提供更详细的上下文
2. 明确约束条件和目标
3. 使用自定义指令指定关注点
4. 尝试使用 gpt-5.2 获得更深入的分析

**Q3: 如何验证 Codex 的建议是否正确？**

A: 验证步骤：
1. 理解建议的原理和逻辑
2. 在测试环境验证
3. 编写单元测试
4. 进行性能测试
5. 代码审查

**Q4: Codex 生成的代码有 bug 怎么办？**

A: 处理方法：
1. 不要盲目复制粘贴代码
2. 理解代码逻辑
3. 添加错误处理
4. 编写测试用例
5. 逐步集成到项目中

**Q5: 如何选择合适的模型？**

A: 参考决策树：
- 单一目标、简单任务 → gpt-5.1-codex-max
- 多约束、复杂任务 → gpt-5.2
- 不确定时，优先使用 gpt-5.2

**Q6: model_reasoning_effort 应该设置为多少？**

A: 推荐设置：
- 默认使用 `high`（推荐）
- 时间敏感的简单任务可用 `medium`
- 避免使用 `low`（质量较差）

**Q7: 自动执行模式安全吗？**

A: 安全建议：
- 仅在测试分支使用
- 确保代码已提交
- 理解可能的风险
- 执行后仔细审查变更

**Q8: 如何处理 Codex 超时？**

A: 解决方法：
1. 简化任务描述
2. 拆分为多个小任务
3. 检查网络连接
4. 重试请求

### 错误处理

**命令执行失败**
```bash
# 检查 Codex CLI 是否正确安装
codex --version

# 检查配置
codex config show

# 查看详细错误信息
codex exec --verbose -m gpt-5.2 "任务描述"
```

**模型不可用**
```bash
# 列出可用模型
codex models list

# 使用备用模型
codex exec -m gpt-5.1-codex-max "任务描述"
```

### 调试技巧

**启用详细日志**
```bash
codex exec --verbose -m gpt-5.2 "任务描述"
```

**查看执行历史**
```bash
codex history
```

**测试连接**
```bash
codex ping
```

## 📊 性能优化

### 模型选择策略

**优化原则**：
1. **成本优先**：简单任务使用 gpt-5.1-codex-max
2. **质量优先**：复杂任务使用 gpt-5.2
3. **平衡策略**：根据任务重要性选择

**成本对比**：
- gpt-5.1-codex-max：适合高频使用
- gpt-5.2：适合关键决策

**响应时间对比**：
- gpt-5.1-codex-max：<10 秒
- gpt-5.2：20-60 秒

### 参数调优建议

**推理强度调优**：
```bash
# 默认推荐（质量优先）
-c 'model_reasoning_effort="xhigh"'

# 时间敏感（平衡）
-c 'model_reasoning_effort="medium"'

# 快速原型（速度优先，不推荐）
-c 'model_reasoning_effort="low"'
```

**任务拆分策略**：
- 大任务拆分为多个小任务
- 每个任务专注单一目标
- 并行执行多个任务

**缓存策略**：
- 相似任务复用结果
- 保存常用分析到文件
- 建立知识库

## 📚 参考资源

### 官方文档

- **Codex CLI 文档**：查看 Codex CLI 官方文档获取最新功能和更新
- **模型文档**：了解各模型的详细特性和限制

### 相关工具

- **Git**：版本控制，配合 `codex review` 使用
- **IDE 集成**：部分 IDE 支持 Codex 集成
- **CI/CD**：将 Codex 集成到持续集成流程

### 最佳实践

- **代码审查**：将 Codex 作为第一道审查关卡
- **架构设计**：使用 Codex 进行架构评审和建议
- **性能优化**：定期使用 Codex 分析性能瓶颈
- **知识传承**：将 Codex 的分析结果文档化

### 社区资源

- **示例库**：查看常见任务的示例命令
- **最佳实践**：学习其他团队的使用经验
- **问题讨论**：参与社区讨论解决问题

---

**文档版本**：v1.0
**最后更新**：2025-12-16
