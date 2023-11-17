extern crate alloc;
use alloc::vec::Vec;

#[derive(Copy, Clone)]
#[repr(C)]
/// task context structure containing some registers
pub struct TaskContext {
    /// Ret position after task switching
    ra: usize,
    /// Stack pointer
    sp: usize,
    /// s0-11 register, callee saved
    s: [usize; 12],
}

impl TaskContext {
    /// Create a new empty task context
    pub const fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}

extern "C" {
    /// Switch to the context of `next_task_cx_ptr`, saving the current context
    /// in `current_task_cx_ptr`.
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}

pub struct AppManager {
    num_app: usize,
    current_id: usize,
    app_start: [usize;20],
    tc: [TaskContext; 10],
}

impl AppManager {
    /// Create a new page table
    pub const fn new() -> Self {
        AppManager {
            num_app:0,
            current_id:0,
            app_start:[0;20],
            tc:[TaskContext::zero_init();10],
        }
    }
    pub fn update_num_app(&mut self, num_app:usize) {
        self.num_app = num_app;
    }
    pub fn get_num_app(&self) -> usize{
        self.num_app
    }
    pub fn update_app_start(&mut self, app_start:Vec<usize>) {
        for i in 0..app_start.len() {
            self.app_start[i] = app_start[i];
        }
        
    }
    pub fn get_app_start(&self,index:usize) -> usize{
        self.app_start[index]
    }
    pub fn update_current_id(&mut self, current_id:usize) {
        self.current_id = current_id;
    }
    pub fn get_current_id(&self) -> usize{
        self.current_id
    }
}
