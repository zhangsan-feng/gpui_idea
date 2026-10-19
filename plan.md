src/infrastructure/timer/mod.rs 在定时器初始化的时候订阅 event_bus 里面的 event_timer 这个事件可以触发 排除gui改动文件 保存文件内容
定时器里面的文件内容监听 直接触发 refresh_data_context 由data_context 去触发 刷新文件树 还是 刷新编辑器的文件内容

使用 alacritty_terminal + gpui 自己绘制一个终端 不再使用
windows-sys
tempfile = "3.27.0"
libc = "0.2.190" 这种系统库了

## 当前完成的进度

- GUI 主窗口左上角与工具窗口标题栏接入 app-logo-v3.png 高清图标，编译期嵌入资源、绘制不增加磁盘 IO；Windows 使用 build.rs 和 embed-resource 将 app-logo-v3.ico 编译为编号 1 的图标资源，供 GPUI 原生窗口与程序图标使用。资源文件和 ICO 更新会触发重新编译。
- 图标接入通过 `cargo check --offline`、`cargo build --offline`、相关 Rust 文件 `rustfmt --check` 与限定文件 `git diff --check`；编译资源包含编号 1 的图标组及 7 张多尺寸图标。未使用 computer use，实际 GUI 与系统图标展示待重启后人工确认。

- LSP 管理“刷新列表”旁新增“清理历史 LSP 版本”，复用已有 Lucide 删除图标和管理操作忙碌/成功/错误状态。按最新要求，每个服务器只保留版本号最新的一个完整已安装版本，支持语义版本、日期版本及 TypeScript 组合版本数值比较；旧固定选择指向保留版本，内存选择同步更新。通过 application 编排、infrastructure 后台清理并刷新列表，不清理下载缓存、暂存或不完整目录。清理与安装/版本切换共用服务器锁，删除前验证目录在 managed 根目录内并跳过符号链接及 Windows 重解析点；旧文件占用导致删除失败时显示重试提示。
- 历史 LSP 清理改动通过 `cargo check --offline`、相关 Rust 文件 `rustfmt --check` 及限定文件 `git diff --check`；未执行实际删除或 GUI 操作，未编写测试用例、未使用 computer use。

- 设置“常规配置”新增 LSP 下载代理开关与 HTTP / HTTPS 代理地址，支持搜索、禁用状态、应用/确定/取消、地址校验与错误提示；复用 settings.json 持久化，兼容已有设置文件。保存后对后续请求生效，关闭后沿用原有系统环境代理行为。
- infrastructure 新增 proxy 模块，按 mod.rs / core.rs / external.rs 拆分初始化、核心逻辑和接口；application 在加载及成功保存后同步配置，统一覆盖 LSP 版本元数据、发行包下载及 npm / Go 安装子进程。代理配置不修改应用全局环境；日志只记录启用状态，不输出代理地址。
- 代理改动通过最新 `cargo check --offline`、相关 Rust 文件 `rustfmt --check` 与限定文件 `git diff --check`；`cargo build --offline` 已链接生成 `target/debug/deps/gpui_idea.exe`，但最终替换 `target/debug/gpui_idea.exe` 时系统返回“拒绝访问”（os error 5），因此构建命令退出失败。未实际执行 GUI 或代理下载验证，未新增测试用例、未使用 computer use。

- 新增 PHP LSP，使用 npm 托管的 Intelephense，复用 Node 兼容版本选择、后台安装、版本目录、固定版本、恢复自动更新和启动更新链路；设置管理列表及底栏显示 PHP / Intelephense。
- PHP/PHTML 等文件映射到 PHP 编辑器语法与 LSP；通过 composer.json / composer.lock 识别项目根目录并启动服务，补全、定义跳转和保存格式化复用现有 application 数据流。Intelephense 缓存集中在 data/lsp/intelephense；不启用自动插入 use 声明和参数提示，以匹配现有补全行为。
- 实际安装 Intelephense 1.18.5，使用项目真实语言服务链路完成初始化、版本采集、函数定义跳转（1 个位置）、补全（包含 strlen）和格式化（3 个编辑）验证，日志为 logs/lsp-php-verify-2026-10-19.log。cargo check --offline、相关 Rust 文件 rustfmt --check 和限定文件 git diff --check 通过；未新增项目测试用例、未使用 computer use。

- 已通过项目真实安装/启动链路补齐 Pyright 1.1.414、typescript-language-server 5.3.0 + TypeScript 5.9.3、yaml-language-server 2.0.0、JDT LS 1.60.0；TypeScript 与 Java 按当前 Node 22.19.0 / JDK 21 选择兼容发行版，不升级本机运行环境。Go 保留官方推荐的 go install 安装方式。
- 复用临时命令行诊断工具，实际验证 Python、JavaScript/TypeScript、YAML、Java、Rust、Go、TOML、Markdown、HTML、CSS、JSON 共 11 类服务，均成功启动并完成 LSP 初始化及版本采集；Go 显示 v0.23.0，TOML 显示 taplo 0.10.0，新安装服务无长 JSON 版本问题。结果记录 logs/lsp-install-results-2026-10-19.json，原始日志 logs/lsp-install-verify-2026-10-19.log。
- 首次 Rust 空目录探测向上命中主项目并遇到沙盒临时路径含花括号导致 Cargo 元数据解析警告；复用已有 test/rust 示例并将诊断进程 TEMP/TMP 指向工作区 target/lsp-runtime-temp 后，初始化及版本显示正常、未再出现元数据警告，结束诊断时正在索引。未修改程序环境处理逻辑，未新增项目测试用例，未使用 computer use。

- 修复底栏 gopls 版本展示整段构建 JSON：初始化时提取 `Main.Version`，本机 gopls v0.23.0 原始版本字段长 2972 字符；解析失败时使用可执行程序版本查询。共用版本入口拒绝 JSON、控制字符和超过 160 字符的元数据并记录日志，命令行回退同样校验。
- 实际查询已安装 Rust、Go、TOML、Markdown、HTML、CSS、JSON 的初始化响应：仅 Go 返回构建 JSON，其余为短字符串或缺失版本字段。另发现 Taplo 初始化响应为内部库 0.8.0，而 CLI 实际为 0.10.0；TOML 改用启动程序版本查询，避免显示库版本。Markdown CLI 输出 2026-02-08，Web 服务沿用安装包版本。Python、TypeScript、Java、YAML 本机未安装，只检查现有版本数据链路。

- 按最新要求删除 LSP 管理顶部说明、指定版本输入区及辅助提示，并移除对应 GUI 状态与手动指定版本操作入口；页面直接显示服务器列表，保留检查更新、已安装版本切换和自动更新模式。

- 设置窗口样式调整：左侧分类栏从 340px 收窄至 220px，搜索与分类增加留白，分类使用已有 Lucide 图标、圆角和柔和蓝色选中态；统一 Microsoft YaHei UI 字体。右侧标题、版本输入分组、服务器卡片和版本状态标签优化间距与层次。
- 本次设置样式调整通过 `cargo check --offline`、相关 Rust 文件 `rustfmt --check` 与限定文件 `git diff --check`；未编写测试用例、未使用 computer use，实际 GUI 效果待人工确认。

- 设置窗口新增“LSP 管理”：显示九组服务器及已安装版本，支持安装最新兼容版本、输入发行版本号安装、固定已安装版本和恢复自动更新。HTML/CSS/JSON 共享同一安装；搜索可按服务器名或语言筛选。
- 版本偏好保存在 managed 工具目录，每个服务器独立保存；启动时优先执行固定版本，版本切换清理内存解析缓存，显式安装历史版本不会将其写入自动更新缓存。设置操作由 application 编排并更新 data_context，磁盘/网络/安装运行于后台，失败展示错误并记录日志。
- LSP 安装及版本选择立即保存，界面明确说明不受底部“应用 / 取消”影响；已有服务重新打开项目后使用新版本。GUI 渲染与交互按 language_servers/ui.rs、core.rs 拆分。

- 设置取消下拉的调整已通过 `cargo check --offline`、相关 Rust 文件的 `rustfmt --check` 和限定文件的 `git diff --check`；实际 GUI 展示待人工确认。

- 设置窗口保留搜索、左右分栏和底部按钮布局，删除旧分类与占位内容；按最新要求左侧仅显示“常规配置”，取消下拉箭头、折叠状态和“自动格式化”子项，右侧直接显示保存时自动格式化开关（默认关闭）。支持搜索、应用/确定/取消、保存中与错误提示；设置持久化至应用 data/settings.json，启动时恢复。
- 格式化按 infrastructure / data_context / application / gui 分层接入 LSP documentFormattingProvider 与 textDocument/formatting；手动保存（含已保存文件 Ctrl+S）和自动保存共用入口。UTF-16 编辑范围校验后逆序应用，保存时在文档锁内核对版本，旧结果丢弃；不支持、失败、超时或交互请求取消时记录日志并继续保存。LSP 等待不占用文件刷新锁，设置磁盘 IO 与格式化处理均在后台运行。
- 格式化回显使用保留撤销历史的 replace_all，并忽略与内存快照一致的 Change 回声；外部磁盘刷新沿用原行为。整个格式化等待共享 15 秒期限，仅使用已存在的语言服务会话，保存不触发安装；只读审查的撤销历史与超时范围问题已修复并复核。
- 设置与 LSP 格式化验证：最新 `cargo check --offline`、`cargo build --offline`、本次变更 Rust 文件的 `rustfmt --check`、限定本次文件的 `git diff --check` 通过；只有环境导致的增量缓存硬链接警告。未编写测试用例、未使用 computer use，GUI 交互和实际 LSP 格式化效果待人工验证。

- 按最新要求取消 LSP 安装进度条（包括往返动画及比例条）；仅保留安装阶段和可获取的真实下载量文字，未改造 Go/npm 下载链路。
- LSP 下载量：直连下载按实际接收字节数每 250ms 更新，已知总大小时提供百分比文字；开始与结束均发布一次，启动/失败时清理安装进度。
- LSP 状态竞争：每个会话独立记录状态，在互斥区内汇总发布；初始化、健康状态、索引与预热由统一运行状态维护，停止/销毁时隔离旧通知。rust-analyzer 的显式 quiescent 状态优先于未结束的进度 token；服务卡片显示各自索引/预热/就绪状态，补充状态转换与 token 日志。
- 安装进度/状态竞争验证：`cargo check --offline`、`cargo build --offline`、变更 Rust 文件的 `rustfmt --check` 以及限定本次已跟踪文件的 `git diff --check` 通过。只读审查未发现新的并发/停止问题，已补齐各服务卡片的警告/错误详情；未编写测试用例，实际下载及 GUI 状态变化尚待人工确认。
- 顶部窗口栏偶发无法拖动/双击最大化：检查现有日志后追踪到 GPUI 的缓存绘制重放遗漏 `window_control_hitboxes`；移除主窗口中 WindowBar 的 `.cached(...)`，确保每帧重新登记原生窗口控制命中区域，并增加拖动区域按下事件的 debug 日志（点击次数、位置、最大化状态）。
- 顶部窗口栏验证：`cargo check --offline`、`cargo build --offline`、变更 Rust 文件的 `rustfmt --check`、限定本次文件的 `git diff --check` 通过；未使用 computer use，实际拖动和双击交互待人工确认。
- LSP 状态改为点击展开/收起，安装期间也不再自动弹出；复用现有 Popover，支持点击外部和 Esc 关闭。详情按服务卡片分组，区分语言/服务名、状态徽标和固定标签列；按最新要求移除工作区及安装目录，仅保留版本和启动程序。
- LSP 界面变更验证：`cargo check --offline`、`cargo build --offline`、变更 Rust 文件的 `rustfmt --check` 通过；未编写测试用例，未使用 computer use，实际 GUI 布局及点击交互待人工确认。
- 定时器在 `mod.rs` 初始化时订阅 `event_bus.timer_event`，`core.rs` 明确匹配 `SaveFile`、`ExcludeGuiFileChange`、`WatchFiles`、`Delay`、`Cancel`；已移除 `execute/every/Work/Operation` 回调接口，事件不携带闭包或 Future。
- 文件保存仍由 application 编排；GUI 自动保存只提交请求，不再筛选和逐个调度待保存文件。
- 文件监听定时批处理直接发送 `refresh_data_context`，区分文件内容变化和目录结构变化。
- application 完成磁盘读取后提交刷新批次，由 data_context 决定通知编辑器、文件树或统计面板；相同目录快照不增加版本号。
- 终端继续使用 Alacritty PTY、ANSI 解析与 GPUI 绘制，移除原 JobObject/FFI 实现和 `windows-sys`、`tempfile`、`libc` 的直接依赖。
- Windows 多行运行命令通过标准库管理启动脚本；Alacritty 关闭和线程回收在后台执行。
- 自动保存开始时在内存数据锁内复核冲突和关闭暂停状态；慢保存后仍有新修改会继续重试。
- 终端显式停止使用标准库调用 Windows `taskkill` 或 Unix `ps`/`kill`，再关闭 Alacritty，避免仅发送 SIGHUP 导致无法停止。
- 验证：`cargo check --offline`、`cargo build --offline`、变更 Rust 文件的 `rustfmt --check` 通过；编译只有环境导致的增量缓存无法硬链接警告。
- 重复编译排查：Cargo 日志确认 `ring` 因 `CARGO_MANIFEST_DIR` 在项目根路径与未设置之间变化而失效，带动 rustls/reqwest 整链重编。已在终端、语言服务、Cargo 元数据与工具链子进程启动时清理继承的 Cargo 临时包信息，保留用户构建配置；诊断记录见 `target/rustrover-rebuild-diagnostic.log`。
- 重复编译验证：模拟 Cargo 包信息继承并清理后构建 2.99 秒，随后普通构建 3.16 秒，两次均没有重新编译 ring/rustls/reqwest。

## 当前的计划

- 重启新构建程序，人工确认主窗口及工具窗口标题栏高清 PNG 图标、Windows 程序/任务栏 ICO 图标；按项目约束不使用 computer use。

- 人工确认 LSP 历史清理按钮位置、处理中禁用、清理结果及列表刷新；检查每个服务器只保留版本号最新的一个已安装版本、旧固定选择迁移和文件占用错误提示。按项目约束不编写测试用例、不使用 computer use。

- 人工确认设置中的代理开关、地址输入、非法地址错误、应用/取消与重启恢复，并通过实际代理分别下载安装直连发行包、npm 和 Go 类型 LSP；本次不新增测试用例、不使用 computer use。

- 人工打开 PHP / Composer 项目，确认 PHP 语法高亮、设置中的安装/更新及版本选择、底栏版本、补全、定义跳转和保存格式化的 GUI 操作。

- 新安装服务已完成真实启动、初始化和版本采集验证；人工在 GUI 打开各语言文件确认底栏显示和编辑交互。

- 人工确认 LSP 管理顶部说明及版本输入区已移除，服务器列表和更新/版本选择操作正常显示。

- 人工确认设置窗口 220px 分类栏、选中与悬停状态、图标和字体，检查 LSP 卡片及版本列表滚动显示。

- 完成新增 LSP 管理的编译、构建、格式和只读审查；人工确认设置分类切换、搜索、安装最新/指定版本、固定/恢复自动更新、重开项目后的实际版本和失败提示。按项目约束不编写测试用例、不使用 computer use。

- 人工检查设置左侧仅显示“常规配置”、右侧直接显示自动格式化开关、设置搜索、应用后重新打开及重启恢复、取消不应用；使用支持格式化的 LSP 检查 Ctrl+S、自动保存、保存期间继续输入及格式化失败回退。按项目要求不编写测试用例、不使用 computer use。

- 人工检查首次安装时的底栏阶段、真实下载量文字，以及多个服务并行运行、索引完成/再次索引、预热取消和服务重启时的状态展示。
- 人工检查编辑、滚动、后台刷新及 LSP 弹层关闭后，顶部空白拖动区仍可拖动、双击最大化/还原，最小化/最大化按钮正常响应。
- 人工检查 LSP 点击展开、再次点击/外部点击/Esc 关闭、列表滚动与长路径换行，以及安装状态更新。
- GUI 实际交互待人工验证：自动保存、外部内容修改、目录增删改名、终端输入与停止。按项目要求不编写测试用例、不使用 computer use。

## 当前限制

- 移除 JobObject/系统 FFI 后，完全脱离 PTY 且在主 shell 退出后继续运行的后台进程，不再具有原来的独立托管清理保障；活动终端的显式停止仍会终止可识别的进程树。未引入其他系统绑定替代上述依赖。
- 当前编译验证平台为 Windows，Unix 进程清理分支尚未在对应系统运行验证。
- Cargo 环境清理已覆盖 Windows 交互/运行终端及 Unix 显式运行命令；Unix 默认交互终端保留 Alacritty 自己的账户 shell/login 行为，未额外包装清理。
