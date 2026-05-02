# Memory 配置模板

用于把项目提交规范持久化到 MCP Memory，供 `git-commit` 技能后续会话自动加载。

## 命名规范

所有模板统一使用：

```
project:<repo>:commit-convention
```

- `<repo>` 为仓库名（仅一个 token，如 `ccode` / `myproject`）。
- 前缀 `project:` 确保命名空间隔离，避免跨项目污染。
- 全小写 kebab-case，便于 `search_nodes` 精确匹配。

## 模板 1：Conventional Commits（中文 subject）

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:<repo>:commit-convention",
      "entityType": "convention",
      "observations": [
        "使用 Conventional Commits 格式",
        "格式：<type>(<scope>): <subject>",
        "type 包括：feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert",
        "scope 为可选，表示影响的模块",
        "subject 使用中文，祈使句，首字母小写",
        "body 可选，详细说明变更原因",
        "footer 可选，包含 BREAKING CHANGE 或关闭的 Issue",
        "body 与 subject 之间必须有空行"
      ]
    }]
  }
}
```

## 模板 2：Conventional Commits（英文 subject）

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:<repo>:commit-convention",
      "entityType": "convention",
      "observations": [
        "Follow Conventional Commits specification",
        "Format: <type>(<scope>): <subject>",
        "Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert",
        "Subject in English, imperative mood, no trailing period",
        "Subject under 50 characters",
        "Body wrapped at 72 characters",
        "Breaking changes marked with `!` after type or `BREAKING CHANGE:` footer"
      ]
    }]
  }
}
```

## 模板 3：自定义简洁格式

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:<repo>:commit-convention",
      "entityType": "convention",
      "observations": [
        "格式：[模块名] 简短描述",
        "模块名使用大写，如：[API]、[UI]、[DB]",
        "描述使用中文，简洁明了",
        "示例：[API] 添加用户查询接口",
        "不使用 body / footer；需要详细说明时开 PR 讨论"
      ]
    }]
  }
}
```

## 模板 4：工单前缀 + 英文正文

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:<repo>:commit-convention",
      "entityType": "convention",
      "observations": [
        "每个提交必须关联 Jira 工单",
        "格式：[PROJ-123] Verb-first English description",
        "subject 使用英文，动词开头，首字母大写",
        "示例：[PROJ-512] Add user authentication endpoint",
        "footer 使用 `Refs: PROJ-123` 显式标注工单号"
      ]
    }]
  }
}
```

## 模板 5：Gitmoji（视觉化）

```json
{
  "name": "mcp__memory__create_entities",
  "parameters": {
    "entities": [{
      "name": "project:<repo>:commit-convention",
      "entityType": "convention",
      "observations": [
        "使用 Gitmoji 视觉化格式",
        "格式：<emoji> 简短描述",
        "常用 emoji: ✨ 新功能 / 🐛 修复 / 📝 文档 / ♻️ 重构 / ⚡️ 性能 / ✅ 测试",
        "subject 使用中文",
        "不自动化发布；仅供人类阅读"
      ]
    }]
  }
}
```

## 更新既有规范

需要修订时用 `add_observations` 追加观察项，而非删除重建：

```json
{
  "name": "mcp__memory__add_observations",
  "parameters": {
    "observations": [{
      "entityName": "project:<repo>:commit-convention",
      "contents": [
        "2026-04 更新：subject 改用英文"
      ]
    }]
  }
}
```

需要删除过时观察项时用 `delete_observations`：

```json
{
  "name": "mcp__memory__delete_observations",
  "parameters": {
    "deletions": [{
      "entityName": "project:<repo>:commit-convention",
      "observations": ["subject 使用中文，祈使句，首字母小写"]
    }]
  }
}
```

## 查询规范

首选精确查询：

```json
{
  "name": "mcp__memory__open_nodes",
  "parameters": {
    "names": ["project:<repo>:commit-convention"]
  }
}
```

若不确定名称，退回到模糊查询（但要加 `project:` 前缀以防止匹配其他项目）：

```json
{
  "name": "mcp__memory__search_nodes",
  "parameters": {
    "query": "project:<repo>:commit"
  }
}
```

查询失败时 fallback 顺序见 [commit-conventions.md#规范识别优先级推荐](commit-conventions.md#规范识别优先级推荐)。

---

**关联文档**：
- 规范参考：[commit-conventions.md](commit-conventions.md)
- 安全红线：[safety-rules.md](safety-rules.md)
- 技能定义：[../SKILL.md](../SKILL.md)
