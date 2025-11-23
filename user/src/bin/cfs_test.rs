#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fork, get_time, getpid, waitpid, yield_, exit};

const WORK_ITERATIONS: usize = 5000000;
const NUM_WORKERS: usize = 4;

/// 执行计算密集型工作
fn do_work(iterations: usize) -> usize {
    let mut sum: usize = 0;
    for i in 0..iterations {
        sum = sum.wrapping_add(i);
        // 每隔一段时间主动让出 CPU，让其他进程有机会运行
        if i % 50000 == 0 {
            yield_();
        }
    }
    sum
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("========================================");
    println!("CFS Scheduler Test");
    println!("========================================");
    println!("Testing fairness with {} worker processes", NUM_WORKERS);
    println!("Each process will perform {} iterations", WORK_ITERATIONS);
    println!("");

    let start_time = get_time() as usize;
    
    // 创建工作进程
    let mut worker_pids = [0usize; NUM_WORKERS];
    let mut fork_count = 0;
    
    for i in 0..NUM_WORKERS {
        let pid = fork();
        if pid == 0 {
            // 子进程：执行计算密集型工作
            let worker_start = get_time() as usize;
            println!("[Worker {}] pid={}, starting work...", i, getpid());
            
            let result = do_work(WORK_ITERATIONS);
            let worker_end = get_time() as usize;
            let worker_time = worker_end - worker_start;
            
            println!("[Worker {}] pid={}, completed! result={}, time={}ms", 
                     i, getpid(), result, worker_time);
            exit(0);
        } else {
            // 父进程：记录子进程 PID
            worker_pids[fork_count] = pid as usize;
            fork_count += 1;
            println!("[Parent] Created worker {} with pid={}", i, pid);
        }
    }

    // 父进程：等待所有子进程完成
    println!("");
    println!("[Parent] Waiting for {} workers to complete...", NUM_WORKERS);
    println!("");
    
    let mut completion_times = [0usize; NUM_WORKERS];
    for i in 0..NUM_WORKERS {
        let pid = worker_pids[i];
        let mut exit_code: i32 = 0;
        let wait_pid = waitpid(pid, &mut exit_code);
        completion_times[i] = get_time() as usize;
        
        if wait_pid > 0 {
            println!("[Parent] Worker with pid={} finished (exit_code={})", pid, exit_code);
        }
    }
    
    let end_time = get_time() as usize;
    let total_time = end_time - start_time;
    
    // 计算统计信息
    println!("");
    println!("========================================");
    println!("Test Results:");
    println!("========================================");
    println!("Total execution time: {}ms", total_time);
    
    // 计算时间差异
    let mut min_time = completion_times[0] - start_time;
    let mut max_time = min_time;
    for i in 1..NUM_WORKERS {
        let t = completion_times[i] - start_time;
        if t < min_time {
            min_time = t;
        }
        if t > max_time {
            max_time = t;
        }
    }
    
    println!("Fastest worker: {}ms", min_time);
    println!("Slowest worker: {}ms", max_time);
    println!("Time difference: {}ms", max_time - min_time);
    
    let fairness_ratio = if max_time > 0 {
        (min_time * 100) / max_time
    } else {
        100
    };
    
    println!("Fairness ratio: {}%", fairness_ratio);
    println!("");
    
    // 评估测试结果
    if fairness_ratio >= 80 {
        println!("✓ PASS: CFS scheduler is working fairly!");
        println!("  All workers completed within reasonable time variance.");
    } else if fairness_ratio >= 60 {
        println!("⚠ WARNING: Fairness could be improved.");
        println!("  Time variance is higher than expected.");
    } else {
        println!("✗ FAIL: Fairness test failed!");
        println!("  Workers show significant time variance.");
    }
    
    println!("========================================");
    println!("");
    
    0
}
