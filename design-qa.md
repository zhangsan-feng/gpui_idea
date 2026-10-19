

验证边界：未新增测试用例；没有通过 Windows UI 自动化执行终端命令。Run 程序启动、输入、退出码与进程树管理路径已编译并完成独立静态审查；界面、统计扫描、Git 读取、终端启动输出及会话关闭通过实际截图和日志验证。
自动命令补充核对：最终构建实机从当前 Rust 源文件识别出 Cargo.toml、gpui_idea 包及 bin，命令框完整显示，日志确认后台识别成功。Windows 包管理器入口、旧配置迁移和停止取消待执行意图经独立审查复核；没有逐一执行其他语言运行器。

- Application 重构后的最终启动同样观察 6 秒、stderr 为空；日志确认首次项目识别发生在窗口创建前，恢复 8 个文件仅一次识别。诊断文件为 artifacts/run-config-application-startup-stdout.log 和 run-config-application-startup-stderr.log。


