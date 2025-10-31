# ccode-notify

为 Claude Code 提供轻量的系统桌面通知：当需要用户交互时提醒（Notification），当会话停止时提示完成/中断/错误（Stop）。插件基于 hooks 调用本地 `notify-send`（libnotify）。

## 功能
- 交互提醒（Notification hook）：Claude 等待用户输入/确认时发送通知。
- 会话停止提醒（Stop hook）：根据停止原因发送不同文案：`complete`、`user_stop`、`error`（未知值会回退为通用提示）。

## 工作方式
- `hooks/hooks.json` 注册了两个事件：`Notification` 与 `Stop`。
- 分别调用 `hooks-handlers/notify-interaction.py` 与 `hooks-handlers/notify-stop.py`，超时 5s；脚本失败不会中断 Claude Code（以 0 退出）。

## 依赖
- Linux 桌面环境且可用 `notify-send`（libnotify）。
  - Debian/Ubuntu: `sudo apt-get install -y libnotify-bin`
  - Arch: `sudo pacman -S libnotify`
  - Fedora: `sudo dnf install libnotify`

## 安装与使用
1. 通过仓库根目录 `.claude-plugin/marketplace.json` 安装 `ccode-notify`，或将本目录作为插件源安装。
2. 安装后自动生效，无需额外配置。

## 本地验证
在插件目录下执行（用于快速验证桌面通知是否可用）：

```bash
# 交互提醒
printf '{"message":"请输入信息以继续"}\n' | python3 hooks-handlers/notify-interaction.py

# 会话停止：完成
printf '{"reason":"complete"}\n' | python3 hooks-handlers/notify-stop.py

# 会话停止：用户中断
printf '{"reason":"user_stop"}\n' | python3 hooks-handlers/notify-stop.py

# 会话停止：错误
printf '{"reason":"error"}\n' | python3 hooks-handlers/notify-stop.py
```

## 行为与限制
- 若缺少 `notify-send`，脚本会在 stderr 输出提示，但不影响会话流程。
- 仅发送通知，不读取或持久化任何敏感数据；无外部网络请求。
