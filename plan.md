# 当前完成的进度

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

- 只读代码审查已完成，发现的旧快照覆盖与忽略窗口漏事件已处理。
- 最终编译、格式与残留引用检查通过；等待实际 GUI 使用验证。
