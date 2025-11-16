# Git Commit 规范参考

本文档提供常见的提交信息规范、Memory 配置模板和常见问题解答。

## 📋 提交信息规范

### Conventional Commits

最流行的提交信息规范，广泛应用于开源项目。

#### 格式
```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Type 类型
| Type | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | feat(auth): 添加用户登录功能 |
| `fix` | 修复 bug | fix(api): 修复用户查询接口错误 |
| `docs` | 文档更新 | docs(readme): 更新安装说明 |
| `style` | 代码格式（不影响功能） | style(main): 统一代码缩进 |
| `refactor` | 重构（不改变功能） | refactor(config): 简化配置加载逻辑 |
| `perf` | 性能优化 | perf(query): 优化数据库查询性能 |
| `test` | 测试相关 | test(user): 添加用户模块单元测试 |
| `build` | 构建系统或依赖 | build(deps): 升级 tokio 到 1.35 |
| `ci` | CI 配置 | ci(github): 添加自动发布工作流 |
| `chore` | 其他杂项 | chore(release): 发布 v1.2.0 |
| `revert` | 回滚提交 | revert: 回滚 feat(auth) 提交 |

#### Scope（可选）
表示影响的模块或范围：
- `api` - API 接口
- `config` - 配置系统
- `cli` - 命令行界面
- `core` - 核心逻辑
- `docs` - 文档
- `test` - 测试

#### Subject
- 使用祈使句，现在时态："添加"而非"添加了"
- 首字母小写
- 结尾不加句号
- 简洁明了，不超过 50 字符

#### Body（可选）
- 详细说明变更的原因和影响
- 每行不超过 72 字符
- 可以包含多个段落

#### Footer（可选）
- Breaking Changes：`BREAKING CHANGE: 描述`
- 关闭 Issue：`Closes #123, #456`
- 引用相关 PR：`Refs #789`

#### 完整示例
```
feat(config): 添加 TOML 配置支持

实现了基于 TOML 格式的配置文件读取功能，替代原有的 JSON 格式。
主要改进：
- 更友好的配置语法
- 支持注释
- 更好的类型安全

BREAKING CHANGE: 配置文件格式从 JSON 改为 TOML
Closes #42
```

### Angular 规范

Angular 团队使用的提交规范，与 Conventional Commits 类似。

#### 格式
```
<type>(<scope>): <subject>
<BLANK LINE>
<body>
<BLANK LINE>
<footer>
```

#### Type 类型
- `feat`: 新功能
- `fix`: 修复
- `docs`: 文档
- `style`: 格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

#### 示例
```
fix(compiler): 修复模板解析错误

当模板包含特殊字符时，解析器会抛出异常。
现在正确处理所有 Unicode 字符。

Fixes #1234
```

### 简洁描述格式

适合小型项目或个人项目的简单格式。

#### 格式
```
简短描述（不超过 50 字符）

可选的详细说明
```

#### 示例
```
添加用户认证功能

实现了基于 JWT 的用户认证系统
```

### Gitmoji 规范

使用 emoji 表示提交类型，视觉化更强。

#### 常用 Emoji
| Emoji | 代码 | 说明 |
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

#### 示例
```
✨ 添加用户登录功能
🐛 修复配置文件解析错误
📝 更新 API 文档
```

## 🔧 Memory 配置模板

### Conventional Commits 配置

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:ccode:commit-convention",
      "entityType": "convention",
      "observations": [
        "使用 Conventional Commits 格式",
        "格式：<type>(<scope>): <subject>",
        "type 包括：feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert",
        "scope 为可选，表示影响的模块",
        "subject 使用中文，祈使句，首字母小写",
        "body 可选，详细说明变更原因",
        "footer 可选，包含 BREAKING CHANGE 或关闭的 Issue"
      ]
    }]
  }
}
```

### 自定义规范配置

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:myproject:commit-convention",
      "entityType": "convention",
      "observations": [
        "格式：[模块名] 简短描述",
        "模块名使用大写，如：[API]、[UI]、[DB]",
        "描述使用中文，简洁明了",
        "示例：[API] 添加用户查询接口"
      ]
    }]
  }
}
```

### 团队规范配置

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:teamproject:commit-convention",
      "entityType": "convention",
      "observations": [
        "使用 Jira 工单号开头",
        "格式：[PROJ-123] 描述",
        "描述使用英文，动词开头",
        "示例：[PROJ-123] Add user authentication",
        "必须关联 Jira 工单"
      ]
    }]
  }
}
```

## ❓ 常见问题

### Q1: 如何修改已生成的提交信息？

**A**: 在技能展示提交信息并询问确认时，你可以：
```
你: "修改提交信息为：fix(api): 修复用户查询接口超时问题"
```

技能会使用你提供的信息进行提交。

### Q2: 如何跳过代码审查？

**A**: 技能默认进行简单审查。如果你确定代码没问题，可以：
```
你: "跳过审查，直接提交"
```

### Q3: 如何提交部分文件？

**A**: 当暂存区为空时，技能会询问你选择文件。你也可以提前使用 `git add`：
```bash
git add src/main.rs src/config.rs
```
然后请求提交，技能会只提交这些文件。

### Q4: 提交信息太长怎么办？

**A**: 技能会自动将详细内容放到 body 部分：
```
feat(config): 添加 TOML 配置支持

- 实现 TOML 配置读取
- 添加 .env 文件支持
- 更新配置迁移逻辑
- 完善测试用例
```

### Q5: 如何处理 Breaking Changes？

**A**: 在 Memory 配置中说明，技能会识别重大变更并添加 footer：
```
feat(api): 重构用户认证接口

BREAKING CHANGE: 认证接口从 /auth 改为 /api/v2/auth
```

### Q6: 提交失败怎么办？

**A**: 技能会显示错误信息。常见原因：
- pre-commit hook 失败 → 修复代码质量问题
- 提交信息格式不符 → 检查项目的 commitlint 配置
- Git 配置问题 → 检查 user.name 和 user.email

### Q7: 如何查看提交历史？

**A**: 技能会自动分析最近 20 条提交。你也可以手动查看：
```bash
git log --oneline -20
```

### Q8: 支持多语言提交信息吗？

**A**: 支持。技能会根据项目历史识别语言习惯：
- 中文项目 → 生成中文提交信息
- 英文项目 → 生成英文提交信息
- 混合项目 → 遵循 Memory 中的配置

### Q9: 如何更新项目规范？

**A**: 使用 Memory 工具更新：
```
你: "更新提交规范：subject 改用英文"
```

技能会更新 Memory 中的配置。

### Q10: 提交后发现错误怎么办？

**A**: 可以使用 `git commit --amend` 修改最后一次提交：
```bash
# 修改文件
git add .
git commit --amend
```

或者创建新的修复提交：
```
你: "提交修复"
```

## 📚 参考资源

### 官方文档
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Angular Commit Guidelines](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)
- [Gitmoji](https://gitmoji.dev/)

### 工具推荐
- [commitlint](https://commitlint.js.org/) - 提交信息校验
- [husky](https://typicode.github.io/husky/) - Git hooks 管理
- [commitizen](https://commitizen-tools.github.io/commitizen/) - 交互式提交

### 最佳实践
- [How to Write a Git Commit Message](https://cbea.ms/git-commit/)
- [Git Commit Best Practices](https://git-scm.com/book/en/v2/Distributed-Git-Contributing-to-a-Project)
- [Semantic Versioning](https://semver.org/)
