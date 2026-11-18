//! Memory-mapped shared memory and lock-free SPSC ring buffers (Windows).
use std::{
    ffi::c_void,
    path::Path,
    ptr,
    sync::atomic::{AtomicU64, Ordering},
};

pub const MAGIC: u64 = 0x5054_4D49_4E44_3031; // "PTMIND01"
pub const VERSION: u64 = 1;

pub const S2E_OFFSET: usize = 4096;
pub const S2E_CAP: usize = 16 * 1024 * 1024;
pub const E2S_OFFSET: usize = S2E_OFFSET + S2E_CAP;
pub const E2S_CAP: usize = 4 * 1024 * 1024;
pub const TOTAL_SIZE: usize = E2S_OFFSET + E2S_CAP;

const OFF_MAGIC: usize = 0;
const OFF_VERSION: usize = 8;
const OFF_SERVER_EPOCH: usize = 16;
const OFF_ENGINE_EPOCH: usize = 24;
const OFF_ENGINE_HEARTBEAT_MS: usize = 32;
#[allow(dead_code)]
const OFF_SERVER_TICK_SEQ: usize = 40;
const OFF_ENGINE_TICK_SEQ: usize = 48;

const OFF_S2E_HEAD: usize = 64;
const OFF_S2E_TAIL: usize = 72;
const OFF_S2E_CAP: usize = 80;
const OFF_S2E_OFFSET: usize = 88;

const OFF_E2S_HEAD: usize = 128;
const OFF_E2S_TAIL: usize = 136;
const OFF_E2S_CAP: usize = 144;
const OFF_E2S_OFFSET: usize = 152;

pub struct Shm {
    base: *mut u8,
    size: usize,
    #[cfg(windows)]
    file_handle: isize,
    #[cfg(windows)]
    map_handle: isize,
}

unsafe impl Send for Shm {}

impl Shm {
    pub fn open(path: &Path) -> Result<Self, String> {
        #[cfg(windows)]
        {
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent).map_err(|e| format!("create shm dir: {e}"))?;
            }
            use std::os::windows::ffi::OsStrExt;
            let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
            wide.push(0);

            const GENERIC_READ: u32 = 0x8000_0000;
            const GENERIC_WRITE: u32 = 0x4000_0000;
            const FILE_SHARE_ALL: u32 = 0x0000_0007;
            const OPEN_ALWAYS: u32 = 4;
            const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
            const INVALID_HANDLE: isize = -1;
            const PAGE_READWRITE: u32 = 0x04;
            const FILE_MAP_ALL_ACCESS: u32 = 0x000F_001F;
            const FILE_BEGIN: u32 = 0;

            let file_handle = unsafe {
                CreateFileW(
                    wide.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_ALL,
                    ptr::null_mut(),
                    OPEN_ALWAYS,
                    FILE_ATTRIBUTE_NORMAL,
                    0,
                )
            };
            if file_handle == INVALID_HANDLE {
                return Err(format!("CreateFileW failed: {}", std::io::Error::last_os_error()));
            }

            let mut existing_size: i64 = 0;
            if unsafe { GetFileSizeEx(file_handle, &mut existing_size) } == 0 {
                unsafe { CloseHandle(file_handle) };
                return Err(format!("GetFileSizeEx failed: {}", std::io::Error::last_os_error()));
            }
            if existing_size < TOTAL_SIZE as i64 {
                let mut new_pos: i64 = 0;
                if unsafe { SetFilePointerEx(file_handle, TOTAL_SIZE as i64, &mut new_pos, FILE_BEGIN) } == 0
                    || unsafe { SetEndOfFile(file_handle) } == 0
                {
                    unsafe { CloseHandle(file_handle) };
                    return Err(format!("extend shm file failed: {}", std::io::Error::last_os_error()));
                }
            }

            let map_handle = unsafe {
                CreateFileMappingW(
                    file_handle,
                    ptr::null_mut(),
                    PAGE_READWRITE,
                    (TOTAL_SIZE >> 32) as u32,
                    TOTAL_SIZE as u32,
                    ptr::null(),
                )
            };
            if map_handle == 0 {
                unsafe { CloseHandle(file_handle) };
                return Err(format!("CreateFileMappingW failed: {}", std::io::Error::last_os_error()));
            }

            let base = unsafe { MapViewOfFile(map_handle, FILE_MAP_ALL_ACCESS, 0, 0, TOTAL_SIZE) };
            if base.is_null() {
                unsafe {
                    CloseHandle(map_handle);
                    CloseHandle(file_handle);
                }
                return Err(format!("MapViewOfFile failed: {}", std::io::Error::last_os_error()));
            }

            Ok(Self { base, size: TOTAL_SIZE, file_handle, map_handle })
        }
        #[cfg(not(windows))]
        {
            let _ = path;
            Err("Windows-only shm backend".into())
        }
    }

    #[inline]
    fn atomic(&self, offset: usize) -> &AtomicU64 {
        debug_assert!(offset + 8 <= self.size && offset.is_multiple_of(8));
        unsafe { &*(self.base.add(offset) as *const AtomicU64) }
    }

    pub fn init_engine(&self) -> (u64, u64) {
        if self.atomic(OFF_MAGIC).load(Ordering::Acquire) != MAGIC
            || self.atomic(OFF_VERSION).load(Ordering::Acquire) != VERSION
            || self.atomic(OFF_S2E_CAP).load(Ordering::Acquire) != S2E_CAP as u64
            || self.atomic(OFF_E2S_CAP).load(Ordering::Acquire) != E2S_CAP as u64
        {
            unsafe { ptr::write_bytes(self.base, 0, S2E_OFFSET) };
            self.atomic(OFF_VERSION).store(VERSION, Ordering::Release);
            self.atomic(OFF_S2E_CAP).store(S2E_CAP as u64, Ordering::Release);
            self.atomic(OFF_S2E_OFFSET).store(S2E_OFFSET as u64, Ordering::Release);
            self.atomic(OFF_E2S_CAP).store(E2S_CAP as u64, Ordering::Release);
            self.atomic(OFF_E2S_OFFSET).store(E2S_OFFSET as u64, Ordering::Release);
            self.atomic(OFF_ENGINE_EPOCH).store(1, Ordering::Release);
            self.atomic(OFF_MAGIC).store(MAGIC, Ordering::Release);
            (0, 1)
        } else {
            // Discard pre-restart s2e backlog cleanly and signal reconnect via engine_epoch.
            let head = self.atomic(OFF_S2E_HEAD).load(Ordering::Acquire);
            self.atomic(OFF_S2E_TAIL).store(head, Ordering::Release);
            let next_epoch = self.atomic(OFF_ENGINE_EPOCH).load(Ordering::Acquire).wrapping_add(1).max(1);
            self.atomic(OFF_ENGINE_EPOCH).store(next_epoch, Ordering::Release);
            (self.server_epoch(), next_epoch)
        }
    }

    #[inline]
    pub fn server_epoch(&self) -> u64 {
        self.atomic(OFF_SERVER_EPOCH).load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_engine_heartbeat_ms(&self, ms: u64) {
        self.atomic(OFF_ENGINE_HEARTBEAT_MS).store(ms, Ordering::Release);
    }

    #[inline]
    pub fn set_engine_tick_seq(&self, seq: u64) {
        self.atomic(OFF_ENGINE_TICK_SEQ).store(seq, Ordering::Release);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn server_tick_seq(&self) -> u64 {
        self.atomic(OFF_SERVER_TICK_SEQ).load(Ordering::Acquire)
    }

    pub fn pop_s2e(&self, scratch: &mut Vec<u8>) -> Option<u16> {
        let head = self.atomic(OFF_S2E_HEAD).load(Ordering::Acquire);
        let tail = self.atomic(OFF_S2E_TAIL).load(Ordering::Relaxed);
        let used = head.wrapping_sub(tail);
        if used < 8 {
            return None;
        }
        let ring = unsafe { std::slice::from_raw_parts(self.base.add(S2E_OFFSET), S2E_CAP) };
        let hdr = read_u64_aligned(ring, (tail as usize) & (S2E_CAP - 1));
        let len = (hdr & 0xFFFF_FFFF) as usize;
        let msg_type = ((hdr >> 32) & 0xFFFF) as u16;
        let rec = (8 + len + 7) & !7;
        if used < rec as u64 {
            return None;
        }
        scratch.clear();
        scratch.resize(len, 0);
        let payload_off = ((tail as usize) + 8) & (S2E_CAP - 1);
        copy_from_ring(ring, S2E_CAP, payload_off, scratch);
        self.atomic(OFF_S2E_TAIL).store(tail.wrapping_add(rec as u64), Ordering::Release);
        Some(msg_type)
    }

    pub fn push_e2s(&self, msg_type: u16, payload: &[u8]) -> bool {
        let head = self.atomic(OFF_E2S_HEAD).load(Ordering::Relaxed);
        let tail = self.atomic(OFF_E2S_TAIL).load(Ordering::Acquire);
        let rec = (8 + payload.len() + 7) & !7;
        if rec > E2S_CAP || head.wrapping_sub(tail) + rec as u64 > E2S_CAP as u64 {
            return false;
        }
        let ring = unsafe { std::slice::from_raw_parts_mut(self.base.add(E2S_OFFSET), E2S_CAP) };
        let off = (head as usize) & (E2S_CAP - 1);
        let hdr = (payload.len() as u64) | ((msg_type as u64) << 32);
        write_u64_aligned(ring, off, hdr);
        let payload_off = (off + 8) & (E2S_CAP - 1);
        copy_to_ring(ring, E2S_CAP, payload_off, payload);
        self.atomic(OFF_E2S_HEAD).store(head.wrapping_add(rec as u64), Ordering::Release);
        true
    }
}

impl Drop for Shm {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            if !self.base.is_null() {
                UnmapViewOfFile(self.base);
            }
            if self.map_handle != 0 {
                CloseHandle(self.map_handle);
            }
            if self.file_handle != -1 {
                CloseHandle(self.file_handle);
            }
        }
    }
}

#[inline]
fn read_u64_aligned(ring: &[u8], off: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&ring[off..off + 8]);
    u64::from_le_bytes(b)
}

#[inline]
fn write_u64_aligned(ring: &mut [u8], off: usize, v: u64) {
    ring[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

fn copy_from_ring(ring: &[u8], cap: usize, off: usize, out: &mut [u8]) {
    let first = out.len().min(cap - off);
    out[..first].copy_from_slice(&ring[off..off + first]);
    if first < out.len() {
        let rest = out.len() - first;
        out[first..].copy_from_slice(&ring[..rest]);
    }
}

fn copy_to_ring(ring: &mut [u8], cap: usize, off: usize, src: &[u8]) {
    let first = src.len().min(cap - off);
    ring[off..off + first].copy_from_slice(&src[..first]);
    if first < src.len() {
        let rest = src.len() - first;
        ring[..rest].copy_from_slice(&src[first..]);
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: isize,
    ) -> isize;
    fn GetFileSizeEx(hFile: isize, lpFileSize: *mut i64) -> i32;
    fn SetFilePointerEx(
        hFile: isize,
        liDistanceToMove: i64,
        lpNewFilePointer: *mut i64,
        dwMoveMethod: u32,
    ) -> i32;
    fn SetEndOfFile(hFile: isize) -> i32;
    fn CreateFileMappingW(
        hFile: isize,
        lpFileMappingAttributes: *mut c_void,
        flProtect: u32,
        dwMaximumSizeHigh: u32,
        dwMaximumSizeLow: u32,
        lpName: *const u16,
    ) -> isize;
    fn MapViewOfFile(
        hFileMappingObject: isize,
        dwDesiredAccess: u32,
        dwFileOffsetHigh: u32,
        dwFileOffsetLow: u32,
        dwNumberOfBytesToMap: usize,
    ) -> *mut u8;
    fn UnmapViewOfFile(lpBaseAddress: *const u8) -> i32;
    fn CloseHandle(hObject: isize) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_copy_wraps_cleanly() {
        const CAP: usize = 64;
        let mut ring = [0u8; CAP];
        let payload = b"hello_wrap_around_test_bytes_12345";
        let off = 52;
        copy_to_ring(&mut ring, CAP, off, payload);
        let mut out = vec![0u8; payload.len()];
        copy_from_ring(&ring, CAP, off, &mut out);
        assert_eq!(out.as_slice(), payload);
    }

    #[test]
    fn shm_open_initializes_and_bumps_epoch() {
        let dir = std::env::temp_dir();
        let p = dir.join(format!("pt_mind_test_{}.shm", std::process::id()));
        let _ = std::fs::remove_file(&p);
        {
            let shm = Shm::open(&p).unwrap();
            let (se, ee) = shm.init_engine();
            assert_eq!(se, 0);
            assert_eq!(ee, 1);
            assert!(shm.push_e2s(101, &[1, 2, 3, 4, 5]));
        }
        {
            let shm = Shm::open(&p).unwrap();
            let (_, ee2) = shm.init_engine();
            assert_eq!(ee2, 2);
        }
        let _ = std::fs::remove_file(&p);
    }
}
