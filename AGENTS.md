
## 项目约束
禁止编写superpowers 的计划 禁止使用computer use 除非用户本次会话指定使用
UI 样式参考 [Tailwind CSS](https://tailwindcss.com/)
使用 Product Design 规范来约束视觉层次和交互状态
图标设计与语义参考 [Lucide](https://lucide.dev/)；优先复用项目已有的 Lucide 图标资源，避免引入重复图标依赖。

不要编写测试用例这是gui 项目 单例测试不出整体问题

gui 模块 个责任边界
core.rs (核心功能 数据流向)
ui.rs渲染
external.rs(给外部用的)
mod.rs(类型定义,子模块,初始化与 Render 入口, start_subscribe, init_component_data 等)
涉及到在gui操作io的时候应该把gui 线程让出去使用 cx.spawn  + tokio::spawn 操作数据

应用和基础设施 责任边界 
core.rs (核心功能) 
external.rs(给外部用的) 
mod.rs(类型定义、子模块声明、初始化)

单个文件行数代码维护在600-800行 超过800行 
就按照目录拆分 目录里面按照功能拆分文件
把功能文件改成目录 比如 core.rs 改成core 文件夹里面按照功能拆分

服务端 按照ddd的架构设计实现 http服务 使用 get 和 post 禁止使用其他的rest 规范



## 定位问题 定位问题优先看日志 没有日志添加日志排查
## 编写计划文档 plan.md
当前完成的进度
当前的计划

## 项目架构

- `main.rs` 启动 GPUI，`application::lifecycle` 统一编排应用服务初始化与关闭。
- `gui` 负责交互与渲染，`component` 提供通用组件；MCP 入口在 `infrastructure::agent_mcp`，业务命令桥接到 Application。
- `application`（`APPLICATION`）负责参数校验、资源存在性检查、业务编排和运行任务；`domain` 定义共享业务模型。
- `infrastructure`（`INFRASTRUCTURE`）提供 SQLite、配置文件、本地文件、SSH/SFTP、代理等底层 IO 能力。
- `data_context`（`DATA_CONTEXT`）保存工作区、终端、SFTP、服务状态的内存快照与版本，不执行 IO。
- 持久化读写：`GUI/MCP → Application → Infrastructure → SQLite/配置文件`。
- 热数据写入：`GUI/MCP → Application → DataContext`；后台 IO 结果经 Application 回写 DataContext。
- 热数据读取：`GUI/MCP → DataContext`，直接读取快照。
- 状态通知：DataContext 更新后通过 `gui_refresh_notification` 通知 GUI 重读快照；`session_lifecycle_events` 向 MCP 发布会话打开、关闭、选择事件。
- GUI 的 IO 操作使用 `cx.spawn → run_application → tokio::spawn`；阻塞 IO 在 Infrastructure 使用 `spawn_blocking`。
- 直接复用现有具体模块，不新增 trait 接口复杂化架构。
