## v0.0.5 发布

### 当前完成的进度

- 更新 `Cargo.toml` 和 `Cargo.lock` 的应用版本至 `0.0.5`。

### 当前的计划

- 由 `v0.0.5` 标签触发 GitHub Actions，构建 Windows x64、macOS ARM64 和 Linux x64 发布包。

## 本地 Windows Release 构建

### 当前完成的进度

- 使用当前工作区代码构建 Windows x64 Release，可执行文件位于 `target/release/gpui_idea.exe`。

### 当前的计划

- 本次构建已完成。

## 编辑器退缩进识别行首空白

### 当前完成的进度

- `Shift+Tab` 支持不足 4 列的缩进、Tab 和常见的 Unicode 空格。一次最多移除 4 列行首缩进。
- `cargo build` 通过。已在编辑器中验证普通空格、不换行空格和 Tab 混合的多行选区可以退缩进，并可一次撤销。

### 当前的计划

- 本次修改已完成。

## Windows 编辑器粘贴后整行空白

### 当前完成的进度

- 用心跳样例复现了多个孤立 `CR` 导致 DirectWrite 排版失败、整行显示为空白的问题。文件中仍有完整正文。
- Windows 编辑器在后台检查 Unicode 剪贴板文本。检测到孤立 `CR` 时，将 `CR` 和 `CRLF` 转换为 `LF` 后插入。
- `cargo build` 通过。已用同一复现样例验证 Ctrl+V 后 `data` 行完整显示，一次 Ctrl+Z 恢复原文。普通文本的快捷键和右键粘贴也已验证。

### 当前的计划

- 本次修改已完成。

## Windows 启动窗口与日志

### 当前完成的进度

- 在 `main.rs` 为 Windows 的 Debug 和 Release 构建统一声明 GUI 子系统，应用启动时不创建控制台窗口。
- `cargo build` 和 `cargo build --release` 均通过。已核对两个 exe 的子系统标记，并验证启动信息和启动失败信息均写入按日期命名的日志文件。

### 当前的计划

- 本次修改已完成。

## 编辑器应用层模块归并

### 当前完成的进度

- 文档服务和编辑器高亮分别归入 `application/editor/document` 和 `application/editor/highlights`。
- 文档服务仍通过 `APPLICATION.documents` 调用。编辑器高亮通过新的模块路径调用。
- Markdown 文件拖入引用归入 `application/editor/markdown`。
- 模块迁移后的 `cargo build` 和格式检查通过。

### 当前的计划

- 在 GUI 中核对文档打开、保存、高亮和 Markdown 文件拖入交互。

## 编辑器网页链接跳转

### 当前完成的进度

- 所有文本编辑器都在 Ctrl + 左键单击后检索网页链接，命中后使用系统默认浏览器打开。
- 普通文本和源码在后台检索点击所在行，Markdown 保留行内链接和引用链接解析。
- 未命中网页链接时，支持语言服务的文件仍可跳转到源码定义。
- 按住 Ctrl 悬停在网页链接上时显示手形光标。悬停命中范围在后台计算，按文本版本和查找范围复用。
- 通用链接识别归入 `application/editor` 模块。
- 手形光标和模块调整完成后，`cargo build` 通过。

### 当前的计划

- 在 GUI 中核对 Markdown、普通文本和源码的手形光标与点击跳转。

## 编辑器格式化管线和 Rust 宏长行换行

### 当前完成的进度

- 长行换行、赋值对齐和条件对齐规则归入 `application/editor/code_formatting`。基础设施层负责向语言服务请求格式化编辑。
- 编辑器在应用语言服务的格式化结果后，通过统一入口依次执行长行换行、赋值对齐和条件对齐模块。
- Rust 宏的圆括号和方括号参数列表超过 120 列时，按当前括号层级的逗号换行，包括 `tokio::select!` 内的日志调用。
- 含注释或无法可靠识别参数边界的宏输入保留原样。
- 已用 timer 源文件验证 LF 和 CRLF 下的缩进及重复格式化一致性，也验证了通用宏和 Go 的换行与对齐。
- `cargo build` 和本次改动的格式检查通过。

### 当前的计划

- 本次修改已完成，GUI 内的交互验证尚未进行。

## Windows 默认应用设置跳转

### 当前完成的进度

- 根据 Windows 版本选择 GPUI Editor 详情页或默认应用总页。应用级链接启动失败时回退到总页。
- 在设置按钮上方提供 Windows 10 的操作路径，并记录跳转目标供排查。
- `cargo build` 通过。

### 当前的计划

- 在 Windows 10 实机上核对设置页落点及按应用配置的操作路径。

## 发布 v0.0.4

### 当前完成的进度

- 已将 Cargo.toml 和 Cargo.lock 中的应用版本同步为 0.0.4。
- `cargo build --locked` 和发布标签一致性校验均已通过。

### 当前的计划

- 推送 master 和 v0.0.4 标签，跟踪 GitHub Actions 并核对三平台发布包及校验文件。
