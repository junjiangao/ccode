# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

`ccode` 是一个用 Rust 编写的 Claude Code 环境管理工具，采用创新的双模式架构设计：

### 🎯 核心架构

- **Direct 模式**：传统的简单API配置方式（向后兼容）
  - 直接配置 ANTHROPIC_AUTH_TOKEN 和 ANTHROPIC_BASE_URL
  - 适合单一API服务的简单切换需求
  - 零学习成本，即插即用

- **CCR 模式**：集成 Claude Code Router 的智能路由系统（新特性）
  - 多Provider支持：OpenRouter、DeepSeek、Gemini、Qwen等
  - 智能路由：根据任务类型自动选择最适合的模型
  - 负载均衡：多模型间的智能分配和容灾
  - 成本优化：后台任务使用高性价比模型，推理任务使用强推理模型

### 🚀 CCR模式优势

1. **智能场景路由**
   - `default`: 日常任务的平衡选择
   - `background`: 后台任务的经济型模型
   - `think`: 推理密集任务的强推理模型
   - `longContext`: 长上下文的大窗口模型
   - `webSearch`: 网络搜索的专用模型

2. **企业级特性**
   - 多Provider容灾和高可用
   - 请求转换适配不同API格式
   - 服务管理和监控
   - 配置热更新和版本管理

3. **开发体验**
   - 自动依赖管理（npm包安装）
   - 交互式配置向导
   - 智能模型推荐
   - 完整的CLI工具链

## 开发命令

### 构建和测试
```bash
# 开发构建
cargo build

# 生产构建
cargo build --release

# 运行测试
cargo test

# 运行特定测试
cargo test test_name
```

### 代码质量检查
```bash
# 代码格式检查
cargo fmt --check

# 代码格式化
cargo fmt

# 代码质量检查（要求零警告）
cargo clippy -- -D warnings

# 安全漏洞扫描
cargo audit

# CI流程完整检查
cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo build --release
```

### 安装和运行
```bash
# 安装到系统
cargo install --path .

# 或者直接运行
cargo run -- <subcommand>
```

## 项目架构

### 技术栈
- **语言**：Rust 2024 Edition (最低要求 Rust 1.70+)
- **CLI框架**：clap 4.x (derive API)
- **异步运行时**：tokio (用于CCR管理)
- **HTTP客户端**：reqwest (用于CCR API交互)
- **序列化**：serde + serde_json
- **系统信息**：sysinfo (进程管理)
- **其他**：dirs (跨平台目录)、chrono (时间戳)、anyhow (错误处理)

### 核心模块结构

```
src/
├── main.rs          # CLI入口，命令路由和参数解析
├── commands.rs      # 所有命令的具体实现逻辑
├── config.rs        # 配置文件管理和数据结构定义
├── ccr_config.rs    # CCR配置文件直接管理器
├── ccr_manager.rs   # CCR服务生命周期管理
├── error.rs         # 统一错误处理
└── lib.rs           # 库入口，模块导出
```

### 配置系统架构

配置文件位置：`~/.config/ccode/config.json` (Linux/macOS) 或 `%APPDATA%/ccode/config.json` (Windows)

配置结构：
- **Direct组** (`groups.direct`): 简单的 token + base_url 配置
- **CCR组** (`groups.ccr`): 复杂的多提供商路由配置
- **默认配置** (`default_profile`): 分别为两组设置默认配置

### CCR集成架构

CCR (Claude Code Router) 通过以下方式集成：
- **依赖管理**：自动检查和安装 `@musistudio/claude-code-router` npm包
- **配置生成**：动态生成 `~/.claude-code-router/config.json`
- **服务管理**：启动/停止/重启CCR服务，监听端口3456
- **代理模式**：将claude请求路由到 `http://localhost:3456`

## 命令组织模式

### 统一接口命令（支持 --group 参数）
- `list --group direct|ccr` - 列出指定组配置
- `add --group direct|ccr <name>` - 添加配置到指定组
- `use --group direct|ccr <name>` - 设置指定组默认配置
- `run --group direct|ccr [name]` - 运行指定组配置
- `remove --group direct|ccr <name>` - 删除指定组配置

### CCR专用快捷命令
- `add-ccr <name>` - 快速添加CCR配置
- `run-ccr [name]` - 快速运行CCR配置
- `list-ccr` - 列出CCR配置
- `use-ccr <name>` - 设置默认CCR配置
- `remove-ccr <name>` - 删除CCR配置

### CCR服务管理
- `ccr start` - 启动CCR服务
- `ccr stop` - 停止CCR服务
- `ccr restart` - 重启CCR服务
- `ccr status` - 查看服务状态
- `ccr logs` - 查看服务日志

### Provider管理（新增）
- `provider list` - 列出所有Providers
- `provider add <name>` - 添加新Provider
- `provider remove <name>` - 删除Provider
- `provider show <name>` - 显示Provider详情
- `provider edit <name>` - 编辑Provider配置

## 开发注意事项

### Rust代码风格规范

#### 字符串格式化
使用内联形式的字符串格式化（clippy: uninlined_format_args）：
- ✅ 正确：`format!("Hello {name}")`, `println!("Value: {value}")`  
- ❌ 错误：`format!("Hello {}", name)`, `println!("Value: {}", value)`

#### 代码简化规则
- 使用 `is_some_and()` 代替 `map_or(false, |x| condition)` (clippy: unnecessary_map_or)
- 使用 `is_none_or()` 代替 `map_or(true, |x| condition)` (clippy: unnecessary_map_or)  
- 避免不必要的 `to_string()` 调用 (clippy: unnecessary_to_owned)
- 优先使用 `?` 操作符进行错误传播

#### 常见clippy问题和解决方案

##### 类型借用和集合兼容性问题
问题：HashSet 类型不匹配导致的借用错误
```rust
// ❌ 错误：类型不匹配
let provider_names: HashSet<&String> = self.providers.iter().map(|p| &p.name).collect();
if !provider_names.contains(provider_name) { ... } // provider_name是&str

// ✅ 正确：统一使用&str类型
let provider_names: HashSet<_> = self.providers.iter().map(|p| p.name.as_str()).collect();
if !provider_names.contains(provider_name) { ... }
```

##### 死代码处理
对于完整但暂未使用的API方法，使用`#[allow(dead_code)]`标注：
```rust
#[allow(dead_code)]
pub fn backup_management_method(&self) -> AppResult<()> {
    // 完整的备份管理API，虽然CLI暂未使用但应保留
}
```

##### Option链式调用优化
```rust
// ❌ 错误：不必要的map_or使用
self.background.as_ref().map_or(false, |s| s.is_empty())
self.background.as_ref().map_or(true, |s| s.is_empty())

// ✅ 正确：使用专用方法
self.background.as_ref().is_some_and(|s| s.is_empty())
self.background.as_ref().is_none_or(|s| s.is_empty())
```

#### 代码质量要求
- 项目要求零 clippy 警告：`cargo clippy -- -D warnings`
- 所有代码必须通过 `cargo fmt` 格式化检查
- 使用 `cargo audit` 进行安全漏洞扫描
- 修复clippy警告后，必须运行完整CI检查：`cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo build --release`

#### Git提交流程要求
**IMPORTANT: 提交代码前必须执行格式化**
```bash
# 每次git提交前必须执行以下命令
cargo fmt

# 然后进行git提交
git add .
git commit -m "你的提交信息"
```

**完整的提交前检查流程**：
```bash
# 1. 格式化代码（必须）
cargo fmt

# 2. 检查代码质量（推荐）
cargo clippy -- -D warnings

# 3. 运行测试（推荐）
cargo test

# 4. 提交代码
git add .
git commit -m "feat: 添加新功能或fix: 修复问题"
```

**原因**：
- 确保所有提交的代码都有统一的格式风格
- 避免因格式问题导致的CI/CD构建失败
- 减少code review中的格式相关讨论
- 保持代码库的整体一致性

### 错误处理模式
使用 `anyhow::Result<T>` 作为统一的错误返回类型（别名为 `AppResult<T>`），所有错误通过 `AppError` 枚举统一处理。

### 异步编程模式
CCR相关功能使用异步编程，通过 `tokio::runtime::Runtime::new()?.block_on()` 在同步主函数中运行异步代码。

### 配置向后兼容
支持从旧版本配置格式自动迁移，保持向后兼容性。

### 交互式输入模式
添加配置时使用标准输入进行交互式配置，支持可选字段（如描述）。

### 系统集成要求
- 要求系统已安装 claude CLI 工具
- CCR模式需要 npm/pnpm 环境支持
- 支持跨平台目录结构

## CI/CD流程

项目使用GitHub Actions进行自动化：
- **代码质量**：rustfmt + clippy + 测试
- **跨平台构建**：Linux (Ubuntu 22.04 LTS)、Windows、macOS
- **安全扫描**：cargo-audit 自动检查
- **自动发布**：基于git tag创建发布包