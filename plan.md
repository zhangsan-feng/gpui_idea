## 当前完成的进度

- 已从 GitHub Actions 日志确认：v0.0.2 标签与 Cargo.toml 的 0.0.1 版本不一致，三个平台在版本校验阶段失败。
- 已将 Cargo.toml 和 Cargo.lock 中的项目版本同步为 0.0.2。
- 发布版本校验通过，cargo build --locked 编译通过。
- 已推送版本修正并重新触发发布，Windows 构建和打包通过。
- Linux/macOS 日志均报告 Unix 终端代码访问 Alacritty Shell 私有字段；已改为在构造 Shell 前组装环境清理参数。

## 当前的计划

- 执行 cargo build，提交并推送 Unix 终端修正，更新 v0.0.2 标签。
- 跟进三平台构建和 Release 发布结果。
