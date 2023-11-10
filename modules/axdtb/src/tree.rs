
use alloc::{collections::VecDeque, string::String, vec, vec::Vec};
use core::fmt::{Display, Formatter};

use crate::error::{DeviceTreeError, Result};
use crate::header::DeviceTreeHeader;
use crate::node::DeviceTreeNode;

use core::option::Option::Some;
use core::result::Result::Err;

/// The tree structure
/// Reads data from a slice of bytes and parses into [DeviceTree]
/// Indexed by nodes and properties' names or by path for the whole tree
pub struct DeviceTree {
    header: DeviceTreeHeader,
    root: DeviceTreeNode,
}

impl DeviceTree {
    /// Parses a slice of bytes and constructs [DeviceTree]
    /// The structure should live as long as the `data`
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let magic = &data[0..4];
        if magic != [0xd0, 0x0d, 0xfe, 0xed] {
            return Err(DeviceTreeError::InvalidMagicNumber);
        }

        let header = DeviceTreeHeader::from_bytes(data)?;

        let root = DeviceTreeNode::from_bytes(
            data,
            &header,
            header.off_dt_struct as usize,
            InheritedValues::new(),
        )?;

        Ok(Self { header, root })
    }

    #[cfg(not(feature = "std"))]
    /// Parses from address where a device tree blob is located at
    pub fn from_address(addr: usize) -> Result<Self> {
        let header_bytes = unsafe { core::slice::from_raw_parts(addr as *const u8, 40) };
        let magic = &header_bytes[0..4];
        if magic != [0xd0, 0x0d, 0xfe, 0xed] {
            return Err(DeviceTreeError::InvalidMagicNumber);
        }
        let header = DeviceTreeHeader::from_bytes(header_bytes)?;
        let data =
            unsafe { core::slice::from_raw_parts(addr as *const u8, header.total_size as usize) };
        Self::from_bytes(data)
    }

    
    /// Get a reference of the root node
    pub fn root(&self) -> &DeviceTreeNode {
        &self.root
    }

    /// Find the node by given node path
    pub fn find_node(&self, path: &str) -> Option<&DeviceTreeNode> {
        let mut slices = path.split('/');
        if let Some("") = slices.next() {
            let mut first = &self.root;
            for i in slices {
                if let Some(node) = first.find_child(i) {
                    first = node;
                } else {
                    return None;
                }
            }
            Some(first)
        } else {
            None
        }
    }

}

/// Iterator for all the tree nodes
pub struct DeviceTreeNodeIter<'a> {
    queue: VecDeque<&'a DeviceTreeNode>,
}

impl<'a> Iterator for DeviceTreeNodeIter<'a> {
    type Item = &'a DeviceTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.queue.pop_front();
        match res {
            Some(node) if node.has_children() => {
                for i in node.nodes() {
                    self.queue.push_back(i);
                }
            }
            _ => {}
        }
        res
    }
}

impl Display for DeviceTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.root)
    }
}

impl<'a> IntoIterator for &'a DeviceTree {
    type Item = &'a DeviceTreeNode;
    type IntoIter = DeviceTreeNodeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DeviceTreeNodeIter {
            queue: VecDeque::from([self.root()]),
        }
    }
}

#[derive(Clone)]
pub(crate) struct InheritedValues(Vec<(String, u64)>);

impl InheritedValues {
    pub const fn new() -> Self {
        InheritedValues(vec![])
    }

    pub fn find(&self, name: &str) -> Option<u64> {
        for i in &self.0 {
            if i.0.as_str() == name {
                return Some(i.1);
            }
        }
        None
    }

    pub fn insert(&mut self, name: String, value: u64) {
        self.0.push((name, value));
    }
}