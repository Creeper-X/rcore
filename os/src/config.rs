#[allow(unused)]

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

// CFS Scheduler configuration
#[allow(dead_code)]
pub const SCHED_MIN_GRANULARITY: usize = 3000;   // 3ms - 最小调度粒度（微秒）
#[allow(dead_code)]
pub const SCHED_TARGET_LATENCY: usize = 24000;  // 24ms - 目标调度延迟（微秒）
#[allow(dead_code)]
pub const SCHED_NICE_0_LOAD: usize = 1024;      // nice=0 时的基准权重

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - PAGE_SIZE;

pub use crate::board::{CLOCK_FREQ, MEMORY_END, MMIO};
