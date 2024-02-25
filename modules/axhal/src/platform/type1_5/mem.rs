// TODO: get memory regions from multiboot info.

use crate::mem::*;

/// Number of physical memory regions.
pub(crate) fn memory_regions_num() -> usize {
    0
}

/// Returns the physical memory region at the given index, or [`None`] if the
/// index is out of bounds.
pub(crate) fn memory_region_at(idx: usize) -> Option<MemRegion> {
    None
}
