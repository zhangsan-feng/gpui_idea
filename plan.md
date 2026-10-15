# 当前完成的进度

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
