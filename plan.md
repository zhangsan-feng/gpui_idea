## 当前完成的进度

- 已从 GitHub Actions 日志确认：v0.0.2 标签与 Cargo.toml 的 0.0.1 版本不一致，三个平台在版本校验阶段失败。
- 已将 Cargo.toml 和 Cargo.lock 中的项目版本同步为 0.0.2。
- 发布版本校验通过，cargo build --locked 编译通过。
- 已推送版本修正并重新触发发布，Windows 构建和打包通过。
- Linux/macOS 日志均报告 Unix 终端代码访问 Alacritty Shell 私有字段；已改为在构造 Shell 前组装环境清理参数。
- Unix 终端修正已提交为 284059a，本地 cargo build --locked、rustfmt 检查通过。
- GitHub Actions 42779060428 的 Windows、Linux、macOS 构建与 Publish GitHub Release 全部成功。
- v0.0.2 已公开发布，包含三个平台的安装包及各自的 SHA256 校验文件。

## 当前的计划

- 本次发布修复已完成，无待处理事项。
