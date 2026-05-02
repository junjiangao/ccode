# 提交信息规范参考

通用 Git 提交信息规范、Memory 配置模板与常见问题解答。与具体语言/框架无关。

## 📋 主流规范

### Conventional Commits

最流行的提交规范，广泛用于开源项目与语义化发布工具链（semantic-release / commitlint）。

#### 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Type 类型

| Type | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat(auth): 添加用户登录功能` |
| `fix` | 修复 bug | `fix(api): 修复用户查询接口错误` |
| `docs` | 文档更新 | `docs(readme): 更新安装说明` |
| `style` | 代码格式（不影响功能） | `style(main): 统一代码缩进` |
| `refactor` | 重构（不改变功能） | `refactor(config): 简化配置加载逻辑` |
| `perf` | 性能优化 | `perf(query): 优化数据库查询性能` |
| `test` | 测试相关 | `test(user): 添加用户模块单元测试` |
| `build` | 构建系统或依赖 | `build(deps): 升级 tokio 到 1.35` |
| `ci` | CI 配置 | `ci(github): 添加自动发布工作流` |
| `chore` | 其他杂项 | `chore(release): 发布 v1.2.0` |
| `revert` | 回滚提交 | `revert: 回滚 feat(auth) 提交` |

#### Scope（可选）

表示影响的模块或范围。常见选择：
- 模块名：`api` / `cli` / `core` / `ui`
- 目录名：`src/auth` / `packages/server`
- 主题名：`config` / `docs` / `test`

若项目没有明显的模块划分，scope 可以省略。

#### Subject

- 使用祈使句、现在时态："添加"而非"添加了"
- 中文可不区分大小写；英文首字母小写
- 结尾不加句号
- 简洁明了，不超过 50 字符（英文）/ 25 汉字

#### Body（可选）

- 详细说明变更的原因和影响
- 每行不超过 72 字符
- 用空行分段；可用 `-` 列点

#### Footer（可选）

- Breaking Changes：`BREAKING CHANGE: 描述` 或 type 后加 `!`（如 `feat!:`）
- 关闭 Issue：`Closes #123, #456`
- 引用相关 PR：`Refs #789`

#### 完整示例

```
feat(config): 添加 TOML 配置支持

实现基于 TOML 格式的配置文件读取，替代原有的 JSON 格式。
主要改进：
- 更友好的配置语法
- 支持注释
- 更好的类型安全

BREAKING CHANGE: 配置文件格式从 JSON 改为 TOML
Closes #42
```

### Angular 规范

Angular 团队的提交规范，是 Conventional Commits 的前身，二者基本兼容。

#### 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Type 类型（较 CC 精简）

`feat` / `fix` / `docs` / `style` / `refactor` / `test` / `chore`

#### 示例

```
fix(compiler): 修复模板解析错误

当模板包含特殊字符时解析器会抛出异常。
现在正确处理所有 Unicode 字符。

Fixes #1234
```

### 简洁描述格式

适合个人项目、内部工具或 MVP 阶段。

#### 格式

```
简短描述（不超过 50 字符）

可选的详细说明
```

#### 示例

```
添加用户认证功能

实现基于 JWT 的用户认证系统
```

### Gitmoji

用 emoji 替代 type，视觉化更强，不适合需要自动化的工具链。

| Emoji | 代码 | 用途 |
|-------|------|------|
| ✨ | `:sparkles:` | 新功能 |
| 🐛 | `:bug:` | 修复 bug |
| 📝 | `:memo:` | 文档 |
| 🎨 | `:art:` | 代码格式/结构 |
| ⚡️ | `:zap:` | 性能优化 |
| 🔥 | `:fire:` | 删除代码/文件 |
| ✅ | `:white_check_mark:` | 测试 |
| 🔒 | `:lock:` | 安全修复 |
| ⬆️ | `:arrow_up:` | 升级依赖 |
| ⬇️ | `:arrow_down:` | 降级依赖 |

示例：

```
✨ 添加用户登录功能
🐛 修复配置文件解析错误
📝 更新 API 文档
```

### 工单前缀格式

企业内部常见，要求每个提交关联一个工单。

#### 格式

```
[PROJ-123] 描述

body 可选
```

示例：

```
[PROJ-512] Add user authentication endpoint

Introduces JWT-based login with refresh tokens.
Refs: PROJ-512
```

---

## 规范识别优先级（推荐）

技能按以下顺序判定项目遵循哪种规范：

1. **Memory 项目规范**：`mcp__memory__open_nodes(["project:<repo>:commit-convention"])`
2. **`.gitmessage` 模板**：`git config --get commit.template` 指向的文件
3. **`.commitlintrc*` / `commitlint.config.*`**：若存在通常意味着 Conventional Commits
4. **提交历史风格推断**：最近 20 条提交的公共结构
5. **默认兜底**：简洁描述格式

发现多个来源冲突时，以上层为准。

---

## ❓ 常见问题

### Q1：如何修改技能生成的提交信息？

在确认阶段直接说出想要的文案：

```
用户: "改成：fix(api): 修复用户查询接口超时问题"
```

技能应改用你给出的文案执行 `git commit`。

### Q2：如何跳过代码审查 / 预检？

明确表达意图：

```
用户: "跳过审查，直接提交"
```

但即便如此，仍应保留敏感文件扫描（避免提交密钥）。

### Q3：如何提交部分文件？

两种方式：

1. 先手动 `git add`，技能检测到暂存区非空时直接使用：
   ```bash
   git add src/main.rs src/config.rs
   ```
2. 让技能询问：暂存区为空时技能会列出变更文件清单，让你选择。

### Q4：提交信息太长怎么办？

技能自动把详细内容放到 body，且每行不超过 72 字符：

```
feat(config): 添加 TOML 配置支持

- 实现 TOML 配置读取
- 添加 .env 文件支持
- 更新配置迁移逻辑
- 完善测试用例
```

### Q5：如何处理 Breaking Changes？

在 Memory 或 `.gitmessage` 中声明，技能识别后会添加 footer：

```
feat(api)!: 重构用户认证接口

BREAKING CHANGE: 认证接口从 /auth 改为 /api/v2/auth
```

### Q6：提交失败怎么办？

常见原因与应对：

| 原因 | 现象 | 处理 |
|------|------|------|
| pre-commit hook 失败 | 非零退出码 + 脚本输出 | **修复问题后新建 commit**，不要 `--amend`（`--amend` 会覆盖上一个已成功的 commit） |
| 提交信息格式不符 | commitlint 报错 | 检查 `.commitlintrc` 约束，重新生成信息 |
| Git 身份未配置 | "Please tell me who you are" | `git config user.name/user.email` |
| 签名失败 | GPG / SSH 签名错误 | 检查 `commit.gpgsign` 与密钥；**不要**加 `--no-gpg-sign` 除非用户明确要求 |

### Q7：如何查看提交历史？

技能自动分析最近 20 条提交学习风格。手动查看：

```bash
git log --oneline -20
git log --format="%s" -20   # 仅 subject
```

### Q8：支持多语言提交信息吗？

支持。技能按以下顺序决定语言：

1. Memory 中 `commit-convention` 观察项（如 `"subject 使用中文"`）
2. 最近 20 条提交的主要语言（统计中/英字符占比）
3. 项目 README 语言
4. 默认中文

### Q9：如何更新项目规范？

告诉技能新的规范，技能通过 `mcp__memory__add_observations` 更新同名实体：

```
用户: "更新提交规范：subject 改用英文"
```

### Q10：提交后发现错误怎么办？

**仅当该提交尚未 push** 时可以安全修改：

```bash
# 修改最后一次提交
git add <修正的文件>
git commit --amend
```

**若已 push 到远端**：不要重写历史（除非是你自己的分支且与协作者确认过）。更安全的做法是**追加修复提交**：

```
用户: "再提交一个修复"
```

---

## 📚 参考资源

### 官方文档

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Angular Commit Guidelines](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)
- [Gitmoji](https://gitmoji.dev/)

### 配套工具

- [commitlint](https://commitlint.js.org/) — 提交信息校验
- [husky](https://typicode.github.io/husky/) — Git hooks 管理
- [commitizen](https://commitizen-tools.github.io/commitizen/) — 交互式提交
- [semantic-release](https://semantic-release.gitbook.io/) — 基于 Conventional Commits 的自动发布

### 最佳实践读物

- [How to Write a Git Commit Message](https://cbea.ms/git-commit/)
- [Git Commit Best Practices (Pro Git)](https://git-scm.com/book/en/v2/Distributed-Git-Contributing-to-a-Project)
- [Semantic Versioning](https://semver.org/)

---

**关联文档**：
- Memory 模板：[memory-templates.md](memory-templates.md)
- 安全红线：[safety-rules.md](safety-rules.md)
- 端到端示例：[../examples/commit-workflow.md](../examples/commit-workflow.md)
- 技能定义：[../SKILL.md](../SKILL.md)
