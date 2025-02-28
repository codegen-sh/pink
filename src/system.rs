use sysinfo::System;

pub fn get_memory() -> u64 {
    let s = System::new_all();
    let current = s.process(sysinfo::get_current_pid().unwrap()).unwrap();
    current.memory()
}
