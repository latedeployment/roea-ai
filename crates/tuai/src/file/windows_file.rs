//! Windows file monitor using NtQuerySystemInformation handle enumeration.
//!
//! Enumerates open file handles system-wide via `NtQuerySystemInformation`
//! (SystemExtendedHandleInformation, class 64), duplicates each handle into
//! our process, checks whether it is a disk file with `GetFileType`, then
//! resolves the path with `GetFinalPathNameByHandleW`.
//!
//! ## Privileges
//! `DuplicateHandle` from another process requires `PROCESS_DUP_HANDLE`
//! access on that process. This succeeds for processes owned by the same
//! user without elevation. SYSTEM processes (PID 4) and processes owned by
//! other users are silently skipped.
//!
//! ## Performance
//! `NtQuerySystemInformation` itself is fast (kernel memcpy). The expensive
//! part is `DuplicateHandle` + `GetFinalPathNameByHandle` per file handle.
//! We pre-filter by PID before duplicating, so only handles belonging to
//! tracked AI agent processes are resolved — typically 50–200 handles total.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use tuai_common::{FileOpInfo, FileOperation, PlatformError, PlatformResult};

use super::FileMonitorBackend;

// ── Windows API constants ────────────────────────────────────────────────────

const PROCESS_DUP_HANDLE: u32 = 0x0040;
const DUPLICATE_SAME_ACCESS: u32 = 0x0002;
const FILE_TYPE_DISK: u32 = 0x0001;
const VOLUME_NAME_DOS: u32 = 0x0;
const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC0000004_u32 as i32;
/// NtQuerySystemInformation information class for extended handle info.
const SYSTEM_EXTENDED_HANDLE_INFORMATION: u32 = 64;

// File access bits used to infer Read vs Write
const FILE_WRITE_DATA: u32 = 0x0002;
const FILE_APPEND_DATA: u32 = 0x0004;
const GENERIC_WRITE: u32 = 0x40000000;

// ── Windows API declarations ─────────────────────────────────────────────────
//
// We declare the symbols directly rather than pulling in windows-sys so that
// this file adds zero new Cargo dependencies.

type HANDLE = isize;
const INVALID_HANDLE_VALUE: HANDLE = -1;

extern "system" {
    fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> HANDLE;
    fn GetCurrentProcess() -> HANDLE;
    fn GetCurrentProcessId() -> u32;
    fn DuplicateHandle(
        source_process: HANDLE,
        source_handle: HANDLE,
        target_process: HANDLE,
        target_handle: *mut HANDLE,
        desired_access: u32,
        inherit_handle: i32,
        options: u32,
    ) -> i32;
    fn CloseHandle(handle: HANDLE) -> i32;
    fn GetFileType(file: HANDLE) -> u32;
    fn GetFinalPathNameByHandleW(
        file: HANDLE,
        file_path: *mut u16,
        cch_file_path: u32,
        flags: u32,
    ) -> u32;
    fn NtQuerySystemInformation(
        system_information_class: u32,
        system_information: *mut c_void,
        system_information_length: u32,
        return_length: *mut u32,
    ) -> i32;
}

// ── NT structures ────────────────────────────────────────────────────────────
//
// SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX — layout is stable on x64 Windows.
// Each field is pointer-sized (8 bytes on x64) unless otherwise noted.

#[repr(C)]
struct HandleEntryEx {
    object: usize,              // kernel object pointer (opaque)
    unique_process_id: usize,   // owning PID
    handle_value: usize,        // handle value in the owning process
    granted_access: u32,        // access mask
    creator_back_trace_index: u16,
    object_type_index: u16,     // type index (File = varies by OS version)
    handle_attributes: u32,
    _reserved: u32,
}

// SYSTEM_HANDLE_INFORMATION_EX header — followed immediately by HandleEntryEx[].
#[repr(C)]
struct HandleInfoExHeader {
    number_of_handles: usize,
    _reserved: usize,
}

// ── Public monitor struct ────────────────────────────────────────────────────

/// File monitor that enumerates open handles system-wide.
pub struct WindowsFileMonitor {
    running: AtomicBool,
}

impl WindowsFileMonitor {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
        }
    }

    /// Collect open file paths, optionally limited to a single PID.
    fn collect(filter_pid: Option<u32>) -> PlatformResult<Vec<FileOpInfo>> {
        let buf = query_system_handles()?;
        let our_pid = unsafe { GetCurrentProcessId() };
        let mut result = Vec::new();

        for entry in iter_handle_entries(&buf) {
            let pid = entry.unique_process_id as u32;

            // Skip our own process, idle (0), and kernel (4)
            if pid == our_pid || pid == 0 || pid == 4 {
                continue;
            }
            if let Some(fp) = filter_pid {
                if pid != fp {
                    continue;
                }
            }

            if let Some(path) = resolve_file_path(entry) {
                let operation = access_to_operation(entry.granted_access);
                result.push(FileOpInfo::new(pid, operation, path));
            }
        }

        Ok(result)
    }
}

impl Default for WindowsFileMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FileMonitorBackend for WindowsFileMonitor {
    fn start(&mut self) -> PlatformResult<()> {
        self.running.store(true, Ordering::Relaxed);
        tracing::info!("Windows handle-based file monitor started");
        Ok(())
    }

    fn stop(&mut self) -> PlatformResult<()> {
        self.running.store(false, Ordering::Relaxed);
        tracing::info!("Windows handle-based file monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn snapshot(&self) -> PlatformResult<Vec<FileOpInfo>> {
        Self::collect(None)
    }

    fn open_files_for_pid(&self, pid: u32) -> PlatformResult<Vec<FileOpInfo>> {
        Self::collect(Some(pid))
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Call `NtQuerySystemInformation` with a growing buffer until it fits.
fn query_system_handles() -> PlatformResult<Vec<u8>> {
    let mut buf: Vec<u8> = vec![0u8; 1 << 20]; // start at 1 MiB
    loop {
        let mut return_len: u32 = 0;
        let status = unsafe {
            NtQuerySystemInformation(
                SYSTEM_EXTENDED_HANDLE_INFORMATION,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u32,
                &mut return_len,
            )
        };

        if status == STATUS_INFO_LENGTH_MISMATCH {
            let new_len = (return_len as usize).max(buf.len() * 2);
            buf.resize(new_len, 0);
            continue;
        }

        if status < 0 {
            return Err(PlatformError::CollectionFailed(format!(
                "NtQuerySystemInformation failed: {:#010x}",
                status as u32,
            )));
        }

        return Ok(buf);
    }
}

/// Interpret the raw buffer as an iterator of `HandleEntryEx` references.
fn iter_handle_entries(buf: &[u8]) -> &[HandleEntryEx] {
    let header_size = std::mem::size_of::<HandleInfoExHeader>();
    if buf.len() < header_size {
        return &[];
    }

    let header = unsafe { &*(buf.as_ptr() as *const HandleInfoExHeader) };
    let count = header.number_of_handles;
    let entry_size = std::mem::size_of::<HandleEntryEx>();
    let available = (buf.len() - header_size) / entry_size;
    let count = count.min(available);

    if count == 0 {
        return &[];
    }

    unsafe {
        std::slice::from_raw_parts(
            buf.as_ptr().add(header_size) as *const HandleEntryEx,
            count,
        )
    }
}

/// Duplicate the handle from the owning process into ours, check it is a disk
/// file, resolve its path, then close the duplicate. Returns `None` if the
/// handle is not a regular file or cannot be accessed.
fn resolve_file_path(entry: &HandleEntryEx) -> Option<String> {
    let pid = entry.unique_process_id as u32;
    let handle_value = entry.handle_value as HANDLE;

    // Open the owning process so we can duplicate from it
    let proc = unsafe { OpenProcess(PROCESS_DUP_HANDLE, 0, pid) };
    if proc == 0 || proc == INVALID_HANDLE_VALUE {
        return None;
    }

    let mut dup: HANDLE = 0;
    let ok = unsafe {
        DuplicateHandle(proc, handle_value, GetCurrentProcess(), &mut dup, 0, 0, DUPLICATE_SAME_ACCESS)
    };
    unsafe { CloseHandle(proc) };

    if ok == 0 {
        return None;
    }

    // Confirm it's a regular disk file (not a pipe, socket, etc.)
    let file_type = unsafe { GetFileType(dup) };
    if file_type != FILE_TYPE_DISK {
        unsafe { CloseHandle(dup) };
        return None;
    }

    let path = get_final_path(dup);
    unsafe { CloseHandle(dup) };
    path
}

/// Call `GetFinalPathNameByHandleW` with a growing buffer and return a
/// `String`, stripping the `\\?\` device-namespace prefix Windows prepends.
fn get_final_path(handle: HANDLE) -> Option<String> {
    let mut buf = vec![0u16; 512];
    loop {
        let len = unsafe {
            GetFinalPathNameByHandleW(handle, buf.as_mut_ptr(), buf.len() as u32, VOLUME_NAME_DOS)
        };

        if len == 0 {
            return None;
        }

        if (len as usize) < buf.len() {
            let raw = String::from_utf16_lossy(&buf[..len as usize]);
            return Some(strip_unc_prefix(&raw).to_string());
        }

        // Buffer was too small; retry with the exact required size
        buf.resize(len as usize + 1, 0);
    }
}

/// Map a Windows access mask to a `FileOperation`.
/// Prefers `Write` if any write bit is set.
pub fn access_to_operation(granted_access: u32) -> FileOperation {
    if granted_access & (FILE_WRITE_DATA | FILE_APPEND_DATA | GENERIC_WRITE) != 0 {
        FileOperation::Write
    } else {
        FileOperation::Read
    }
}

/// Strip the `\\?\` device-namespace prefix that `GetFinalPathNameByHandleW`
/// prepends to paths. Returns the original string if the prefix is absent.
pub fn strip_unc_prefix(path: &str) -> &str {
    path.strip_prefix("\\\\?\\").unwrap_or(path)
}

// ── Tests ────────────────────────────────────────────────────────────────────
//
// The pure helper functions can be tested on any platform. The integration
// path (actually calling Windows APIs) is exercised on a Windows runner via
// GitHub Actions or QEMU.

#[cfg(test)]
mod tests {
    use super::*;

    // ── strip_unc_prefix ──────────────────────────────────────────────────

    #[test]
    fn test_strip_unc_prefix_present() {
        assert_eq!(
            strip_unc_prefix("\\\\?\\C:\\Users\\user\\file.txt"),
            "C:\\Users\\user\\file.txt"
        );
    }

    #[test]
    fn test_strip_unc_prefix_absent() {
        assert_eq!(strip_unc_prefix("C:\\foo\\bar.txt"), "C:\\foo\\bar.txt");
    }

    #[test]
    fn test_strip_unc_prefix_unc_server() {
        // \\?\UNC\server\share stays as UNC\server\share after stripping
        assert_eq!(
            strip_unc_prefix("\\\\?\\UNC\\server\\share"),
            "UNC\\server\\share"
        );
    }

    #[test]
    fn test_strip_unc_prefix_empty() {
        assert_eq!(strip_unc_prefix(""), "");
    }

    // ── access_to_operation ───────────────────────────────────────────────

    #[test]
    fn test_access_read_only() {
        // FILE_READ_DATA = 0x0001, GENERIC_READ = 0x80000000
        assert_eq!(access_to_operation(0x0001), FileOperation::Read);
        assert_eq!(access_to_operation(0x80000000), FileOperation::Read);
    }

    #[test]
    fn test_access_write_data() {
        assert_eq!(access_to_operation(FILE_WRITE_DATA), FileOperation::Write);
    }

    #[test]
    fn test_access_append() {
        assert_eq!(access_to_operation(FILE_APPEND_DATA), FileOperation::Write);
    }

    #[test]
    fn test_access_generic_write() {
        assert_eq!(access_to_operation(GENERIC_WRITE), FileOperation::Write);
    }

    #[test]
    fn test_access_read_write_prefers_write() {
        // read + write → Write
        assert_eq!(access_to_operation(0x0001 | FILE_WRITE_DATA), FileOperation::Write);
    }

    #[test]
    fn test_access_zero_is_read() {
        // 0 access mask: no write bits → Read
        assert_eq!(access_to_operation(0), FileOperation::Read);
    }
}
