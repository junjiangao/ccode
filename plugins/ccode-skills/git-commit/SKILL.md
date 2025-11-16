---
name: git-commit
description: 智能 Git 提交助手：审查代码变更、分析提交历史、生成符合项目规范的提交信息并执行提交。支持自定义规范、提交前确认和智能暂存区处理。
---

# Git Commit 技能

智能化的 Git 提交工作流，集成代码审查、历史分析和规范化提交信息生成。

## 🎯 使用场景

### 触发条件
- 用户明确请求创建 Git 提交
- 关键词：`提交`、`commit`、`git commit`、`提交代码`
- 用户完成代码修改后需要提交变更

### 典型示例
```
用户: "帮我提交这些代码"
用户: "审查并提交当前的修改"
用户: "创建一个 commit"
用户: "把这些改动提交到 git"
```

## 📋 工作流程

### 1. 检查暂存区状态
```bash
# 检查暂存区
git status --porcelain
git diff --cached --stat

# 如果暂存区为空，列出工作区文件
git status --short
git diff --name-status
```

**决策逻辑**：
- **暂存区非空** → 直接使用暂存区文件
- **暂存区为空** → 列出工作区修改文件，询问用户确认提交范围

### 2. 用户确认（暂存区为空时）
**列出工作区文件**：
```bash
# 获取所有修改、新增、删除的文件
git status --short
# 输出格式：
#  M src/main.rs        (修改)
#  A src/config.rs      (新增)
#  D old/legacy.rs      (删除)
# ?? untracked.txt      (未跟踪)
```

使用 `AskUserQuestion` 工具询问：
- 提交所有修改的文件？
- 提交特定文件？（展示上述文件清单供选择）
- 取消提交？

根据用户选择执行 `git add` 操作。

### 3. 简单代码审查
检查项：
- 语法错误（通过 `cargo check`、`npm run lint` 等）
- 明显的逻辑问题
- 调试代码残留（console.log、println!、TODO 等）
- 敏感信息泄露（密钥、令牌、凭证）

**不执行深度审查**（不调用 Codex MCP），保持快速反馈。

### 4. 分析提交历史
```bash
git log --oneline -10
git log --format="%s" -20
```

**分析目标**：
- 识别项目的提交信息风格
- 提取常用的 type 和 scope
- 保持提交风格一致性

### 5. 读取项目提交规范
从 Memory 中查找项目级提交规范：
```
# 使用项目级精确查询，避免跨项目污染
mcp__memory__search_nodes(query: "project:<repo>:commit-convention")
mcp__memory__open_nodes(names: ["project:<repo>:commit-convention"])
```

**命名空间说明**：
- `<repo>` 应替换为实际的仓库名称（如 `ccode`、`myproject`）
- 使用 `project:` 前缀确保查询范围限定在当前项目
- 避免使用模糊查询（如 `"commit convention"`），防止匹配到其他项目的规范

**规范来源优先级**：
1. Memory 中的项目规范
2. 提交历史中的风格模式
3. 通用的简洁描述格式

### 6. 生成提交信息
使用 `mcp__sequential-thinking__sequentialthinking` 分析变更并生成提交信息：

**思考步骤**（6-8 步）：
1. 分析 `git diff --cached` 的变更内容
2. 识别变更的主要目的（新功能、修复、重构等）
3. 提取关键的文件和模块
4. 匹配项目提交规范格式
5. 生成简洁准确的提交信息
6. 验证提交信息是否符合规范

**提交信息格式**（根据项目规范）：
- 自定义规范：遵循 Memory 中的定义
- 无规范时：`<type>: <简洁描述>`

### 7. 展示变更摘要和提交信息
向用户展示：
```
📊 变更摘要：
  - 修改文件：3 个
  - 新增行：+45
  - 删除行：-12

📝 提交信息：
  feat(config): 添加 TOML 配置支持

  - 实现 TOML 配置读取
  - 添加 .env 文件支持
  - 更新配置迁移逻辑

✅ 代码审查：通过（无明显问题）

是否确认提交？
```

### 8. 执行提交
用户确认后执行：
```bash
git commit -m "$(cat <<'EOF'
<提交信息>
EOF
)"
```

验证提交成功：
```bash
git log -1 --oneline
```

## 🔧 MCP 工具调用

### Memory 工具
```json
{
  "name": "mcp__memory__search_nodes",
  "parameters": {
    "query": "project:<repo>:commit-convention"
  }
}
```

```json
{
  "name": "mcp__memory__open_nodes",
  "parameters": {
    "names": ["project:<repo>:commit-convention"]
  }
}
```

**注意**：`<repo>` 应替换为实际仓库名，如 `project:ccode:commit-convention`

### Sequential Thinking 工具
```json
{
  "name": "mcp__sequential-thinking__sequentialthinking",
  "parameters": {
    "thought": "分析 git diff 变更内容...",
    "thoughtNumber": 1,
    "totalThoughts": 6,
    "nextThoughtNeeded": true
  }
}
```

## 📝 协作模板

### 标准提交流程
```markdown
1. 检查暂存区状态（git status --porcelain, git diff --cached --stat）
2. 如果暂存区为空：
   - 使用 git status --short 列出所有工作区修改文件
   - 询问用户选择提交范围（全部/特定文件/取消）
   - 根据选择执行 git add
3. 简单代码审查（语法、明显问题）
4. 分析最近 20 条提交历史（git log --format="%s" -20）
5. 从 Memory 读取项目提交规范（使用 project:<repo>:commit-convention 精确查询）
6. 使用 Sequential Thinking 生成提交信息
7. 展示变更摘要和提交信息
8. 等待用户确认
9. 执行 git commit
10. 验证提交成功（git log -1 --oneline）
```

### 提交信息生成模板
```markdown
根据以下信息生成提交信息：

**变更内容**：
<git diff --cached 输出>

**项目规范**：
<Memory 中的规范或历史风格>

**要求**：
- 简洁准确，一行概括主要变更
- 如有多个变更，在正文中分点说明
- 遵循项目规范格式
- 避免技术细节，关注变更目的
```

## ⚙️ 配置项目提交规范

### 在 Memory 中记录规范
```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:<repo>:commit-convention",
      "entityType": "convention",
      "observations": [
        "使用 Conventional Commits 格式：<type>(<scope>): <subject>",
        "常用 type：feat, fix, docs, refactor, test, chore",
        "scope 为可选，表示影响的模块",
        "subject 使用中文，简洁描述变更",
        "正文可选，详细说明变更原因和影响"
      ]
    }]
  }
}
```

**命名规范**：
- 实体名称格式：`project:<repo>:commit-convention`
- `<repo>` 替换为实际仓库名（如 `ccode`、`myproject`）
- 使用 kebab-case 命名风格
- 确保命名空间隔离，避免跨项目污染

### 示例规范
```
type(scope): subject

body (可选)

footer (可选)
```

**type 类型**：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建/工具/依赖更新

## ✅ 最佳实践

### 推荐做法
1. **提交前审查**：始终检查 `git diff` 确认变更内容
2. **原子提交**：每次提交只包含一个逻辑变更
3. **清晰描述**：提交信息应让他人快速理解变更目的
4. **遵循规范**：保持项目提交风格一致
5. **及时提交**：完成一个功能点后立即提交

### 避免事项
1. **混合变更**：不要在一次提交中包含多个不相关的修改
2. **模糊描述**：避免 "update code"、"fix bug" 等无意义信息
3. **提交敏感信息**：检查是否包含密钥、令牌等
4. **跳过审查**：即使是小改动也应快速检查
5. **忽略规范**：不要随意偏离项目约定的格式

## 🚨 错误处理

### 暂存区为空且用户取消
```
暂存区为空，需要先添加文件。
已取消提交操作。
```

### 代码审查发现问题
```
⚠️ 代码审查发现以下问题：
  - src/main.rs:42: 包含 println! 调试代码
  - config/.env: 可能包含敏感信息

建议修复后再提交。是否继续？
```

### 提交失败
```
❌ 提交失败：<错误信息>

可能原因：
  - pre-commit hook 失败
  - 提交信息格式不符合要求
  - Git 配置问题

请检查错误信息并重试。
```

## 📚 相关资源

- [Conventional Commits 规范](https://www.conventionalcommits.org/)
- [Git 提交最佳实践](https://git-scm.com/book/zh/v2)
- [如何编写 Git 提交信息](https://cbea.ms/git-commit/)
