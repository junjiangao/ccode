# 端到端代码审查工作流

完整演示从 git 变更采集到 Codex MCP 调用、续聊与格式化输出的全过程。复制本样例并替换占位符即可使用。

## 场景

用户: "帮我审查一下未提交的代码"

目标: 让 Codex 对当前工作区（暂存区 + 工作区）的变更做一次快速 review，标出潜在问题、性能隐患与安全风险。

## 步骤 1 · 采集上下文（Bash）

Codex 不会自动读你的仓库；需要把 diff 直接塞进 prompt。

```json
{
  "name": "Bash",
  "parameters": {
    "command": "git diff HEAD -- . ':!*.lock' ':!*.snap' | head -n 1500",
    "description": "采集当前工作区对比 HEAD 的完整 diff，过滤 lockfile 与快照"
  }
}
```

要点：
- `HEAD`：同时覆盖暂存区与未暂存区；想只看暂存区可换成 `--cached`。
- `| head -n 1500`：Codex 单轮 prompt 有上限，过长 diff 先按行截断；按行而非按字节截断可避免把中文注释、路径等多字节字符切成半字符。问题集中在某几个文件时用 `git diff HEAD -- path/to/file` 精确收敛。
- 排除 lockfile、snapshot、生成物，降低无效 token。

## 步骤 2 · 首轮 MCP 调用

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex",
  "parameters": {
    "prompt": "你是一个资深代码审查员。请审查下面这段 git diff，按优先级输出：\n\n## 审查维度\n1. 正确性：边界条件、空指针、并发、错误处理\n2. 性能：时间/空间复杂度、不必要的分配、I/O 放大\n3. 安全：命令/SQL 注入、路径穿越、敏感信息\n4. 可维护性：命名、重复代码、耦合\n\n## 输出格式\n对每一处问题给出：\n- 文件:行号\n- 严重级别（阻断 / 重要 / 提示）\n- 描述与根因\n- 具体修复建议\n\n## 变更\n```diff\n<把步骤 1 的 diff 输出原样粘贴在这里>\n```",
    "model": "gpt-5.3-codex",
    "config": {"model_reasoning_effort": "xhigh"},
    "approval-policy": "never",
    "sandbox": "read-only"
  }
}
```

保留返回的 `threadId`（形如 `thread-xxxxxxxx`），用于下一步续聊。

### 为什么用这些参数

| 参数 | 这里选的值 | 为什么 |
|------|-----------|--------|
| `model` | `gpt-5.3-codex` | review 是典型简单任务：明确目标、单约束、追求响应速度 |
| `model_reasoning_effort` | `xhigh` | review 需要细扫，推理强一些漏网少 |
| `sandbox` | `read-only` | 审查场景绝对不改代码 |
| `approval-policy` | `never` | 全程无需人工审批，避免阻塞 |

## 步骤 3 · 按模板呈现给用户

把 Codex 的 `structuredContent.result`（或文本输出）整理成下面的格式交付：

```
📊 分析结果
- 任务类型: 代码审查
- 使用模型: gpt-5.3-codex
- 变更范围: <文件数> 个文件 / <行数> 行

📝 Codex 建议

### 🔴 阻断问题
1. src/foo.rs:42 — 空指针解引用
   - 根因: option 未判 None 直接 unwrap
   - 建议: 使用 `ok_or_else` 返回错误

### 🟡 重要问题
2. src/bar.rs:88 — N+1 查询
   ...

### 🟢 提示
3. src/baz.rs:15 — 命名建议
   ...

💡 下一步行动
- 先修 #1（阻断），再处理 #2
- 如需针对某条建议展开，回复编号我会继续询问 Codex
```

## 步骤 4 · （可选）续聊追问

当用户问"#1 的修复具体怎么写？"时，**不要**开新会话——复用 `threadId`：

```json
{
  "name": "mcp__plugin_codex-mcp-tool_codex-mcp-tool__codex-reply",
  "parameters": {
    "threadId": "thread-xxxxxxxx",
    "prompt": "请给出 #1（src/foo.rs:42 空指针解引用）的完整修复代码，包含错误类型定义与调用点的错误传播。"
  }
}
```

续聊的优势：
- Codex 仍有首轮 diff 的完整上下文，不需要再次粘贴
- 保持一致的审查基线（模型、推理强度）
- 追问成本低：新一轮 prompt 只装"这次我要什么"

## 步骤 5 · 结束条件

下列任一情况结束本次审查会话，并告知用户：

- 所有问题已逐条讨论完毕
- 用户表示"先按这些修"
- 连续 2 次续聊未带来新信息（Codex 开始重复 / 泛泛）

结束后不要主动删除 `threadId`——Codex MCP server 侧由 `codex-mcp-tool` 管理会话生命周期。

## 变体：针对单个 PR 的审查

把步骤 1 的命令换成：

```bash
git diff origin/main...HEAD -- . ':!*.lock'
```

其余步骤完全相同。若 diff 超大（超过约 1500 行或明显超出模型上下文），优先分文件 review：每个核心文件独立开一次 `codex` 调用，在最后用一轮续聊做"综合意见"。

## 变体：审查他人 PR（只读 checkout）

```bash
gh pr checkout <PR 编号>
git diff main...HEAD
```

之后走同样流程。`sandbox: "read-only"` 尤其重要——避免 Codex 对他人分支做任何写操作。

---

**关联文档**：
- 技能定义：[../SKILL.md](../SKILL.md)
- 快速参考：[../references/quick-reference.md](../references/quick-reference.md)
- 完整 API：[../references/api-reference.md](../references/api-reference.md)
- 环境诊断脚本：[../scripts/check-codex-mcp.sh](../scripts/check-codex-mcp.sh)
