

#![allow(dead_code)]
use crate::{Result as HyperResult, Error as HyperError};

use super::VirtMsrDevice;
const MSR_IA32_UMWAIT_CONTROL_TIME_MASK:u32	= !3;
const MSR_IA32_UMWAIT_CONTROL_C02_DISABLE:u32 = 1 << 0;

pub struct Ia32UmwaitControl {
	value: u32,
}

impl Ia32UmwaitControl {
    pub fn new(max_time: u32, c02_disable: u32) -> Self {
        Self {
			value: ((max_time) & MSR_IA32_UMWAIT_CONTROL_TIME_MASK) | ((c02_disable) & MSR_IA32_UMWAIT_CONTROL_C02_DISABLE),
		}
    }
}

impl VirtMsrDevice for Ia32UmwaitControl {
    fn msr_range(&self) -> core::ops::Range<u32> {
        0xe1..(0xe1 + 1)
    }

    fn read(&mut self, _msr: u32) -> HyperResult<u64> {
		info!("read Ia32UmwaitControl: {}", self.value);
        Ok(self.value as u64)
    }

    fn write(&mut self, _msr: u32, value: u64) -> HyperResult {
		info!("write Ia32UmwaitControl:{}", value);
		self.value = value as u32;
        Ok(())
    }
}