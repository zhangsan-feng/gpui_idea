## 当前完成的进度

- 已从 GitHub Actions 日志确认：v0.0.2 标签与 Cargo.toml 的 0.0.1 版本不一致，三个平台在版本校验阶段失败。
- 已将 Cargo.toml 和 Cargo.lock 中的项目版本同步为 0.0.2。
- 发布版本校验通过，cargo build --locked 编译通过。

## 当前的计划

- 提交并推送修正，更新 v0.0.2 标签，检查重新触发的发布任务。
