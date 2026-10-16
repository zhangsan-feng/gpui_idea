# gpui_idea

使用 Rust 和 GPUI Kit 开发的原生桌面代码编辑器，界面参考 IDEA 风格

## 代码提示与日志

Python、JavaScript、Go、Rust 编辑器通过 LSP 提供语义提示，输入标识符、`::` 或 `.` 自动触发，也可按 `Ctrl+Space`。首次使用需等待语言服务加载项目；其他语言保留本地词法提示。当前不提供自动导入和代码片段占位符。

| 语言 | 后端 | 准备方式 |
| --- | --- | --- |
| Python | Pyright | Node.js 20+，在 `tools/lsp` 执行 `npm ci` |
| JavaScript | typescript-language-server + TypeScript | 同上 |
| Go | gopls | `go install golang.org/x/tools/gopls@latest`，将 Go bin 目录加入 PATH |
| Rust | rust-analyzer | `rustup component add rust-analyzer`，将 Rust bin 目录加入 PATH |

Python/JavaScript 后端依赖在 `tools/lsp/package-lock.json` 锁定；程序直接用 Node.js 启动实际入口，兼容 Windows npm 的 cmd 包装器，也可使用项目本地或全局 npm 包。启动时不会自动下载语言服务器。缺失后端会在状态栏提示原因，失败启动有 30 秒重试间隔。

打开源码时按语言启动并复用会话，嵌套项目根据最近的 Cargo.toml、go.mod/go.work、pyproject.toml、package.json 等配置识别根目录。根目录 `test` 提供这四种语言的独立手动验证项目，详见 [验证说明](test/README.md)。

控制台和日志文件默认只记录警告与错误。需要排查时，在启动前设置 `RUST_LOG`，无需修改代码：

```powershell
$env:RUST_LOG = 'warn,gpui_idea=debug'
cargo run --offline
```

可用 `RUST_LOG=debug` 查看所有模块，或用 `warn,gpui_idea::infrastructure::language_server=debug` 只跟踪语言服务。恢复默认：`Remove-Item Env:RUST_LOG -ErrorAction SilentlyContinue`，然后重新启动应用。


## 界面预览

![100510.png](image_example/100510.png)
![161002.png](image_example/161002.png)
![161312.png](image_example/161312.png)
![161328.png](image_example/161328.png)
