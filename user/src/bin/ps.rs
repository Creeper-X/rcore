#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{get_process_info, ProcessInfo};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("  PID  PPID  STATUS");
    println!("====================");
    
    let mut buf = [ProcessInfo {
        pid: 0,
        ppid: 0,
        status: 0,
    }; 64];
    
    let count = get_process_info(&mut buf);
    
    if count < 0 {
        println!("Error: failed to get process information");
        return -1;
    }
    
    for i in 0..count as usize {
        let info = buf[i];
        let status_str = match info.status {
            0 => "Ready",
            1 => "Running",
            2 => "Zombie",
            _ => "Unknown",
        };
        
        println!("{:5} {:5}  {}", info.pid, info.ppid, status_str);
    }
    
    println!("====================");
    println!("Total: {} process(es)", count);
    
    0
}
