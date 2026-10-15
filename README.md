# gpui_idea

使用 Rust 和 GPUI Kit 开发的原生桌面代码编辑器，界面参考 IDEA 风格

## 界面预览

![gpui_idea 界面示例：项目文件树、多标签编辑器和工具栏](image_example/093944.png)

## 已有功能

- **项目管理**：创建、打开项目，查看最近项目，恢复工作区会话。
- **文件管理**：浏览目录树，新建文件或目录，重命名、删除、复制粘贴、拖动移动，在资源管理器中定位文件。
- **文本编辑**：多标签编辑、拖入外部文件、手动保存、空闲后自动保存。
- **文件同步**：监听外部文件变化，批量刷新目录和已打开文档；遇到未保存内容冲突时保留本地编辑并提示。
- **搜索**：文件搜索和项目内容搜索。
- **运行配置**：识别项目或文件的运行方式，编辑命令与参数，管理多个运行配置和运行实例。
- **集成终端**：基于 Alacritty 终端内核提供 PowerShell 会话，支持多个终端标签。
- **辅助面板**：查看项目统计、Git 状态、提交记录和提交详情。
- **工作区恢复**：保存打开文件、目录展开状态及部分窗口布局。

## 开发与运行

### 环境准备

- Windows 开发环境。
- 支持 Rust 2024 Edition 的 Rust 工具链及 Cargo。
- 使用 MSVC 工具链时，需准备 Visual Studio C++ Build Tools 和 Windows SDK。
- 系统中可用的 `powershell.exe`；Git 面板需要 `git` 命令。
- 运行其他语言的项目时，需自行安装对应的编译器或运行时。

在仓库根目录执行：

```powershell
cargo run
```

指定打开的项目目录：

```powershell
cargo run -- --project "C:\path\to\your-project"
```

不传 `--project` 时，程序尝试恢复上次工作区；没有可恢复的项目时使用本仓库目录。

编译检查与发布构建：

```powershell
cargo check
cargo build --release
```

构建产物位于 `target/release/gpui_idea.exe`。当前数据和日志路径基于编译时的仓库目录，尚未改为独立安装目录。

## 常用操作

| 操作 | 使用方式 |
| --- | --- |
| 保存当前文档 | `Ctrl+S` |
| 自动保存 | 活动空闲约 3 秒后触发 |
| 文件操作 | 项目树右键菜单 |
| 打开外部文件 | 将文件拖入工作区 |
| 终端复制 | 终端获得焦点时按 `Ctrl+Shift+C` |
| 终端粘贴 | 终端获得焦点时按 `Ctrl+Shift+V` |

## 架构与数据流

```text
src/
├── main.rs          # GPUI 启动入口
├── gui/             # 界面交互、视图状态与渲染
├── component/       # 通用组件与图标资源
├── application/     # 参数校验、业务编排及后台任务
├── data_context/    # 内存快照、版本和 GUI 变更通知
├── domain/          # 共享业务模型
├── event_bus/       # refresh_data_context 刷新请求
└── infrastructure/  # 文件 IO、文件监听、终端、定时器和持久化
```

GUI 发起操作时，通过 `cx.spawn → run_application → tokio::spawn` 执行业务任务；阻塞 IO 在 Infrastructure 中使用 `spawn_blocking`，避免阻塞 GUI 线程。

**主动文件操作：**

```text
GUI → Application
    → 通知 Infrastructure 登记本次操作的监听忽略路径
    → Infrastructure 执行文件 IO
    → Application 更新 DataContext
    → DataContext 通知 GUI → 重读快照并重绘
```

**外部文件变化：**

```text
Infrastructure 文件监听 → 累积受影响路径
    → 定时器每 3 秒发布 refresh_data_context
    → Application 读取磁盘数据并批量更新 DataContext
    → DataContext 通知 GUI → 重读快照并重绘
```

EventBus 只负责 `refresh_data_context` 请求，支持指定路径或全量刷新。Application 也提供可等待完成的立即刷新入口。DataContext 不执行 IO，GUI 直接订阅它的变更通知。

主动文件操作、目录加载与后台刷新使用共享异步锁，避免旧目录快照覆盖新数据。监听忽略期间被过滤的路径，会在操作及宽限期结束后由定时器核对；操作失败或取消时安排全量核对。

## 数据与日志

| 路径 | 用途 |
| --- | --- |
| `data/open-history.json` | 打开历史及工作区会话 |
| `logs/` | 应用运行日志，排查问题时优先查看 |
| `image_example/` | README 界面示例图片 |
| `assets/` | 界面使用的静态资源 |

`data/`、`logs/` 和 `target/` 已加入 Git 忽略规则。

## 当前限制

- 文本编辑目前支持 UTF-8（含 BOM），支持统一的 LF 或 CRLF 换行。
- 单个文档的读取与保存上限为 2 MiB，文本行数上限为 50,000 行；不支持二进制文件编辑。
- 原生终端、进程管理和资源管理器定位包含 Windows 专用实现。
- 运行配置是否可执行，取决于本机工具链和目标项目；界面中提供的操作不代表所有语言均支持完整调试能力。
