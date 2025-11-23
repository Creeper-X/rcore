# CFS 调度器测试指南

## 测试程序：cfs_test

### 功能说明

`cfs_test` 是一个用于测试 CFS (Completely Fair Scheduler) 调度器公平性的程序。

### 测试原理

1. **创建多个工作进程**：程序创建 4 个子进程
2. **执行相同工作量**：每个进程执行 5,000,000 次迭代的计算密集型任务
3. **周期性让出 CPU**：每 50,000 次迭代主动调用 `yield_()` 让出 CPU
4. **测量完成时间**：记录每个进程的开始和结束时间
5. **计算公平性**：比较各进程的执行时间差异

### 运行方法

#### 1. 在 QEMU 中运行

```bash
cd /home/artorias/rcore/os
make run
```

系统启动后，在用户 shell 中输入：

```bash
>> cfs_test
```

#### 2. 预期输出

```
========================================
CFS Scheduler Test
========================================
Testing fairness with 4 worker processes
Each process will perform 5000000 iterations

[Parent] Created worker 0 with pid=3
[Parent] Created worker 1 with pid=4
[Parent] Created worker 2 with pid=5
[Parent] Created worker 3 with pid=6

[Parent] Waiting for 4 workers to complete...

[Worker 0] pid=3, starting work...
[Worker 1] pid=4, starting work...
[Worker 2] pid=5, starting work...
[Worker 3] pid=6, starting work...
[Worker 0] pid=3, completed! result=..., time=XXXms
[Worker 1] pid=4, completed! result=..., time=XXXms
[Worker 2] pid=5, completed! result=..., time=XXXms
[Worker 3] pid=6, completed! result=..., time=XXXms
[Parent] Worker with pid=3 finished (exit_code=0)
[Parent] Worker with pid=4 finished (exit_code=0)
[Parent] Worker with pid=5 finished (exit_code=0)
[Parent] Worker with pid=6 finished (exit_code=0)

========================================
Test Results:
========================================
Total execution time: XXXms
Fastest worker: XXXms
Slowest worker: XXXms
Time difference: XXXms
Fairness ratio: XX%

✓ PASS: CFS scheduler is working fairly!
  All workers completed within reasonable time variance.
========================================
```

### 评估标准

程序会自动评估调度器的公平性：

- **Fairness ratio ≥ 80%**: ✓ PASS - 调度器工作良好
- **Fairness ratio ≥ 60%**: ⚠ WARNING - 公平性有待改进
- **Fairness ratio < 60%**: ✗ FAIL - 公平性测试失败

**公平性比例计算**：
```
Fairness ratio = (最快进程时间 / 最慢进程时间) × 100%
```

### 测试参数调整

如需调整测试参数，可以修改 `user/src/bin/cfs_test.rs` 中的常量：

```rust
const WORK_ITERATIONS: usize = 5000000;  // 每个进程的工作量
const NUM_WORKERS: usize = 4;            // 工作进程数量
```

然后重新编译：

```bash
cd /home/artorias/rcore/user
cargo build --release
```

### CFS 调度器特性验证

此测试验证了以下 CFS 特性：

1. **公平性 (Fairness)**
   - 所有进程获得大致相等的 CPU 时间
   - vruntime 机制确保公平调度

2. **响应性 (Responsiveness)**
   - 进程能够快速响应和切换
   - 周期性 yield 测试调度延迟

3. **负载均衡**
   - 多个进程并发运行时的时间分配
   - 确保没有进程被饿死

### 故障排除

如果测试失败或结果异常：

1. **时间差异过大**
   - 检查 vruntime 更新逻辑
   - 验证任务比较函数（Ord trait）

2. **进程卡死**
   - 检查调度队列是否正常工作
   - 验证 fetch/add 操作的正确性

3. **时间测量不准确**
   - 确认 `get_time()` 系统调用正常工作
   - 检查时间单位转换

### 高级测试

#### 手动验证 vruntime

可以在内核中添加调试输出来观察 vruntime 的变化：

```rust
// 在 manager.rs 的 fetch() 中添加
info!("[CFS] Fetching task with vruntime={}", task_inner.vruntime);
```

#### 测试不同工作负载

修改 `do_work()` 函数来测试不同类型的工作负载：

- **纯计算**：不调用 yield_()
- **I/O 密集**：频繁调用 yield_()
- **混合负载**：部分计算 + 部分 I/O

## 相关文档

- [CFS_SCHEDULER_DESIGN.md](../CFS_SCHEDULER_DESIGN.md) - CFS 调度器设计文档
- [进程调度.md](进程调度.md) - 进程调度实现文档
