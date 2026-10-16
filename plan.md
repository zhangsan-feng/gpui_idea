# 当前完成的进度

- v0.0.1 发布准备：Cargo.toml/Cargo.lock 版本统一为 0.0.1，新增仅由 v* 标签推送触发的 GitHub Actions；Windows x64、macOS ARM64、Linux x64 分别在原生 Runner 编译 release。工作流校验标签版本、安装 Linux 依赖、缓存 Rust 构建，生成包含可执行文件及同级 assets 的便携包和 SHA-256 校验文件；三个构建全部成功后发布 GitHub Release。打包脚本使用 Python 标准库，不在工作流中新增测试用例。
- 发布检查已通过 actionlint v1.7.12、Python 语法、Cargo/标签版本一致性、Windows 便携包布局及 SHA-256 核对、暂存范围 git diff --check 和只读工作流审查；打包检查使用已有二进制，仅验证归档格式，实际 v0.0.1 三平台二进制由 GitHub Actions 编译。

- 终端跨平台适配：Windows 使用 COMSPEC/cmd，macOS/Linux 交互终端沿用 Alacritty 的用户 Shell 发现和 macOS 登录会话处理；自动运行先加载用户登录环境，再由 /bin/sh 执行 POSIX 命令。移除固定 PowerShell 启动脚本及 base64 直接依赖，windows-sys 仅保留在 Windows 目标依赖中。Windows Job Object 和 Unix session/进程组清理拆分，覆盖停止、关闭和退出后的后台任务；Windows 使用私有临时目录 ready 文件握手，不消费键盘输入。
- 运行配置同步迁移：路径引用、切换目录、创建输出目录、构建成功后运行、Python/tsx 入口、Rust 调试和启动参数按 cmd/POSIX 语法生成，Unix 输出不再强制 .exe，显式 .ps1 文件仍调用已安装的 PowerShell。Windows 实际 PTY 诊断已验证交互输入、多行中文、退出码、复杂路径编译运行、失败短路、复杂 argv、停止及直接关闭时清理父子进程；诊断代码仅位于忽略的 target 目录。终端及运行参数模块已通过 macOS ARM64、Linux x86_64 独立交叉 cargo check；只读审查未发现剩余重大问题。未新增项目测试用例、未使用 computer use、未修改 README。
- 跨平台终端的 Windows 整体编译链接通过，输出 target/debug/gpui_idea-terminal-platform.exe；修改文件 rustfmt --check 与修改范围 git diff --check 通过。编译仅提示自定义输出参数及增量缓存硬链接回退复制，未执行 GUI 操作。

- 编辑器代码提示列表接入已有 Scrollbar，超过 9 条候选时持续显示纵向滚动条，预留右侧空间，提供悬停/拖动颜色并在鼠标按下时保留编辑器焦点。修改文件 rustfmt 检查通过；cargo check --offline 被现有 src/event_bus/external.rs 第 5 行附近语法错误及关联缺失方法阻断，本次 UI 文件未报告错误。未新增测试、未使用 computer use。

- 本次重构完成：event_bus 新增 refresh_gui（每组件独立 watch 通道，终端按 TerminalId）与 gui_event（按工作区和目标组件隔离的事件邮箱）。Workspace 直接组装左/右侧栏、窗框栏、底部状态栏、底部工具面板、文件树、编辑器七个 Entity；移除工作区总 components.rs、parts.rs、external.rs 和所有聚合状态容器。各组件自行持有 State、订阅、Render、操作及外部入口，枚举与结构体归所属模块；跨组件修改经定向事件执行，文件移动/删除分别由文件树和编辑器处理自身投影/缓冲区。区域使用渲染缓存，文档脏标记、光标、运行/LSP 状态按组件刷新，终端普通输出只刷新对应终端。
- 验证：cargo check --offline、最终独立编译链接（target/debug/gpui_idea-components.exe）、修改文件 rustfmt --check、修改范围 git diff --check 通过；全部本次相关 Rust 文件不超过 800 行。启动日志确认主窗口打开、文件树与编辑器恢复、Rust LSP initialize，未见 panic。历史展开目录 tools/managed/rust-analyzer 已不存在，保留原有读取失败提示。未新增测试用例、未使用 computer use、未修改 README。

- 本次 LSP 安装改为先读取用户已安装的 SDK/运行时，再直接选择兼容的服务器版本：Node 系根据 npm engines 匹配并由 npm 安装，gopls 根据模块的 Go 版本要求匹配并由本机 Go 安装，JDT LS 根据 JDK 对应兼容发行线选择；Rust Analyzer、Taplo、Marksman、JDT LS 的独立发行包由 reqwest 下载，Rust 原生解压，不再用 Node 下载脚本或外部 tar 解压。只安装语言服务器，不安装或降级用户 SDK。启动时后台检查已有服务器，缺失服务器按需安装，按版本隔离缓存，下载失败保留已有安装，状态详情保留匹配版本的原因。
- 已通过 cargo check --offline、修改文件 rustfmt 与修改范围 diff 检查。移除 Node 的 PATH 后，Rust Analyzer、Taplo、Marksman 全新下载及 initialize 通过；Pyright、HTML/CSS/JSON、YAML 的 npm 安装及 initialize 通过；本机 Node 22.19.0 自动匹配 typescript-language-server 5.3.0 / TypeScript 5.9.3 并初始化成功，本机 JDK 21 自动匹配 JDT LS 1.60.0 并初始化成功。模拟断网后 Rust/Taplo/Marksman/Pyright 均复用缓存启动；启动维护任务正常结束且未覆盖会话状态。Go 1.26.2 已匹配 gopls 0.23.0，首次 SDK 构建及 initialize 已通过。临时诊断仅在忽略目录 target/logs，未新增测试用例、未使用 computer use、未修改 README。本段取代下方历史记录中的“拒绝降级”和“JDT LS 一律要求 JDK 25”策略。

- 按本次要求移除 C/C++/CMake 的 LSP 文件映射、根目录识别、启动、安装及状态条目。Go 改用本机 Go 执行 go install，服务器与构建/模块缓存放入 tools/managed，并设置 GOTOOLCHAIN=local 禁止自动下载 SDK；Java 使用本机 JDK、Node.js 下载官方 JDT LS；TOML/Markdown 使用本机 Node.js 下载官方 Taplo/Marksman。缺少环境时明确提示安装；Node.js 不兼容保留升级报错，不降级。实际验证 Taplo/Marksman 首次安装及 initialize 成功；JDT LS 1.62.0 下载成功，从发行包依赖检测到 JDK 25+ 要求，本机 Java 21 已正确显示升级提示；最新版 gopls v0.24.0 要求 Go 1.27+，本机 Go 1.26.2 已正确阻止 SDK 自动获取并回报错误。空 PATH 验证缺少 Node/Go/Java 时均返回安装提示，C/C++/CMake 不再接入。未新增测试用例、未使用 computer use、未修改 README。

- 本次直接调用现有 Infrastructure LSP 启动链路进行实际验证：Python、HTML、CSS、JSON、YAML 首次 npm 下载及 initialize 成功，缓存重启成功；本机 PATH 的 gopls 启动成功。JS/TS/JSX/TSX 最新服务器要求 Node >=22.22.2，本机 v22.19.0，确认按要求拒绝安装，不降级。安装失败详情现会显示所需/当前版本和“请升级 Node.js 后重试”，每次安装使用独立日志避免互相覆盖；已复现确认错误写入 Failed 状态详情。原生 clangd、CMake、JDT LS、Taplo、Marksman 本机未安装；其自动安装来源正在确认。诊断程序与输出仅位于忽略目录 target/logs，未新增测试用例、未使用 computer use、未修改 README。

- 修正首次启动 Rust LSP 绕过 tools 的问题：不再直接复用 PATH 中的 rust-analyzer，缺失本地缓存时用用户已安装的 Node.js 在线下载官方服务器 ZIP，解压并提交到 tools/managed/rust-analyzer，再从该目录启动；不安装 Rust SDK。语言服务在准备阶段即登记，发布安装/初始化/就绪/失败状态、实时下载量、安装目录和实际启动程序，经 Application/DataContext 刷新 GUI。安装时状态栏自动展开原位置的浮动卡片，完成后恢复普通悬浮详情；安装失败保留错误条目。未新增测试用例、未使用 computer use、未修改 README。

- 应用目录统一为 infrastructure/paths：开发构建使用项目根目录，发布构建使用 exe 同级目录，data/assets/logs/tools 均不依赖当前工作目录。历史和启动默认目录、日志及 Java LSP 工作数据接入统一路径；外部 assets 启动时在后台预加载，GUI 从内存读取并保留内嵌资源回退。Node LSP 只调用用户自行安装的 Node.js/npm，首次使用通过 npm install 在线获取对应服务器包；不再内嵌或读取开发仓库中的 package.json/package-lock.json，配置与安装记录均由 npm 在 tools/managed 运行时生成。使用跨进程锁、临时目录提交、缓存复用和安装日志；程序不下载 Node.js 或任何 SDK/运行时，原生 LSP 仍由用户自行安装。未新增测试用例、未使用 computer use、未修改 README。

- 补齐编辑器已有 16 种语言的 LSP 映射：TypeScript/TSX 与 JS/JSX 复用 typescript-language-server；新增 C/C++ clangd、CMake、Java JDT LS、TOML Taplo、Markdown Marksman、HTML/CSS/JSON 与 YAML 启动接入及项目根目录识别。定义跳转解除 Rust 限制，同族未保存文档参与同步；能力协商避免向不支持的服务发送补全/定义请求，缺失后端保留词法补全。Web 片段转换为普通初始文本，Java 工作数据按进程/项目隔离。Node 依赖已安装并锁定，原生新增服务需按 tools/lsp/SERVERS.md 安装。未新增测试用例、未操作 GUI、未修改 README。

- 打开历史升级为 v2 项目索引：仅保留最近项目、最后项目及恢复状态文件引用，各项目状态存放 data/workspace-sessions，去掉 session/sessions 重复存储；目录展开、文本/图片打开和标签切换不再写文件/目录历史。光标及布局保存通常只写当前项目状态，保留旧快照版本保护、进程锁及临时文件提交，旧 v1 自动迁移并备份为 open-history.v1-backup.json。当前实际文件由 32704 字节降为 2353 字节，7 个项目状态逐项核对一致，重复读取不会重写索引；未新增测试用例、未使用 computer use。

- 项目树右键新增“复制项目路径”（相对项目根目录）和“复制全盘路径”（包含磁盘根目录的绝对路径）；支持多选按行复制，项目根目录相对路径显示“.”，保留原有文件复制/粘贴语义。仅复制路径文本，不执行文件 IO；未新增测试用例、未使用 computer use。

- 编辑区支持 PNG/JPEG/GIF/WebP/BMP/ICO/SVG/TIFF/PNM 图片只读预览：沿用文件双击、标签切换、关闭、前进后退与会话恢复，自动适应窗口并显示文件大小及加载/解码失败信息，图片不创建文本编辑器或启动 LSP。读取经 GUI spawn → Application → Infrastructure spawn_blocking，复用项目路径校验和 32 MiB 读取上限；DataContext 保存图片快照，文件监听更新项目内图片，关闭、移动和删除时清理或重新加载预览。未新增依赖、测试用例，未使用 computer use。

- 项目树文件改为单击仅选中、双击打开内容；Ctrl 多选、整组拖放和目录原有展开交互保留。cargo check --offline、修改文件格式检查和修改范围 git diff --check 通过；未新增测试用例、未使用 computer use。

- 项目树新增 Ctrl+左键多选/取消选中与选中数量提示，普通点击恢复单选，右键已选项保留整组选中；拖动选中项移动整组，Delete 键或右键“删除选中项”统一弹出批量确认。父子项同时选中按顶层项处理，移动后重映射标签/导航/选中路径，删除后清理编辑器状态。Application 串行编排批量操作并全量预检查，Infrastructure 在后台验证路径、链接和重名；IO 失败停止剩余项并回报已完成结果。未新增测试用例、未使用 computer use。

- 设置窗口按 IDEA 参考图调整为 1120×680 内容区：左侧搜索占位与分层分类导航，右侧面包屑和设置内容占位，底部固定帮助及确定/取消/应用操作栏；默认选中“外观”，分类可切换展示，应用禁用，确定/取消仅关闭窗口。按本次要求暂不接入搜索、设置项、保存或业务操作，复用现有图标及单实例窗口生命周期，未新增测试用例、未使用 computer use。

- 外部源码标签补充 IDEA 参考图的浅黄色背景（#fff7e6），悬停加深为 #ffefcc，拖动预览同步背景；保留已有蓝色文字和选中下划线。cargo check --offline 和修改范围 git diff --check 通过；未新增测试用例、未使用 computer use。

- 底部 LSP 状态改为悬浮详情卡片，列出实际启动的语言服务、名称、版本、初始化状态和工作区路径；空列表显示无已启动服务。Infrastructure 维护运行列表，经 Application 订阅写入 DataContext，GUI 只读快照；版本优先使用 initialize.serverInfo，Node 后端回退实际安装包版本，Rust/Go 后台限时查询对应程序版本，退出后移除条目。Session 启动代码按功能拆到 core/start.rs，未新增测试用例、未使用 computer use。

- 源码跳转标签配色：标准库及依赖等外部源码使用蓝色标签文字和选中下划线，拖动预览同步配色，悬停说明“外部源码 · 只读”；项目文件保留原有配色。复用现有外部源码标记，cargo check --offline、标签文件格式检查和修改范围 git diff --check 通过；编译仅提示增量缓存硬链接回退复制。未新增测试用例，未使用 computer use。

- 顶部最左侧主菜单仅保留“新建项目”和“打开项目”，其余下拉项已移除；复用已验收的项目窗口，其他工具栏按钮和现有快捷键保留。cargo check --offline 通过，未新增测试用例。

- 新建项目窗口按 IDEA 参考改为左侧 Rust、Go、Python、JavaScript、TypeScript 五语言选择，右侧配置项目位置、工具链及版本和项目模板；Rust 提供应用/库模板，Go 支持 Module 路径。Rust/Go 分别在后台实际执行 cargo init/go mod init，Python/JS/TS 生成基础工程；初始化在临时目录完成后再创建目标，拒绝覆盖已有目录。cargo check、独立构建及只读审查通过，用户已验收；未新增测试用例。
- 右上角齿轮改为直接打开独立“工作区配置”窗口，重复点击激活已有窗口；保留 Project 面板开关、工具链重新识别、运行配置和刷新项目树操作。复用现有工作区动作及异步 IO 链路，配置窗随主工作区关闭。已处理首次绘制父实体借用冲突，编译和窗口显示验证通过，用户已验收，未新增测试用例。
- 新增 Infrastructure git 模块，迁出 filesystem 的 Git 进程调用；Application 编排读取并写入 DataContext。顶部仅在真实工作树仓库存在时显示实际分支，支持父目录仓库、空仓库和 Detached HEAD 短哈希；切回工作区时异步刷新。cargo check、独立构建及只读审查通过，窗口已验证真实 master、无仓库隐藏、未提交仓库显示分支，以及外部切换分支后激活窗口刷新；未新增测试用例。
- Python、JavaScript、Go 补全已接入 Pyright、typescript-language-server、gopls，Rust 保留 rust-analyzer；复用现有异步会话、取消、文档同步和位置转换。按最近项目配置选择嵌套根目录，非 Rust 后端打开文件后按需启动，状态栏移除 Rust 固定文案；缺失后端启动失败缓存 30 秒，避免持续输入时刷屏。
- 新增 tools/lsp/package.json 与锁文件，锁定 Python/JavaScript 后端和 TypeScript 版本，Windows 通过 Node.js 直接运行包入口；Go/Rust 使用 PATH 中的原生服务器。已安装 Node 后端，本机 gopls 可用。未新增自动化测试用例。
- 四语言示例均已运行通过；实际窗口已验证 Python Path 标准库和跨文件 Book 成员，JavaScript featuredBook 字段/方法与 Enter 插入，Go fmt 标准库和跨包 Book 字段/方法。临时输入已从 test/ 示例恢复。多语言接入最终构建、格式与差异空白检查通过；只读审查发现的嵌套项目导航根目录与 Rust 兄弟 crate 未保存内容同步问题已修复并复核。
- 新增 test/，本期仅包含 Python、JavaScript、Go、Rust 四个独立手动验证项目；提供运行命令、Rust 标准库/Tokio 补全检查说明及生成目录忽略规则，不接入主应用构建、不新增自动化测试用例。其他语言后续再支持。
- Rust 代码提示由本地词法扫描接入现有 rust-analyzer 会话，传入文档绝对路径，同步当前内容与其他已打开 Rust 文档；支持标识符、`::`、`.` 和 Ctrl+Space 触发。后台处理取消、响应排序、UTF-16 到编辑器字符列转换；仅使用编辑器可直接插入的候选，暂不提供自动 use 导入和片段占位符。
- 日志默认 Warn，通过 RUST_LOG 按需开启全局或指定模块 Debug；窗口性能、文件统计和 LSP 详细过程降为 Debug，控制台及文件采用同一过滤条件。README 已补充启动命令。
- 本次获准使用 computer use，已在 target/completion-check 隔离项目实际验证 std、std::、std::io::std、tokio、tokio:: 自动候选及上下键/Enter 插入；含中文和 emoji 的行也能准确替换候选。cargo build --offline 通过，未新增测试用例。

- 统计表改为 uniform_list 虚拟滚动，按可见范围创建固定 22px 数据行，固定表头、持久化滚动位置并接入纵向滚动条；保留横向滚动、排序、行选中、复制和末尾 Total 汇总行。cargo check --offline、cargo build --offline、修改文件 rustfmt 检查与 git diff --check 通过；未新增测试用例、未操作 GUI。

- 首次跳转延迟已按日志定位：索引就绪后的 initialize 首次解析约 1703–1713ms，后续约 6ms，目标编辑器初始化约 250µs。协议诊断中，提前 didOpen 并执行当前文件 semanticTokens/full 分析约 1739ms，随后相同定义请求约 0.7ms；该耗时仅代表本机这次诊断。
- Rust 文档打开后由 Application 后台同步并预热语义缓存；预热期间状态栏显示“LSP 预热中”，完成后才恢复就绪。已预热且内容相同的文档跳过重复分析，排队的旧快照不会覆盖交互请求同步的新文本。新增文档预热与定义请求分阶段耗时日志。

- 底部状态栏由 25px 加高到 32px；工作区 GUI 初始化后经 cx.spawn → run_application → Tokio 后台启动并复用 rust-analyzer 会话。Infrastructure 发布语言服务状态，Application 订阅并回写 DataContext，状态栏显示启动、索引、就绪、警告、异常和停止状态，悬停可查看详情。

- Rust 编辑器新增 Ctrl 悬停名称时的小手光标；移开名称或松开 Ctrl 恢复普通光标，不显示下划线。使用编辑器已排版的字符边界判断命中，不修改依赖、不额外请求 LSP；跳转继续沿用现有 Ctrl+左键逻辑。
- 小手光标变更已通过 cargo check --offline、Rust 2024 格式检查、git diff --check 及只读审查。默认构建的 exe 被占用，改用 cargo rustc --offline --bin gpui_idea -- -o target/debug/gpui_idea-cursor-check.exe 完成独立输出编译与链接；未操作 GUI，实际交互待验证。
- Rust 已接入成熟 rust-analyzer LSP 客户端，Ctrl+左键沿用原生鼠标定位后异步跳转函数、方法及类型定义；多个结果弹出候选，项目外标准库/依赖源码只读打开并显示只读状态。当前仅启用 Rust，其他语言不启动语言服务。
- LSP 按工作区复用常驻进程，协商 UTF-16、按服务能力增量同步、覆盖未保存缓冲；独立请求取消令牌处理光标移动、选区、失焦与较新请求，避免旧取消误伤新请求。通信帧由独立 writer 完整写出；首次索引等待就绪，ContentModified 等暂态响应有界重试，退出时关闭服务。
- 已安装官方 rust-analyzer 组件（rust-src 原已安装），修正 cargo.sysroot 为 discover。实际客户端协议诊断已定位 HashSet 到 Rust 标准库 set.rs 第 126 行，并在含 emoji 的增量编辑后定位相同目标；项目内 go_to_definition 调用定位到 core/definition.rs。最后一轮首次索引约 24.7 秒，已启动会话内 HashSet 增量请求约 0.155 秒、项目函数约 2.17 秒；这些是本机特定请求数据，不代表 GUI 帧率或其他项目性能。
- Rust 跳转最终 cargo build --offline、cargo check --offline 已通过，已修复只读审查指出的点击坐标、异步抢焦点及取消代次竞态；Windows LSP URI 大小写/verbatim 前缀与工作区快照路径对齐，避免项目文件被误认为外部源码。未操作 GUI。

- 代码提示已接入语言关键字与当前文档附近标识符，100ms 防抖，经 Application 在后台收集、去重、排序；补全范围适配当前 GPUI 按 Unicode 标量计算列的实现，替换当前完整单词。项目内下拉组件通过请求代次、光标、选区、焦点和可编辑状态校验丢弃过期结果。
- 补全菜单改为 component/code_editor/completion 项目内独立组件，按 core/ui/external/popup 分工，参考 IDEA 浅色样式：白色浮层、灰色选中行、Lucide 类型图标、蓝色匹配前缀、弱化来源说明、底部快捷键提示。自动触发及 Ctrl+Space 手动触发，Enter/鼠标确认、↑↓ 切换、Esc 关闭；Tab 保留缩进。光标定位使用公开 prepaint 几何，浮层靠右时收回窗口内、底部空间不足时向上展开。
- 按用户要求删除整个 vendor 目录和 Cargo 补丁配置；gpui-component、gpui-base 均使用 registry 原始依赖，不再修改依赖源码。下列旧排查记录涉及的 vendor 补丁已不在当前源码中。

- 已按用户要求对照 Zed 官方 main 源码：Zed 的 on_moved 同样调用 bounds_changed→refresh，Windows 拖动定时绘制及项目树 uniform_list 与本项目一致，不能把“不重绘”说成 Zed 流畅的原因。已撤回此次新增的 gpui-pre 依赖补丁并删除此次创建的 vendor/gpui-pre，原有 vendor/gpui-base 未改动。
- 对照 Zed Cargo.toml 中 dev.package 的 taffy opt-level=3，本项目原来为默认未优化 dev 构建。当前仅对齐 taffy 布局引擎的开发编译优化，不改 GPUI 源码；是否为卡顿的主要原因仍需相同场景日志验证。参考：https://github.com/zed-industries/zed/blob/main/Cargo.toml 、https://github.com/zed-industries/zed/blob/main/crates/gpui/src/window.rs 。
- 拖动诊断改为边界变化后由单个 1 秒异步定时任务汇总，不依赖绘制回调发日志，无绘制时也能记录 moves>0、frames=0；保留用于后续前后对照。

- 项目树进一步定位到透明拖放提示的重复开销：每个文件/目录行始终创建“目标：目录名”文本、文件夹图标及容器，仅 opacity(0) 隐藏；GPUI 仍执行布局、预绘制和绘制，uniform_list 在移动刷新时还会重新测量样本行及布局可见行。现改为 cx.has_active_drag() 时才创建提示子树；GPUI 开始拖放会 refresh，保留原有 group_drag_over 显示规则及行交互。仅修改这一变量，等待真实拖动前后对照确认改善幅度。

- 已读取两次真实拖动日志（11:12:50–52、11:13:05–09）：共 284 个记录的 CPU 绘制样本全部超过 120 Hz 的 8,333 µs 预算，纯移动 resizes=0。各批次 CPU 最大耗时 22.1–36.7 ms，界面构建最大仅 0.65–1.19 ms；项目树 prepaint 最大 5.19–8.59 ms、paint 最大 3.62–7.18 ms，高于编辑器对应阶段。当前确认热点在布局/预绘制/绘制，项目树为已测子树中的优先排查对象，尚未证明单一根因或 GPU/Present 开销；不同阶段最大值可能来自不同帧，不相加。

- 窗口拖动卡顿排查：已核对日志与原生拖动链路，纯位置变化不会触发工作区自动保存；快照读取为 Arc 克隆，项目树使用可见行虚拟渲染。GPUI 的移动触发刷新是调用路径，尚不能认定为根因。已增加工作区 CPU 阶段及编辑器/项目树子树计时，移动期间汇总输出 `window movement CPU diagnostics`，不包含 GPU/Present 耗时；子树 layout_interval 包含等待其他布局的时间，不能当作子树独占耗时。

- 已定位编辑器横向滚到最右仍截断行尾：gpui-base 0.7.3 仅按 UTF-8 字节最长行估算滚动宽度，design-qa.md 的第 62 行实际比字节最长的第 20 行更宽。已通过项目内 vendor 补丁，以已渲染可见行的真实宽度扩展并缓存代码编辑器滚动范围，增加修正日志；仅改动依赖的一处 Rust 源码，cargo build --offline 已通过，未操作 GUI。
- 编辑器已按 Cargo.toml 的 15 个独立语言特性接入 C、C++、CMake、Java、Python、Go、JavaScript/JSX、TypeScript、TSX、Rust、TOML、YAML、Markdown、HTML、CSS，并识别组件自带 JSON；扩展名与状态栏标签共用语言表，支持 CMakeLists.txt。
- 已补齐上游 C++ 查询的 C 基础规则、TSX 查询的 TypeScript/JavaScript 基础规则、JSX 标签及 CMake 高亮，扩展 IDEA Light 的 HTML/Markdown 配色；cargo build --offline 和新增查询的语法检查已通过，未新增项目测试用例或操作 GUI。
- 已核对 alacritty_terminal 0.26.0 关闭链路并在 Windows 10.0.26200 实际运行对照：单独 Shutdown 能结束普通控制台子进程，GUI 子进程和调用 FreeConsole 的子进程仍存活；现有 ProcessJob 加 Shutdown 的三种场景均退出。记录在 target/terminal-shutdown-probe/results.log，未修改终端业务代码或依赖，验证进程已清理。
- Python、JavaScript、Go 已接入现有 IDEA Light 配色与按扩展名识别，补充各语言的捕获分类适配和状态栏标签。
- 新增语言的 cargo check --offline、cargo build --offline、格式检查与只读代码审查通过；未运行 GUI 验证。
- 已接入 Rust 文件识别与 IDEA Light 风格高亮，启用 GPUI Kit 的 tree-sitter-rust；未支持的扩展名暂按纯文本显示，状态栏显示当前语言。
- Rust 配色集中在 assets/editor/idea-light.json；已核对主题字段及语法捕获适配，cargo check --offline、cargo build --offline、格式检查与只读代码审查通过。
- 文档已改为 64 KiB 缓冲块读取、增量 UTF-8 解码及换行归一化；取消文档读取和保存的 2 MiB 限制，保留 50,000 行限制。
- 保存直接遍历 Rope 块编码，复用现有分块基线校验与差异写入，移除保存前整文件读取及全文字符串转换。
- 分块读写代码审查、rustfmt 检查、git diff --check 和 cargo check --offline 已通过；未新增测试或运行 GUI。
- 已编写 README，补充功能、启动方式、架构与数据流说明，并加入 image_example/093944.png 界面预览。
- EventBus 仅保留 refresh_data_context，支持按路径或全量请求刷新。
- Infrastructure 文件监听按原有 3 秒周期合并变化，由定时器直接发布 refresh_data_context。
- Application 订阅刷新请求、执行 IO 并批量更新 DataContext；也提供可等待完成的立即刷新 API。
- 主动新增、删除、重命名、移动、粘贴及保存：Application 在 IO 前向 Infrastructure 登记精确忽略路径，成功后直接更新 DataContext。
- 已移除 FileCommand、GuiHandlerFileChange、project_file_change 和 EventBus GUI 刷新事件及对应订阅任务。
- DataContext 自身持有 watch 通知，GUI 直接订阅 subscribe_changes。
- 主动操作、目录加载与后台刷新共享异步锁；失败或取消时安排磁盘全量核对。
- 监听忽略期内被过滤的路径在操作及宽限期结束后交由定时器核对，避免漏掉同时发生的外部修改。
- cargo check --offline 已通过；没有新增测试用例，也未操作 GUI。

# 当前的计划

- v0.0.1 标签推送后跟踪 GitHub Actions 的三平台构建和 Release 发布结果；若构建失败，根据 Actions 日志修复。本次不在本地重新编译 GUI。

- 跨平台终端待 macOS/Linux 实机核对登录 Shell、Finder 启动的工具链 PATH、前后台/挂起任务关闭与界面交互；当前交叉编译仅覆盖终端及运行参数模块，未验证整个 GUI 应用的 macOS/Linux 构建。手工保存的 PowerShell 命令需按新系统 Shell 语法调整，不自动改写用户命令；主动 setsid 脱离终端会话的守护进程不在 Unix 会话清理范围内。

- 代码提示滚动条待运行 GUI 核对：长列表持续可见、拖动和滚轮滚动、键盘选中跟随以及短列表不显示滚动条；完整编译需先解决现有 event_bus 编译错误。

- 本次组件拆分待人工核对：侧栏开关与分栏拖动、文件双击/搜索/导航、移动/删除后的标签和路径、多终端切换、运行配置、自动保存及关闭恢复；未执行 GUI 交互验证。

- 本次 LSP 已按最终要求直接匹配用户 SDK 支持的服务器版本，保留启动更新检查；仅无兼容版本或缺失 SDK 时提示用户处理。安装链路及实际协议初始化验证已完成；GUI 中安装状态与匹配说明留待实际使用核对。

- Rust 从全新缓存真实在线下载 16.8 MiB 官方压缩包、解压、初始化通过；实际进程路径位于 tools/managed/rust-analyzer，版本由服务器返回 0.3.3081-standalone，安装阶段库存快照已连续记录下载量、目标目录及 Installing 状态，成功后才变为 Ready。cargo check --offline、修改文件 rustfmt、修改范围 diff 检查和独立编译链接通过，输出 target/debug/gpui_idea-lsp-install-popup.exe。浮动卡片位置、长路径换行及首次安装/失败时的实际 GUI 显示待使用新版程序核对。

- LSP 在线安装进一步简化：node.rs 已移除全部 include_str/package 清单与锁文件引用，删除清单摘要和 sha2 直接依赖；首次使用直接调用系统 Node/npm install 对应服务器包，npm 在 tools/managed/lsp-语言 中生成 package.json/package-lock.json。tools/lsp 源目录不存在时 cargo check --offline、rustfmt 和修改范围 diff 检查均通过；真实 Pyright 在线安装、运行时配置生成与初始化通过。最新版 typescript-language-server@6.0.2 要求 Node >=22.22.2，本机 v22.19.0 被 npm engine 检查拒绝，已修正提示，需用户自行升级 Node 后重试，不自动安装运行时。未新增测试用例、未操作 GUI。

- Node/npm 按需安装与统一路径已完成：cargo check --offline、修改文件 rustfmt 检查、修改范围 git diff --check、独立编译链接（target/debug/gpui_idea-node-lsp.exe）通过。使用真实模块的临时协议诊断在全新目录完成 TypeScript、Pyright、HTML/CSS/JSON、YAML 在线安装并初始化，重复启动直接复用缓存；隐藏系统 Node 后正确返回自行安装提示。关闭 debug_assertions 的诊断程序从不同工作目录运行，data/assets/logs/tools 均定位 exe 同级，并独立完成 TypeScript 在线安装和初始化。修复 Windows 安装清单句柄未关闭导致目录提交失败；未新增项目测试用例、未使用 computer use。GUI 安装状态与外部资源覆盖待实际使用核对。仅 Node 系 LSP 包由系统 Node/npm 安装，不下载 Node、其他 SDK/运行时或原生服务器。

- 本次 LSP 补齐已通过 cargo check --offline、修改文件 rustfmt 检查、修改范围 git diff --check 和独立编译链接，输出 target/debug/gpui_idea-lsp-complete.exe。通过临时命令进行了真实后端协议诊断：TS/TSX 返回 pages/title 成员，HTML/CSS/JSON 在启用片段能力后均声明并返回补全，JSON 本地 schema 返回 enabled/title，YAML 初始化与补全请求正常响应。未编写测试用例、未使用 computer use。新增原生后端本机未安装，启动与实际工程解析待安装后验证；所有语言 GUI 候选插入、跨文件 Ctrl+点击和状态详情仍待实际使用验证。

- 历史精简已完成现有数据迁移、7 个项目状态完整性与重复读取核对；cargo check --offline、修改文件 rustfmt 检查、修改范围 git diff --check 和独立编译链接（target/debug/gpui_idea-history-compact.exe）通过。待新版 GUI 实际核对最近项目列表、多项目切换恢复、标签/展开目录/窗口布局恢复及后续自动保存。

- 路径复制菜单通过 cargo check --offline 和修改范围 git diff --check，待 GUI 实际核对文件、目录、项目根目录和多选复制结果。

- 图片预览通过 cargo check --offline、修改文件 rustfmt 检查、修改范围 git diff --check 和独立编译链接（target/debug/gpui_idea-image-preview.exe）；等待 GUI 实际核对 PNG/JPEG/GIF/WebP/SVG 等格式、透明图片、窗口适配、损坏/超限提示、外部文件拖入、文件更新及移动删除后的标签状态。不新增测试用例、不使用 computer use。

- 文件双击打开待 GUI 实际核对单击不打开、双击打开，以及 Ctrl 多选后拖动仍保持整组选择。

- 项目树多选通过 cargo check --offline、修改文件格式检查、修改范围 git diff --check 和独立编译链接（target/debug/gpui_idea-project-multiselect.exe）；待实际 GUI 核对 Ctrl 选择/取消、跨目录整组拖放、根目录投放、Delete 焦点隔离、批量删除确认、重名和未保存文件保护、父子去重与部分失败反馈。不新增测试用例、不使用 computer use。

- 设置窗口布局通过 cargo check --offline、修改文件格式检查、修改范围 git diff --check 和独立编译链接（target/debug/gpui_idea-settings-layout.exe）；等待 GUI 实际核对分栏比例、分类滚动、选中状态及底部固定操作栏。本次仅完成布局，不实现具体设置功能。

- 外部源码标签浅黄色背景等待 GUI 实际核对普通、悬停和拖动状态。

- LSP 悬浮详情等待 GUI 实际核对悬停位置、四语言版本及服务退出/重启刷新；cargo check --offline、修改文件 rustfmt 检查、修改范围 git diff --check 和独立输出编译链接通过，可执行文件为 target/debug/gpui_idea-lsp-details.exe。构建仅提示自定义输出参数及增量缓存硬链接回退复制，未新增测试用例、未使用 computer use。

- 源码标签颜色区分等待 GUI 实际核对：项目标签保留原色，外部标准库/依赖标签、选中下划线及拖动预览显示蓝色，悬停显示只读说明；不新增测试用例、不使用 computer use。

- IDEA 风格新建项目窗口已由用户验收，本期五种语言的创建入口完成，不再调整已验收界面。
- Rust 标准库与依赖补全、默认安静日志，以及 Python、JavaScript、Go 语义补全已完成窗口验证；本期四语言接入完成，其他语言后续再支持。

- 统计表虚拟滚动等待实际 GUI 使用验证，重点检查长列表滚动、横向列对齐和扫描后数据减少时的滚动位置。

- 首次跳转预热已通过 cargo check --offline、独立可执行文件编译链接（target/debug/gpui_idea-lsp-warm.exe）、修改文件格式检查与差异空白检查。只读审查发现的关闭时预热阻塞已修复并复核：点击跳转或关闭应用可取消后台预热，预热有总超时限制，关闭后排队任务不会重新启动进程。协议诊断结果已记录，实际 GUI 交互待验证；未新增测试用例、未使用 computer use。

- LSP 启动与状态栏变更已通过 cargo check --offline、独立输出编译链接（target/debug/gpui_idea-lsp-status.exe）、修改文件 rustfmt 检查、修改范围 git diff --check 和只读自查；全局 diff 检查发现 main.rs 原有空白行含尾随空格，本次未改动。未新增测试用例、未使用 computer use，实际状态栏布局和启动状态变化等待 GUI 使用验证。

- Rust Ctrl+左键跳转等待实际 GUI 验证：项目函数、HashSet 等标准库类型、Cargo 依赖源码、多目标选择、后退导航和只读状态。首次索引在后台执行，但仍消耗语言服务进程的 CPU/内存；不新增项目测试用例、不使用 computer use，其他语言暂不接入源码跳转。

- 独立代码提示组件已通过 cargo check --offline、最终 cargo build --offline、修改文件 rustfmt 检查、git diff --check 和只读审查；Cargo.lock 与 cargo tree 确认使用 registry 的 gpui-component 0.7.3，vendor 目录不存在。文档变化、失焦、移动光标和外部文件重载会取消旧候选。等待 GUI 实际使用验证，不新增测试用例、不使用 computer use。当前为本地词法补全，未接入 LSP 语义分析或跨文件候选。

- 仅优化 taffy 的开发版本已通过 cargo build --offline、格式检查及只读审查，确认使用原始 registry GPUI。待保持相同目录展开、窗口尺寸及文件，对比 tree_prepaint_max_us 和 cpu_max_us。不能直接把未优化 dev 的数据与安装版 Zed 的流畅度当作同条件对照。不新增测试用例、不使用 computer use。

- 项目树透明拖放提示优化已通过 cargo build --offline、rustfmt --check 和 git diff --check；未新增测试或操作 GUI。待复现相同窗口大小与目录展开状态的窗口拖动，对比 tree_prepaint_max_us、tree_paint_max_us、cpu_max_us，另核对文件拖放时目标提示仍显示。

- 拖动诊断代码的 cargo build --offline 与修改文件格式检查已通过。待使用新构建手动复现后读取日志，依据编辑器/项目树 prepaint、paint 与工作区总耗时继续定位；终端逐单元格文字排版是待验证疑点，暂不修改渲染行为。不新增测试用例、不使用 computer use。

- 编辑器行尾截断修复已完成构建验证；实际 GUI 核对待运行更新后的程序，不新增测试用例、不使用 computer use。
- 独立语言特性的识别、高亮与状态栏适配已完成，等待实际 GUI 使用验证；不新增测试用例、不使用 computer use。
- 分块文档读写等待实际 GUI 使用验证；当前仍保留全文及磁盘基线，尚非按需分页加载。
- 只读代码审查已完成，发现的旧快照覆盖与忽略窗口漏事件已处理。
- 最终编译、格式与残留引用检查通过；等待实际 GUI 使用验证。
