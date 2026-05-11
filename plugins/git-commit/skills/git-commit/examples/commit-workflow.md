# 端到端提交工作流

演示 `git-commit` 技能从接收用户请求到 commit 完成的完整过程。所有命令与具体语言无关，预检步骤由 `scripts/precheck.sh` 按项目类型自动适配。

## 场景

用户: "帮我提交这些代码"

当前状态：工作区有 3 个修改的文件，暂存区为空。

## 步骤 1 · 摸清暂存区与工作区

```json
{
  "name": "Bash",
  "parameters": {
    "command": "git status --porcelain && echo '---STAGED---' && git diff --cached --stat && echo '---WORKTREE---' && git diff --stat",
    "description": "一次性获取 porcelain 状态、暂存区统计、工作区统计"
  }
}
```

输出示例：

```
 M src/foo.rs
 M src/bar.rs
?? notes/new.md
---STAGED---
---WORKTREE---
 src/bar.rs |  6 +++---
 src/foo.rs | 20 +++++++++++++-------
 2 files changed, 17 insertions(+), 9 deletions(-)
```

判定：**暂存区为空**（`---STAGED---` 与 `---WORKTREE---` 之间没有内容），需要询问用户选择范围。

## 步骤 2 · 让用户选择提交范围

```json
{
  "name": "AskUserQuestion",
  "parameters": {
    "questions": [{
      "question": "暂存区为空。要提交哪些文件？",
      "header": "提交范围",
      "multiSelect": false,
      "options": [
        {"label": "两个改动文件（推荐）", "description": "src/foo.rs + src/bar.rs，不包含未跟踪的 notes/new.md"},
        {"label": "全部（含未跟踪）", "description": "上述两个 + notes/new.md"},
        {"label": "仅 src/foo.rs", "description": "只提交主要变更"}
      ]
    }]
  }
}
```

> **切忌直接 `git add -A` / `git add .`**（见 `references/safety-rules.md` 第 2 条）。

假设用户选"两个改动文件"。

## 步骤 3 · 精确 stage

```json
{
  "name": "Bash",
  "parameters": {
    "command": "git add src/foo.rs src/bar.rs",
    "description": "按用户选择逐个 stage"
  }
}
```

## 步骤 4 · 预检

```json
{
  "name": "Bash",
  "parameters": {
    "command": "bash \"${CLAUDE_PLUGIN_ROOT}/skills/git-commit/scripts/precheck.sh\"",
    "description": "运行预检：敏感文件/密钥扫描 + 自动探测的 fmt/lint"
  }
}
```

预期退出码：
- `0` → 全部通过，继续
- `1` → 阻断，必须修复后重新 stage 再走一遍
- `2` → 有告警（如 clippy 非零），拿去让用户确认

若脚本不可用或 `CLAUDE_PLUGIN_ROOT` 未定义，退回手动最小检查：

```bash
git diff --cached --check                 # 空白/冲突
git diff --cached | grep -En '(AKIA|sk-|-----BEGIN.*PRIVATE KEY)' || true
```

## 步骤 5 · 识别项目提交规范

按 `references/commit-conventions.md#规范识别优先级推荐` 的 5 层优先级：

```json
{
  "name": "mcp__memory__open_nodes",
  "parameters": {"names": ["project:<repo>:commit-convention"]}
}
```

`<repo>` 取 `basename "$(git rev-parse --show-toplevel)"` 的结果。

若 Memory 未命中：

```bash
# 1) 查 .gitmessage 模板
git config --get commit.template

# 2) 查 commitlint 配置
ls .commitlintrc* commitlint.config.* 2>/dev/null

# 3) 读最近 20 条历史学习风格
git log --format="%s" -20
```

把识别结果记在内存里，后面生成信息时参照。

## 步骤 6 · 生成提交信息（Sequential Thinking）

```json
{
  "name": "mcp__sequential-thinking__sequentialthinking",
  "parameters": {
    "thought": "分析 git diff --cached：src/foo.rs 新增了配置项解析分支；src/bar.rs 更新了错误消息。主要目的是为新配置项提供错误提示。判定 type=feat，scope=config。",
    "thoughtNumber": 1,
    "totalThoughts": 5,
    "nextThoughtNeeded": true
  }
}
```

（此处省略余下 4 步思考，最终得到：）

```
feat(config): 为 timeout 选项补充解析错误提示

- src/foo.rs 新增 timeout 字段的类型校验分支
- src/bar.rs 增强解析失败时的错误消息，指向具体字段
```

## 步骤 7 · 向用户展示摘要等待确认

```
📊 变更摘要
  - 修改文件：2
  - +17 / -9 行
  - 暂存区：src/foo.rs, src/bar.rs

📝 提交信息
  feat(config): 为 timeout 选项补充解析错误提示

  - src/foo.rs 新增 timeout 字段的类型校验分支
  - src/bar.rs 增强解析失败时的错误消息，指向具体字段

✅ 预检通过（ruff / cargo fmt / 敏感扫描 / 空白符）

确认提交？回复"确认" / "改成 ...：<新信息>" / "取消"。
```

## 步骤 8 · 执行提交（HEREDOC）

用户回复"确认"后：

```json
{
  "name": "Bash",
  "parameters": {
    "command": "git commit -m \"$(cat <<'EOF'\nfeat(config): 为 timeout 选项补充解析错误提示\n\n- src/foo.rs 新增 timeout 字段的类型校验分支\n- src/bar.rs 增强解析失败时的错误消息，指向具体字段\nEOF\n)\"",
    "description": "HEREDOC 创建多行提交"
  }
}
```

> `'EOF'` 带引号能阻止 shell 展开 `$var`、反引号，保留文本原貌。

## 步骤 9 · 验证提交成功

```json
{
  "name": "Bash",
  "parameters": {
    "command": "git log -1 --oneline",
    "description": "确认提交已生成"
  }
}
```

输出：

```
a1b2c3d feat(config): 为 timeout 选项补充解析错误提示
```

向用户汇报：

```
✅ 提交成功
  a1b2c3d feat(config): 为 timeout 选项补充解析错误提示
```

**不要**自动 `git push`（见 `references/safety-rules.md` 第 8 条）。

---

## 分支路径：Pre-commit Hook 失败

若步骤 8 的 commit 报错：

```
husky > pre-commit (node v20)
✖ eslint --fix
  src/foo.rs:12: unexpected token
husky > pre-commit hook failed
```

**正确做法**：

1. 告知用户 hook 报了什么
2. 修复问题（本例是代码语法）
3. 重新 `git add src/foo.rs`
4. **新建 commit**：

   ```bash
   git commit -m "$(cat <<'EOF'
   feat(config): 为 timeout 选项补充解析错误提示
   EOF
   )"
   ```

**错误做法**：

- ❌ `git commit --amend`：commit 没成功，没有"上一条"可以 amend，`--amend` 会误改前序提交。
- ❌ `git commit --no-verify`：除非用户明确要求跳过；即便跳过也应提醒风险。

## 分支路径：预检检出敏感文件

步骤 4 的预检脚本退出码为 1，输出：

```
[ FAIL ] 暂存区包含疑似敏感文件:
         - config/.env
         请用 'git restore --staged <file>' 撤销后再提交,或与用户确认是占位符.
```

**正确做法**：停下，用 `AskUserQuestion` 让用户判断是真密钥还是占位符：

- 真密钥 → 执行 `git restore --staged config/.env` + 提醒用户轮换 key
- 占位符 → 用户显式说"这是占位符继续"才继续（并在最终汇报里注明）

## 分支路径：暂存区已非空

若步骤 1 发现暂存区已有文件，跳过步骤 2-3，直接进入预检 + 生成信息 + 确认 + commit。

## 分支路径：用户要求"提交并推送"

步骤 9 之后追加：

```json
{
  "name": "Bash",
  "parameters": {
    "command": "git push",
    "description": "推送到上游（用户显式请求）"
  }
}
```

**只有**当用户消息中同时包含"提交/commit"和"推送/push"才执行。

---

**关联文档**：
- 技能定义：[../SKILL.md](../SKILL.md)
- 规范参考：[../references/commit-conventions.md](../references/commit-conventions.md)
- 安全红线：[../references/safety-rules.md](../references/safety-rules.md)
- Memory 模板：[../references/memory-templates.md](../references/memory-templates.md)
- 预检脚本：[../scripts/precheck.sh](../scripts/precheck.sh)
