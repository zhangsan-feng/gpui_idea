
## 项目约束
禁止编写superpowers 的计划 禁止使用computer use 除非用户本次会话指定使用
禁止修改READNE.md 除非用户本次会话明确说明修改
不要编写测试用例这是gui 项目 单例测试不出整体问题

UI 样式参考 [Tailwind CSS](https://tailwindcss.com/)
使用 Product Design 规范来约束视觉层次和交互状态
图标设计与语义参考 [Lucide](https://lucide.dev/)；优先复用项目已有的 Lucide 图标资源，避免引入重复图标依赖。

明确的需求不要设计方案直接开始执行

个责任边界
core.rs (核心功能 数据流向)
ui.rs渲染
external.rs(给外部用的)
mod.rs(类型定义,子模块,初始化与 Render 入口, start_subscribe, init_component_data 等)
涉及到在gui操作io的时候应该把gui 线程让出去使用 cx.spawn  + tokio::spawn 操作数据

单个文件行数代码维护在600-800行 超过800行 
就按照目录拆分 目录里面按照功能拆分文件
把功能文件改成目录 比如 core.rs 改成core 文件夹里面按照功能拆分

http服务 使用 get 和 post 禁止使用其他的rest 规范

数据流向约束   
infrastructure 提供底层能力
data_context 内存数据存储 
application 功能编排 
gui 交互层
禁止在gui 层编写数据交互功能逻辑

## 定位问题 定位问题优先看日志 没有日志添加日志排查
## 编写计划文档 plan.md
当前完成的进度
当前的计划
