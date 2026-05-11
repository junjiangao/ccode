---
name: git-commit
description: 当用户要求"提交代码 / commit / git commit / 帮我提交 / 提交当前修改 / 审查并提交"时使用本技能。它会探测暂存区、运行项目自适应的预检（格式化/lint/敏感信息扫描）、学习项目提交风格、生成符合规范的提交信息，并在用户确认后执行 `git commit`。通用实现，不绑定特定语言或框架。
allowed-tools:
  - Bash
  - AskUserQuestion
  - mcp__memory__open_nodes
  - mcp__memory__search_nodes
  - mcp__memory__create_entities
  - mcp__memory__add_observations
  - mcp__sequential-thinking__sequentialthinking
---

# Git Commit 技能

智能化 Git 提交助手：检查变更 → 预检 → 识别规范 → 生成信息 → 确认 → 提交。**仅在用户明确要求时 commit**；其他自动化场景（如 `保存`、`build`）不触发。

## 何时使用

- 用户说"提交"、"commit"、"git commit"、"帮我提交"、"把这些改动提交到 git"、"审查并提交"
- 用户完成修改后显式请求入库

## 何时不使用

- 未包含明确提交意图的动词（`保存`、`格式化`、`跑测试`）
- 修改中但用户只要预检/审查，没说要 commit
- 用户要求 push / PR / merge —— 另外的工作流，但本技能结束后不会自动 push（见安全红线 #8）

## 工作流程（8 步）

### 1. 摸清状态

```bash
git status --porcelain
git diff --cached --stat
git diff --stat              # 暂存区为空时追加看工作区
```

判定：**暂存区非空** → 直接使用；**暂存区为空** → 进入第 2 步。

### 2. 用户确认提交范围（仅当暂存区空）

用 `AskUserQuestion` 列出 `git status --short` 结果，让用户选择「全部 / 特定文件 / 取消」。**禁止自动 `git add -A` 或 `git add .`**（避免误提交 `.env`、凭证等，详见 `references/safety-rules.md#2`）。根据用户选择执行 `git add <显式文件列表>`。

### 3. 预检

优先调用预检脚本（自动探测 Rust/Node/Python/Go 并运行对应 fmt/lint + 通用敏感信息扫描）：

```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/git-commit/scripts/precheck.sh"
```

退出码：`0` 通过 / `1` 阻断（必须修复）/ `2` 告警（让用户确认）。

脚本不可用时的手动最小检查：

```bash
git diff --cached --check    # 空白符 / 冲突标记
git diff --cached | grep -En '(AKIA|sk-|-----BEGIN.*PRIVATE KEY)' || true
```

**命中敏感信息必须停下**，不要自作主张绕过（详见 `references/safety-rules.md#5--6`）。

### 4. 学习项目提交风格

```bash
git log --oneline -10
git log --format="%s" -20
```

分析：常用 type/scope、subject 语言（中/英）、是否带 body / footer。

### 5. 读取项目规范

按优先级查找（见 `references/commit-conventions.md#规范识别优先级推荐`）：

1. **Memory**：
   ```json
   {
     "name": "mcp__memory__open_nodes",
     "parameters": {"names": ["project:<repo>:commit-convention"]}
   }
   ```
   `<repo>` 取 `basename "$(git rev-parse --show-toplevel)"`。
2. **`.gitmessage` 模板**：`git config --get commit.template`
3. **`.commitlintrc*` / `commitlint.config.*`**：若存在默认套用 Conventional Commits
4. **历史推断**：第 4 步的结论
5. **兜底**：简洁描述格式

### 6. 生成提交信息

使用 `mcp__sequential-thinking__sequentialthinking`（6–8 步）分析 `git diff --cached`：

1. 识别变更目的（新功能 / 修复 / 重构 / 文档 ...）
2. 提取关键文件与模块
3. 匹配第 5 步确定的规范
4. 起草 subject（简洁、祈使句、长度合规）
5. 若变更跨多点，补充 body 列点
6. 校验是否命中规范要求的字段

**不要**在消息里写「Co-Authored-By: Claude」、「Generated with ...」等广告语，除非项目规范明文要求。

### 7. 展示变更摘要 + 等待确认

```
📊 变更摘要
  - 修改文件：<数量>
  - +<新增> / -<删除>
  - 暂存区：<文件列表>

📝 提交信息
  <subject>

  <body，如有>

✅ 预检：<通过 / 有告警，详情…>

确认提交？回复"确认" / "改成：<新文案>" / "取消"。
```

### 8. 执行提交

**强制使用 HEREDOC** 以保留多行格式、规避 shell 转义：

```bash
git commit -m "$(cat <<'EOF'
<subject>

<body>
EOF
)"
```

`'EOF'` 带引号是关键——阻止 `$` / 反引号展开。

验证：

```bash
git log -1 --oneline
```

**不自动 `git push`**，除非用户消息中同时包含"提交"与"推送"（见 `references/safety-rules.md#8`）。

## 安全红线（简版，完整见 references/safety-rules.md）

| 红线 | 动作 |
|------|------|
| 只在用户明示要 commit 时才执行 | 模糊时用 `AskUserQuestion` 确认 |
| 禁止 `git add -A` / `.` / `--all` | 按显式文件名 stage |
| 禁止 `--no-verify` / `--no-gpg-sign` | 除非用户显式要求并已告知风险 |
| Hook 失败 → 新建 commit | **不要** `--amend` 覆盖上一次成功的提交 |
| 敏感文件名命中 → 停下询问 | `.env`、`*.pem`、`id_rsa*` 等 |
| diff 命中密钥正则 → 停下询问 | 真密钥拒绝；占位符需用户显式确认 |
| 不向 main/master 做破坏性操作 | `push --force`、`reset --hard` 等 |
| 提交信息用 HEREDOC + `'EOF'` | 不要用单行 `-m "..."` 处理多行 |

## 错误处理

| 场景 | 处理 |
|------|------|
| 暂存区为空且用户选"取消" | 回复"已取消提交"，不执行任何 git 命令 |
| 预检阻断 | 展示失败项，询问用户是修复还是放弃；不要绕过 |
| Hook 失败 | 读取报错 → 修复 → 重新 stage → **新建** commit |
| 签名失败 | 提示用户 `git config commit.gpgsign` 与密钥问题；禁止 `--no-gpg-sign` 绕过 |
| 提交信息不符合 commitlint | 按报错调整信息重试，不改 hook |

## 附加资源

### references/

- **[commit-conventions.md](references/commit-conventions.md)** — Conventional Commits / Angular / Gitmoji / 工单前缀四种规范 + FAQ
- **[memory-templates.md](references/memory-templates.md)** — 五种 Memory 配置模板（多语言/多格式）
- **[safety-rules.md](references/safety-rules.md)** — 完整安全红线详解与默认行为表

### examples/

- **[commit-workflow.md](examples/commit-workflow.md)** — 端到端样例：空暂存区选文件 → 预检 → HEREDOC commit → hook 失败重试 → 敏感文件分支

### scripts/

- **[precheck.sh](scripts/precheck.sh)** — 按项目类型自动探测的通用预检（Rust/Node/Python/Go）+ 敏感信息扫描；`--quick` 秒级模式跳过工具链仅做安全扫描

### 用户入口

- **[README.md](README.md)** — 面向使用者的快速入门

---

## 配置项目规范（一次性）

用户首次提供规范时保存到 Memory，供后续会话复用：

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:<repo>:commit-convention",
      "entityType": "convention",
      "observations": ["..."]
    }]
  }
}
```

完整模板见 [references/memory-templates.md](references/memory-templates.md)。
