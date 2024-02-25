use super::config::{CellConfig, HvSystemConfig};
// use super::error::HvResult;
use hypercraft::{GuestPhysAddr, HostPhysAddr, HyperResult};
use super::mm::{GuestPhysMemorySet, GuestMemoryRegion};
use page_table_entry::MappingFlags;

#[derive(Debug)]
pub struct Cell<'a> {
    /// Cell configuration.
    pub config: CellConfig<'a>,
    /// Guest physical memory set.
    pub gpm: GuestPhysMemorySet,
}

impl Cell<'_> {
    fn new_root() -> HyperResult<Self> {
        let sys_config = HvSystemConfig::get();
        let cell_config = sys_config.root_cell.config();
        let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
        let hv_phys_size = sys_config.hypervisor_memory.size as usize;

        let mut gpm = GuestPhysMemorySet::new()?;

        // Map hypervisor memory to the empty page.
        gpm.map_region(GuestMemoryRegion{
            gpa: hv_phys_start,
            hpa: hv_phys_start,
            size: hv_phys_size,
            flags: MappingFlags::READ | MappingFlags::NO_HUGEPAGES,
        }.into())?;
        
        // Map all physical memory regions.
        for region in cell_config.mem_regions() {
            let flags = region.flags;
            gpm.map_region(GuestMemoryRegion{
                gpa: region.virt_start as GuestPhysAddr,
                hpa: region.phys_start as HostPhysAddr,
                size: region.size as usize,
                flags: MappingFlags::from_bits(flags.bits() as _).unwrap(),
            }.into())?;
        }
        trace!("Guest phyiscal memory set: {:#x?}", gpm);

        Ok(Self {
            config: cell_config,
            gpm,
        })
    }
}

static ROOT_CELL: spin::Once<Cell> = spin::Once::new();

pub fn root_cell<'a>() -> &'a Cell<'a> {
    ROOT_CELL.get().expect("Uninitialized root cell!")
}

pub fn init() -> HyperResult {
    // crate::arch::vmm::check_hypervisor_feature()?;

    let root_cell = Cell::new_root()?;
    info!("Root cell init end.");
    debug!("{:#x?}", root_cell);

    ROOT_CELL.call_once(|| root_cell);
    Ok(())
}
