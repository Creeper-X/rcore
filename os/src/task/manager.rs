use super::{ProcessControlBlock, TaskControlBlock, TaskStatus};
use crate::sync::UPIntrFreeCell;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use lazy_static::*;

pub struct TaskManager {
    ready_queue: BTreeMap<usize, Arc<TaskControlBlock>>,  // key: task id（用于区分相同 vruntime 的任务）
    task_count: usize,  // 用于生成唯一的 task id
    total_weight: usize,  // 所有任务的权重总和
}

/// CFS (Completely Fair Scheduler) - Linux-like scheduler.
/// 使用红黑树（BTreeMap）维护按 vruntime 排序的就绪队列。
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: BTreeMap::new(),
            task_count: 0,
            total_weight: 0,
        }
    }
    
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        let task_inner = task.inner.exclusive_access();
        self.total_weight += task_inner.load_weight;
        drop(task_inner);
        
        // 使用递增的 task_count 作为 key 确保唯一性
        self.ready_queue.insert(self.task_count, task);
        self.task_count += 1;
    }
    
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // BTreeMap 按 vruntime（通过 Ord trait）排序，first_entry 获取最小的
        if let Some((&key, _)) = self.ready_queue.iter().min_by(|(_, a), (_, b)| a.cmp(b)) {
            let task = self.ready_queue.remove(&key).unwrap();
            let task_inner = task.inner.exclusive_access();
            self.total_weight = self.total_weight.saturating_sub(task_inner.load_weight);
            drop(task_inner);
            Some(task)
        } else {
            None
        }
    }
    
    #[allow(dead_code)]
    pub fn get_nr_running(&self) -> usize {
        self.ready_queue.len()
    }
    
    #[allow(dead_code)]
    pub fn get_total_weight(&self) -> usize {
        self.total_weight
    }
    
    pub fn get_min_vruntime(&self) -> usize {
        // 返回队列中最小的 vruntime（用于新任务初始化）
        self.ready_queue
            .values()
            .map(|t| t.inner.exclusive_access().vruntime)
            .min()
            .unwrap_or(0)
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPIntrFreeCell<TaskManager> =
        unsafe { UPIntrFreeCell::new(TaskManager::new()) };
    pub static ref PID2PCB: UPIntrFreeCell<BTreeMap<usize, Arc<ProcessControlBlock>>> =
        unsafe { UPIntrFreeCell::new(BTreeMap::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn wakeup_task(task: Arc<TaskControlBlock>) {
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}

#[allow(dead_code)]
pub fn get_nr_running() -> usize {
    TASK_MANAGER.exclusive_access().get_nr_running()
}

pub fn get_min_vruntime() -> usize {
    TASK_MANAGER.exclusive_access().get_min_vruntime()
}

pub fn pid2process(pid: usize) -> Option<Arc<ProcessControlBlock>> {
    let map = PID2PCB.exclusive_access();
    map.get(&pid).map(Arc::clone)
}

pub fn insert_into_pid2process(pid: usize, process: Arc<ProcessControlBlock>) {
    PID2PCB.exclusive_access().insert(pid, process);
}

pub fn remove_from_pid2process(pid: usize) {
    let mut map = PID2PCB.exclusive_access();
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}
