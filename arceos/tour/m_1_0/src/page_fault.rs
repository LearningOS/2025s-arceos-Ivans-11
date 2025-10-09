use axhal::trap::{register_trap_handler, PAGE_FAULT};
use axhal::paging::MappingFlags;
use axhal::mem::VirtAddr;
use axtask::TaskExtRef;

#[register_trap_handler(PAGE_FAULT)]
fn handle_page_fault(vaddr: VirtAddr, flags: MappingFlags, is_user: bool) -> bool {
    ax_println!("handle_page_fault...");
    if is_user {
        if axtask::current().task_ext().aspace.lock().handle_page_fault(vaddr, flags) {
            ax_println!("handle_page_fault: OK");
            true
        } else {
            ax_println!("handle_page_fault: FAIL");
            false
        }
    } else {
        false
    }
}