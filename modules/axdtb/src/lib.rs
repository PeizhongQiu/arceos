#![no_std]

//! # dtb_parser
//!
//! Parses a device tree blob to a human-friendly data structure.
//!
//! The no [std] but [alloc] library is required.
// #[macro_use]
// extern crate axlog;

extern crate alloc;

pub use tree::DeviceTree;
use alloc::vec::Vec;
use error::Result;
use error::DeviceTreeError;
use prop::PropertyValue;
use core::option::Option::Some;
use core::result::Result::Err;

mod byte_util;
mod header;

/// `DeviceTree`
pub mod tree;
/// `DeviceTreeError`
pub mod error;
/// `DeviceTreeNode`
pub mod node;
/// `NodeProperty`
pub mod prop;

/// DtbInfo 
pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}
/// parse_dtb
pub fn parse_dtb(dtb_pa: usize) -> Result<DtbInfo> {
    let mut memory_addr = 0;
    let mut memory_size = 0;
    let tree = DeviceTree::from_address(dtb_pa).unwrap();
    let mut ok = false;
    if let Some(memory) = tree.root().find_child_type("memory") {
        
        if let Some(reg_prop) = memory[0].of_value("reg") {
            
            if let PropertyValue::Address(a,b) = reg_prop {
                
                memory_addr = *a as usize;
                // info!("{:#x}",memory_addr);
                memory_size = *b as usize;
                ok = true;
            
            }
            
        }

    } 
    if !ok {
        return Err(DeviceTreeError::MissingCellParameter);
    }
    ok = false;
    let mut mmio_regions = Vec::new();
    if let Some(soc) = tree.root().find_child("soc") {
        if let Some(mmio) = soc.find_child_type("virtio_mmio") {
            for i in mmio.iter() {
                if let Some(PropertyValue::Address(a,b)) = i.of_value("reg") {
                    
                    mmio_regions.push((*a as usize, *b as usize));
                    // info!("{:#x} {:#x} ",*a as usize, *b as usize);
                    ok = true;
                } 
            }
        } 
    } 
    if !ok {
        return Err(DeviceTreeError::MissingCellParameter);
    }
    Ok(DtbInfo{memory_addr,memory_size,mmio_regions})
}