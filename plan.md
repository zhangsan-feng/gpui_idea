# 当前完成的进度

- 已确认：EventBus 只负责 refresh_data_context；GUI 订阅 DataContext 自身的变更通知。
- 已替换事件入口，GUI 文件操作直接异步调用 Application。

# 当前的计划

- Application 在主动文件 IO 前登记监听忽略范围，操作失败或取消时安排磁盘核对。
- 定时器统一发布 refresh_data_context，Application 读取数据并批量更新 DataContext。
- 完成编译、格式和调用链检查，不新增测试用例。
