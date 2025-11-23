use super::id::TaskUserRes;
use super::{KernelStack, ProcessControlBlock, TaskContext, kstack_alloc};
use crate::trap::TrapContext;
use crate::{
    mm::PhysPageNum,
    sync::{UPIntrFreeCell, UPIntrRefMut},
};
use alloc::sync::{Arc, Weak};
use core::cmp::Ordering;

pub struct TaskControlBlock {
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    pub inner: UPIntrFreeCell<TaskControlBlockInner>,
}

impl PartialEq for TaskControlBlock {
    fn eq(&self, other: &Self) -> bool {
        self.inner.exclusive_access().vruntime == other.inner.exclusive_access().vruntime
    }
}

impl Eq for TaskControlBlock {}

impl PartialOrd for TaskControlBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskControlBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        // CFS: 选择 vruntime 最小的任务
        // 正向比较：vruntime 小的任务优先级高
        self.inner.exclusive_access().vruntime.cmp(&other.inner.exclusive_access().vruntime)
    }
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> UPIntrRefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access();
        inner.memory_set.token()
    }
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,
    // CFS scheduler fields
    pub vruntime: usize,        // 虚拟运行时间（微秒）
    #[allow(dead_code)]
    pub nice: i8,               // nice 值：-20 到 19，默认 0
    pub load_weight: usize,     // 负载权重，由 nice 值计算
    #[allow(dead_code)]
    pub time_slice: usize,      // 时间片（微秒）
    pub last_scheduled: usize,  // 上次调度时间（用于计算实际运行时间）
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    /// 根据 nice 值计算负载权重（类似 Linux 的 prio_to_weight 表）
    pub fn calc_load_weight(nice: i8) -> usize {
        // Linux 使用的权重表，nice 每增加 1，权重约降低 10%
        // nice=0 的权重为 1024
        const NICE_0_LOAD: usize = 1024;
        const SCHED_PRIO_TO_WEIGHT: [usize; 40] = [
            88761, 71755, 56483, 46273, 36291,  // -20 to -16
            29154, 23254, 18705, 14949, 11916,  // -15 to -11
            9548,  7620,  6100,  4904,  3906,   // -10 to -6
            3121,  2501,  1991,  1586,  1277,   // -5 to -1
            1024,  820,   655,   526,   423,    // 0 to 4
            335,   272,   215,   172,   137,    // 5 to 9
            110,   87,    70,    56,    45,     // 10 to 14
            36,    29,    23,    18,    15,     // 15 to 19
        ];
        let idx = (nice + 20).max(0).min(39) as usize;
        SCHED_PRIO_TO_WEIGHT[idx]
    }

    /// 更新虚拟运行时间
    pub fn update_vruntime(&mut self, delta_time: usize) {
        // vruntime 增量 = 实际运行时间 * NICE_0_LOAD / load_weight
        // 权重越大，vruntime 增长越慢，获得更多 CPU 时间
        #[allow(dead_code)]
        const NICE_0_LOAD: usize = 1024;
        self.vruntime += (delta_time * NICE_0_LOAD) / self.load_weight.max(1);
    }

    /// 计算时间片（类似 Linux 的 sched_slice）
    #[allow(dead_code)]
    pub fn calc_time_slice(&self, nr_running: usize) -> usize {
        const MIN_GRANULARITY: usize = 3000;  // 3ms 最小粒度
        const TARGET_LATENCY: usize = 24000;  // 24ms 目标延迟
        
        let period = if nr_running > 0 {
            TARGET_LATENCY.max(MIN_GRANULARITY * nr_running)
        } else {
            TARGET_LATENCY
        };
        
        // 时间片 = period * (weight / total_weight)
        // 简化：假设每个任务权重相等
        period / nr_running.max(1)
    }
}

impl TaskControlBlock {
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        let nice = 0i8;  // 默认 nice 值
        let load_weight = TaskControlBlockInner::calc_load_weight(nice);
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPIntrFreeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                    vruntime: 0,
                    nice,
                    load_weight,
                    time_slice: 6000,  // 默认 6ms
                    last_scheduled: 0,
                })
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Blocked,
}
