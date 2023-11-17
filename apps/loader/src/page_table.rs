
#[derive(Copy, Clone)]
pub struct PageTable {
    pub APP_PT_SV39: [u64; 512],
}

impl PageTable {
    /// Create a new page table
    pub const fn new(id: u64) -> Self {
        let mut newPageTable = PageTable {
            APP_PT_SV39: [0; 512],
        };
        // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
        newPageTable.APP_PT_SV39[2] = (0x80000 << 10) | 0xef;
        // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
        newPageTable.APP_PT_SV39[0x102] = (0x80000 << 10) | 0xef;
    
        // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
        newPageTable.APP_PT_SV39[0] = (0x00000 << 10) | 0xef;
    
        // For App aspace!
        // 0x4000_0000..0x8000_0000, VRWX_GAD, 1G block
        newPageTable.APP_PT_SV39[1] = ((0x80000) << 10) | 0xef;
        newPageTable
    }
    
    
}
