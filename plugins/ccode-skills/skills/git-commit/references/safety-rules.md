# Git 提交安全红线

这份清单给 `git-commit` 技能（也适用于人工提交）提供通用、与具体项目无关的安全底线。默认遵守；用户显式例外时才能绕过，并在交付总结中说明。

## 🔒 红线清单（绝不跨越）

### 1. 只在用户明确要求时才 commit

- **原则**：`git commit` 是一个会进入历史的持久化动作，不可当作"顺手一起做"的副作用。
- **允许触发词**：`提交` / `commit` / `git commit` / `提交代码` / `把这些改动提交到 git` / `帮我提交`。
- **不应触发**：`保存` / `build` / `跑测试` / `修一下` —— 这些动词不包含"提交"意图。
- **处理不确定情况**：用 `AskUserQuestion` 确认，不要默认 commit。

### 2. 禁止宽泛 staging

- ❌ `git add -A` / `git add .` / `git add --all` / `git add *`
- ✅ `git add <显式文件 1> <显式文件 2> ...`
- 原因：宽泛 staging 容易把 `.env`、`secrets/`、凭证文件、IDE 本地配置、日志、`.DS_Store` 等误提交。
- 若必须批量添加：先 `git status --short` 列清单，用 `AskUserQuestion` 让用户勾选，再逐个 `git add`。

### 3. 禁止跳过 hooks / 签名

- ❌ `git commit --no-verify`
- ❌ `git commit --no-gpg-sign`
- ❌ `git -c commit.gpgsign=false commit ...`
- 仅在用户**明确要求跳过**时才允许，且必须先告知风险。
- Hook 失败几乎总是发现了真问题（格式、类型、测试、密钥扫描）；先修再提，不要绕过。

### 4. Hook 失败时新建 commit，不要 `--amend`

- 当 pre-commit hook 失败，**commit 没有生成**。
- 若此时用 `--amend`，会误改"上一个已成功的 commit"（往往是前序工作），造成历史污染甚至丢失变更。
- 正确流程：
  1. 阅读 hook 报错
  2. 修复问题
  3. `git add <修正后的文件>`
  4. `git commit`（新建，不要 `--amend`）

### 5. 敏感文件名白名单式检查

在 staging 前对以下模式做严格排查，命中即停下向用户确认：

| 模式 | 风险 |
|------|------|
| `.env`, `.env.*`（除 `.env.example` 外） | 环境变量 / 密钥 |
| `secrets/`, `credentials*`, `*credentials.json` | 凭证文件 |
| `*.pem`, `*.key`, `*.p12`, `*.pfx`, `id_rsa*`, `id_ed25519*` | 私钥 |
| `*.keystore`, `*.jks` | Java keystore |
| `.aws/`, `.ssh/`, `.gnupg/` | 账号配置 |
| `*.sqlite`, `*.db` (未显式 ignore 的) | 数据快照可能含 PII |
| `nohup.out`, `*.log` | 日志可能泄露 token |

### 6. 密钥字面量扫描（diff 级）

对 `git diff --cached` 做正则扫描，命中即停：

- 云厂商 key：`AKIA[0-9A-Z]{16}`、`AIza[0-9A-Za-z_-]{35}`、`sk-[A-Za-z0-9]{20,}`
- Slack/GitHub token：`xoxb-`、`ghp_[A-Za-z0-9]{36}`、`github_pat_`
- JWT 开头：`eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.`
- 显式字段名：`password\s*[:=]`、`api[_-]?key\s*[:=]`、`secret\s*[:=]`
- 私钥头：`-----BEGIN (RSA|EC|OPENSSH|PGP) PRIVATE KEY-----`

命中后：
1. 用 `AskUserQuestion` 询问是"真密钥"还是"占位符/测试数据"。
2. 若是真密钥：拒绝提交，提示用户撤销 staging 并立即轮换密钥。
3. 若是占位符：让用户确认"我知道这是占位符，继续提交"再执行。

### 7. 不对 main/master 做破坏性操作

- ❌ `git push --force` 到 `main` / `master` / `release/*`
- ❌ `git reset --hard` 覆盖他人提交
- ❌ `git branch -D <共享分支>`
- 若用户请求这些操作：先警告后果，显式二次确认，并**永远不要自动执行**。

### 8. 不自动 push

- 完成 `git commit` 后**不要**自动 `git push`。
- 除非用户消息中同时包含"提交"和"推"（例如"提交并推送"）。
- 理由：push 是跨网络的不可完全回退操作，blast radius 更大。

### 9. 提交信息使用 HEREDOC

- ✅
  ```bash
  git commit -m "$(cat <<'EOF'
  feat(scope): subject

  body line 1
  body line 2
  EOF
  )"
  ```
- ❌ `git commit -m "feat: ..."`（单行、多行都容易被 shell 转义吞掉引号/反引号）
- HEREDOC 好处：保留换行、禁用变量展开（`'EOF'` 带引号）、避免转义地狱。

### 10. 不隐瞒失败

- 若 `git commit` 失败，向用户完整复述错误输出，不要美化。
- 不要伪造"已提交"的假消息；`git log -1 --oneline` 验证是交付前的最后一步。

## 建议默认行为

| 场景 | 默认 | 例外触发 |
|------|------|---------|
| staging 空 + 有工作区变更 | `AskUserQuestion` 列文件让用户选 | 用户说"提交所有" |
| 发现敏感文件名 | 停下询问 | 用户明确说"是占位符" |
| diff 命中密钥正则 | 停下询问 | 同上 |
| pre-commit hook 失败 | 读取报错 → 修复 → 新建 commit | 用户要求 `--no-verify`（仍需提醒风险） |
| 提交后 | 停在 `git log -1` 汇报 | 用户同时要求了 push |

---

**关联文档**：
- 规范参考：[commit-conventions.md](commit-conventions.md)
- Memory 模板：[memory-templates.md](memory-templates.md)
- 端到端示例：[../examples/commit-workflow.md](../examples/commit-workflow.md)
- 预检脚本：[../scripts/precheck.sh](../scripts/precheck.sh)
- 技能定义：[../SKILL.md](../SKILL.md)
