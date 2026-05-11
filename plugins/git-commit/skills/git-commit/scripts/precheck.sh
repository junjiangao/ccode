#!/usr/bin/env bash
# git-commit 技能 · 提交前预检脚本
#
# 用法:
#   bash scripts/precheck.sh            # 跑所有可用检查
#   bash scripts/precheck.sh --quick    # 只做安全扫描,跳过 fmt/lint (秒级)
#   bash scripts/precheck.sh --no-fmt   # 跳过格式化相关检查
#   bash scripts/precheck.sh --no-lint  # 跳过 lint
#
# 特性:
# - 自动探测项目类型 (Cargo.toml / package.json / pyproject.toml / go.mod 等),
#   仅运行探测到的工具链;缺少某工具时降级为 WARN,不会让脚本失败.
# - 通用安全扫描:暂存区敏感文件名 + diff 密钥字面量扫描.
# - 退出码 0 = 可以提交;1 = 有阻断问题;2 = 用户需要确认 (有 WARN).

set -u

MODE_QUICK=false
RUN_FMT=true
RUN_LINT=true

for arg in "$@"; do
  case "$arg" in
    --quick)   MODE_QUICK=true ;;
    --no-fmt)  RUN_FMT=false ;;
    --no-lint) RUN_LINT=false ;;
    -h|--help)
      sed -n '3,10p' "$0"
      exit 0
      ;;
    *) ;;
  esac
done

PASS='[  OK  ]'
WARN='[ WARN ]'
FAIL='[ FAIL ]'
SKIP='[ SKIP ]'

say()  { printf '%s %s\n' "$1" "${2:-}"; }
hint() { printf '         %s\n' "$1"; }

blocker=0
warnings=0

# --- 0) 必要前置: 必须在 git 仓库内 ---------------------------------------
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  say "$FAIL" "当前目录不是 git 仓库"
  exit 1
fi
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

staged_files="$(git diff --cached --name-only 2>/dev/null || true)"
if [ -z "$staged_files" ]; then
  say "$WARN" "暂存区为空;仅运行仓库级检查"
  warnings=$((warnings + 1))
fi

# --- 1) 敏感文件名扫描 (始终执行) ----------------------------------------
sensitive_patterns=(
  '(^|/)\.env($|\.)'          # .env / .env.local 等 (但保留 .env.example)
  '(^|/)secrets/'
  '(^|/)credentials(\.|$)'
  '.*credentials\.json$'
  '.*\.pem$'
  '.*\.key$'
  '.*\.p12$'
  '.*\.pfx$'
  '(^|/)id_rsa'
  '(^|/)id_ed25519'
  '.*\.keystore$'
  '.*\.jks$'
  '(^|/)\.aws/'
  '(^|/)\.ssh/'
  '(^|/)\.gnupg/'
)

sensitive_hits=""
if [ -n "$staged_files" ]; then
  while IFS= read -r file; do
    # 放行 .env.example / .env.sample
    case "$file" in
      *.env.example|*.env.sample|*.env.template) continue ;;
    esac
    for pat in "${sensitive_patterns[@]}"; do
      if printf '%s\n' "$file" | grep -Eq "$pat"; then
        sensitive_hits="${sensitive_hits}${file}\n"
        break
      fi
    done
  done <<<"$staged_files"
fi

if [ -n "$sensitive_hits" ]; then
  say "$FAIL" "暂存区包含疑似敏感文件:"
  printf '%b' "$sensitive_hits" | sed 's/^/         - /'
  hint "请用 'git restore --staged <file>' 撤销后再提交,或与用户确认是占位符."
  blocker=$((blocker + 1))
else
  say "$PASS" "敏感文件名扫描通过"
fi

# --- 2) 密钥字面量扫描 (始终执行) ----------------------------------------
secret_patterns=(
  'AKIA[0-9A-Z]{16}'
  'AIza[0-9A-Za-z_-]{35}'
  'sk-[A-Za-z0-9]{20,}'
  'xox[baprs]-[0-9A-Za-z-]{10,}'
  'ghp_[A-Za-z0-9]{36}'
  'github_pat_[A-Za-z0-9_]{50,}'
  '-----BEGIN (RSA|EC|OPENSSH|PGP|DSA) PRIVATE KEY-----'
)

secret_union="$(IFS='|'; echo "${secret_patterns[*]}")"
if [ -n "$staged_files" ]; then
  secret_hits="$(git diff --cached -U0 2>/dev/null | grep -En "$secret_union" || true)"
  if [ -n "$secret_hits" ]; then
    say "$FAIL" "diff 中疑似密钥字面量:"
    printf '%s\n' "$secret_hits" | head -5 | sed 's/^/         /'
    hint "若确为占位符/测试数据,请让用户在确认消息中明示."
    blocker=$((blocker + 1))
  else
    say "$PASS" "密钥字面量扫描通过"
  fi
fi

# --- 3) diff 白空间与冲突标记 --------------------------------------------
if [ -n "$staged_files" ]; then
  if ! git diff --cached --check >/dev/null 2>&1; then
    say "$WARN" "暂存区存在尾随空白或冲突标记 (git diff --check)"
    hint "运行 'git diff --cached --check' 查看详情"
    warnings=$((warnings + 1))
  else
    say "$PASS" "空白符与冲突标记检查通过"
  fi
fi

# --- quick 模式:跳过工具链检查 -------------------------------------------
if "$MODE_QUICK"; then
  echo
  say "$PASS" "--quick 模式: 已跳过工具链检查"
  if [ "$blocker" -gt 0 ]; then exit 1; fi
  if [ "$warnings" -gt 0 ]; then exit 2; fi
  exit 0
fi

echo
echo "--- 项目工具链探测 ---"

# --- 4) Rust ---------------------------------------------------------------
if [ -f Cargo.toml ]; then
  say "$PASS" "探测到 Rust 项目 (Cargo.toml)"
  if command -v cargo >/dev/null 2>&1; then
    if "$RUN_FMT"; then
      if cargo fmt --check >/dev/null 2>&1; then
        say "$PASS" "cargo fmt --check"
      else
        say "$FAIL" "cargo fmt 未通过"
        hint "运行 'cargo fmt' 后重新 stage"
        blocker=$((blocker + 1))
      fi
    else
      say "$SKIP" "cargo fmt (--no-fmt)"
    fi
    if "$RUN_LINT"; then
      if cargo clippy --all-targets --quiet -- -D warnings >/dev/null 2>&1; then
        say "$PASS" "cargo clippy -- -D warnings"
      else
        say "$WARN" "cargo clippy 有警告或错误"
        hint "运行 'cargo clippy --all-targets -- -D warnings' 查看详情"
        warnings=$((warnings + 1))
      fi
    else
      say "$SKIP" "cargo clippy (--no-lint)"
    fi
  else
    say "$WARN" "cargo 未安装,跳过 Rust 检查"
    warnings=$((warnings + 1))
  fi
fi

# --- 5) Node / TypeScript --------------------------------------------------
if [ -f package.json ]; then
  say "$PASS" "探测到 Node 项目 (package.json)"
  pkg_runner=""
  if   command -v pnpm >/dev/null 2>&1; then pkg_runner="pnpm"
  elif command -v yarn >/dev/null 2>&1; then pkg_runner="yarn"
  elif command -v npm  >/dev/null 2>&1; then pkg_runner="npm"
  fi

  if [ -z "$pkg_runner" ]; then
    say "$WARN" "未找到 pnpm/yarn/npm,跳过 Node 检查"
    warnings=$((warnings + 1))
  else
    # 从 package.json 提取 scripts 键名 (容忍无 jq)
    scripts_json="$(node -e "console.log(Object.keys(require('./package.json').scripts||{}).join('\n'))" 2>/dev/null || true)"

    if "$RUN_FMT" && echo "$scripts_json" | grep -qx "format:check"; then
      if "$pkg_runner" run --silent format:check >/dev/null 2>&1; then
        say "$PASS" "$pkg_runner run format:check"
      else
        say "$FAIL" "$pkg_runner run format:check 未通过"
        blocker=$((blocker + 1))
      fi
    elif "$RUN_FMT" && echo "$scripts_json" | grep -qx "format"; then
      say "$SKIP" "仅存在 'format' 脚本 (无 format:check);跳过"
    fi

    if "$RUN_LINT" && echo "$scripts_json" | grep -qx "lint"; then
      if "$pkg_runner" run --silent lint >/dev/null 2>&1; then
        say "$PASS" "$pkg_runner run lint"
      else
        say "$WARN" "$pkg_runner run lint 有警告或错误"
        warnings=$((warnings + 1))
      fi
    fi

    if "$RUN_LINT" && echo "$scripts_json" | grep -qx "typecheck"; then
      if "$pkg_runner" run --silent typecheck >/dev/null 2>&1; then
        say "$PASS" "$pkg_runner run typecheck"
      else
        say "$WARN" "$pkg_runner run typecheck 失败"
        warnings=$((warnings + 1))
      fi
    fi
  fi
fi

# --- 6) Python -------------------------------------------------------------
if [ -f pyproject.toml ] || [ -f setup.py ] || [ -f requirements.txt ]; then
  say "$PASS" "探测到 Python 项目"
  ran_any=false
  if "$RUN_FMT" && command -v ruff >/dev/null 2>&1; then
    if ruff format --check . >/dev/null 2>&1; then
      say "$PASS" "ruff format --check"
    else
      say "$FAIL" "ruff format 未通过"
      blocker=$((blocker + 1))
    fi
    ran_any=true
  fi
  if "$RUN_LINT" && command -v ruff >/dev/null 2>&1; then
    if ruff check . >/dev/null 2>&1; then
      say "$PASS" "ruff check"
    else
      say "$WARN" "ruff check 有告警"
      warnings=$((warnings + 1))
    fi
    ran_any=true
  fi
  if "$RUN_FMT" && ! command -v ruff >/dev/null 2>&1 && command -v black >/dev/null 2>&1; then
    if black --check . >/dev/null 2>&1; then
      say "$PASS" "black --check"
    else
      say "$FAIL" "black 未通过"
      blocker=$((blocker + 1))
    fi
    ran_any=true
  fi
  if ! "$ran_any"; then
    say "$WARN" "未找到可用的 Python formatter/linter,跳过"
    warnings=$((warnings + 1))
  fi
fi

# --- 7) Go -----------------------------------------------------------------
if [ -f go.mod ]; then
  say "$PASS" "探测到 Go 项目 (go.mod)"
  if command -v gofmt >/dev/null 2>&1; then
    bad="$(gofmt -l . 2>/dev/null | head -5)"
    if [ -z "$bad" ]; then
      say "$PASS" "gofmt"
    else
      say "$FAIL" "gofmt 未通过的文件:"
      printf '%s\n' "$bad" | sed 's/^/         - /'
      blocker=$((blocker + 1))
    fi
  fi
  if "$RUN_LINT" && command -v go >/dev/null 2>&1; then
    if go vet ./... >/dev/null 2>&1; then
      say "$PASS" "go vet"
    else
      say "$WARN" "go vet 发现问题"
      warnings=$((warnings + 1))
    fi
  fi
fi

# --- 总结 ------------------------------------------------------------------
echo
if [ "$blocker" -gt 0 ]; then
  say "$FAIL" "预检未通过: $blocker 个阻断项,请修复后再提交"
  exit 1
elif [ "$warnings" -gt 0 ]; then
  say "$WARN" "预检完成: $warnings 个告警,建议与用户确认是否继续提交"
  exit 2
else
  say "$PASS" "全部通过,可以提交"
  exit 0
fi
