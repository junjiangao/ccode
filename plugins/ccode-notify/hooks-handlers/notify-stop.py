#!/usr/bin/env python3
"""
Claude Code Stop 事件通知 Hook
在 Claude 停止响应时发送系统桌面通知
"""
import json
import subprocess
import sys
from typing import Tuple


def send_notification(title: str, message: str) -> bool:
    """发送桌面通知

    Args:
        title: 通知标题
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
                title,
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


def get_message_for_reason(reason: str):
    """根据停止原因返回通知标题和消息

    Args:
        reason: 停止原因 (complete, user_stop, error)

    Returns:
        tuple: (标题, 消息) 或空元组 ()
    """
    reason_messages = {
        "complete": ("Claude Code", "任务已完成 ✓"),
        "user_stop": ("Claude Code", "已停止（用户中断）"),
        "error": ("Claude Code", "已停止（发生错误）⚠"),
    }

    # key 不存在时，视为正常完成
    if reason is None:
        return ("Claude Code", "任务已完成 ✓")

    # key 存在但值为空字符串或非字符串类型，视为未知原因
    if not isinstance(reason, str) or not reason:
        return ("Claude Code", "已停止 (原因: unknown)")

    # 如果找不到对应的 reason，使用原始的格式化默认消息
    return reason_messages.get(reason, ("Claude Code", f"已停止 (原因: {reason})"))


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

    # 提取停止原因
    reason = input_data.get("reason")

    # 获取对应的通知消息
    title, message = get_message_for_reason(reason)

    # 发送通知
    send_notification(title, message)

    # 无论成功与否都以 0 退出，避免影响 Claude Code 正常流程
    # 错误信息已通过 stderr 输出
    sys.exit(0)


if __name__ == "__main__":
    main()
