#!/usr/bin/env python3
"""
Claude Code 交互通知 Hook
在 Claude Code 需要用户交互时发送系统桌面通知
"""
import json
import subprocess
import sys


def send_notification(message: str) -> bool:
    """发送桌面通知

    Args:
        message: 通知消息内容

    Returns:
        bool: 是否成功发送通知
    """
    try:
        # 使用 notify-send 发送桌面通知
        # -u normal: 正常优先级
        # -t 5000: 5秒后自动关闭
        # -i dialog-information: 使用信息图标
        subprocess.run(
            [
                "notify-send",
                "-u", "normal",
                "-t", "5000",
                "-i", "dialog-information",
                "Claude Code",
                message
            ],
            check=True,
            capture_output=True,
            timeout=5
        )
        return True
    except FileNotFoundError:
        # notify-send 命令不存在
        print("错误: 未找到 notify-send 命令，请安装 libnotify", file=sys.stderr)
        return False
    except subprocess.TimeoutExpired:
        print("错误: 通知发送超时", file=sys.stderr)
        return False
    except subprocess.CalledProcessError as e:
        print(f"错误: notify-send 执行失败: {e}", file=sys.stderr)
        return False
    except Exception as e:
        print(f"错误: 发送通知时发生异常: {e}", file=sys.stderr)
        return False


def main():
    """主函数：读取 hook 输入并发送通知"""
    try:
        # 从 stdin 读取 JSON 输入
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError as e:
        print(f"错误: 无效的 JSON 输入: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"错误: 读取输入时发生异常: {e}", file=sys.stderr)
        sys.exit(1)

    # 提取通知消息
    message = input_data.get("message", "")

    if not message:
        # 没有消息内容，静默退出
        sys.exit(0)

    # 发送通知
    success = send_notification(message)

    # 无论成功与否都以 0 退出，避免影响 Claude Code 正常流程
    # 错误信息已通过 stderr 输出
    sys.exit(0)


if __name__ == "__main__":
    main()
