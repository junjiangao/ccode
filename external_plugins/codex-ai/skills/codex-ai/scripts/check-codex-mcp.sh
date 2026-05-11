#!/usr/bin/env bash
# Codex-AI 诊断脚本
#
# 用法:
#   bash scripts/check-codex-mcp.sh
#   bash "${CLAUDE_PLUGIN_ROOT}/skills/codex-ai/scripts/check-codex-mcp.sh"
#
# 作用: 快速诊断 Codex CLI 与 codex-ai 插件所需环境是否就绪。
# 仅做只读检查,不改动任何配置。

set -u

PASS='[  OK  ]'
WARN='[ WARN ]'
FAIL='[ FAIL ]'

say() { printf '%s %s\n' "$1" "${2:-}"; }
hint() { printf '         %s\n' "$1"; }

status_ok=true
warn_count=0

# --- 1) Codex CLI 是否存在 ---------------------------------------------------
if command -v codex >/dev/null 2>&1; then
  ver="$(codex --version 2>&1 | head -n1 || true)"
  say "$PASS" "codex CLI 已安装: ${ver:-未知版本}"
else
  say "$FAIL" "未找到 codex CLI (PATH 中)"
  hint "安装指引: https://github.com/openai/codex"
  status_ok=false
fi

# --- 2) Codex 配置目录 -------------------------------------------------------
codex_home="${CODEX_HOME:-$HOME/.config/codex}"
if [ -d "$codex_home" ]; then
  say "$PASS" "配置目录存在: $codex_home"
  if [ -f "$codex_home/config.toml" ]; then
    say "$PASS" "config.toml 存在"
  else
    say "$WARN" "config.toml 缺失 (Codex 首次运行时会生成)"
    warn_count=$((warn_count + 1))
  fi
  if [ -f "$codex_home/auth.json" ] || [ -f "$codex_home/credentials.json" ]; then
    say "$PASS" "检测到鉴权文件"
  else
    say "$WARN" "未检测到鉴权文件,可能需要 'codex login'"
    warn_count=$((warn_count + 1))
  fi
else
  say "$WARN" "未找到 Codex 配置目录: $codex_home"
  hint "首次运行 codex 将自动创建"
  warn_count=$((warn_count + 1))
fi

# --- 3) Claude Code marketplace.json 是否注册 codex-ai -------------------------
# 尝试若干常见位置
marketplace_candidates=(
  "${CLAUDE_PLUGIN_ROOT:-}"/../../../.claude-plugin/marketplace.json
  "$PWD/.claude-plugin/marketplace.json"
  "$HOME/.claude/plugins/marketplace.json"
)

found_marketplace=""
for path in "${marketplace_candidates[@]}"; do
  [ -z "$path" ] && continue
  if [ -f "$path" ]; then
    found_marketplace="$path"
    break
  fi
done

if [ -n "$found_marketplace" ]; then
  if grep -q '"codex-ai"' "$found_marketplace" 2>/dev/null; then
    say "$PASS" "marketplace.json 已注册 codex-ai ($found_marketplace)"
  else
    say "$WARN" "marketplace.json 未注册 codex-ai ($found_marketplace)"
    warn_count=$((warn_count + 1))
  fi
else
  say "$WARN" "未能在常见位置找到 marketplace.json"
  warn_count=$((warn_count + 1))
fi

# --- 4) 外部插件目录是否存在 --------------------------------------------------
# 基于脚本自身位置反推项目根目录: scripts/../../../../
script_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
# scripts -> codex-ai -> skills -> codex-ai -> external_plugins -> repo root
repo_root="$(cd "$script_dir/../../../../.." 2>/dev/null && pwd || echo "")"
if [ -n "$repo_root" ] && [ -d "$repo_root/external_plugins/codex-ai" ]; then
  say "$PASS" "external_plugins/codex-ai 存在"
else
  say "$WARN" "未找到 external_plugins/codex-ai (非 ccode 仓库内运行时可忽略)"
  warn_count=$((warn_count + 1))
fi

# --- 5) 最小端到端连通测试(可选) ----------------------------------------------
if "$status_ok" && command -v codex >/dev/null 2>&1; then
  say "$PASS" "基础工具链就绪"
  echo
  echo "下一步可选验证:"
  echo "  codex exec -m gpt-5.3-codex \"return the string 'ok'\""
  echo "若上条命令在 10 秒内返回 'ok',则 Codex CLI 路径畅通。"
  echo "MCP 层的真正连通性,请在 Claude Code 里发起一次最小 MCP 调用验证。"
fi

echo
if ! "$status_ok"; then
  say "$FAIL" "诊断未通过: 请先解决上面的 [FAIL] 项"
  exit 1
elif [ "$warn_count" -gt 0 ]; then
  say "$WARN" "诊断完成,有 $warn_count 条警告可按需处理"
  exit 0
else
  say "$PASS" "全部就绪"
  exit 0
fi
