//! Zero-dependency dynamic OpenCL 2.1 FFI loader (Windows `OpenCL.dll`), RAII buffers, and kernels.
use std::{
    ffi::{CStr, CString, c_char, c_int, c_uint, c_void},
    mem::size_of,
    ptr,
    sync::{Arc, Mutex},
    time::Instant,
};

type ClPlatformId = *mut c_void;
type ClDeviceId = *mut c_void;
type ClContext = *mut c_void;
type ClCommandQueue = *mut c_void;
type ClProgram = *mut c_void;
type ClKernel = *mut c_void;
type ClMem = *mut c_void;

const CL_SUCCESS: c_int = 0;
const CL_FALSE: c_uint = 0;
const CL_TRUE: c_uint = 1;
const CL_DEVICE_TYPE_GPU: u64 = 1 << 2;
const CL_PLATFORM_VENDOR: c_uint = 0x0903;
const CL_PLATFORM_NAME: c_uint = 0x0902;
const CL_DEVICE_MAX_COMPUTE_UNITS: c_uint = 0x1002;
const CL_DEVICE_MAX_WORK_GROUP_SIZE: c_uint = 0x1004;
const CL_DEVICE_MAX_CLOCK_FREQUENCY: c_uint = 0x100C;
const CL_DEVICE_GLOBAL_MEM_SIZE: c_uint = 0x1010;
const CL_DEVICE_NAME: c_uint = 0x102B;
const CL_DEVICE_EXTENSIONS: c_uint = 0x1030;
const CL_PROGRAM_BUILD_LOG: c_uint = 0x1183;

const CL_MEM_READ_WRITE: u64 = 1 << 0;
const CL_MEM_READ_ONLY: u64 = 1 << 2;
const CL_MEM_COPY_HOST_PTR: u64 = 1 << 5;

#[cfg(windows)]
unsafe extern "system" {
    fn LoadLibraryA(lp_lib_file_name: *const c_char) -> *mut c_void;
    fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const c_char) -> *mut c_void;
}

type FnGetPlatformIDs = unsafe extern "system" fn(c_uint, *mut ClPlatformId, *mut c_uint) -> c_int;
type FnGetPlatformInfo = unsafe extern "system" fn(ClPlatformId, c_uint, usize, *mut c_void, *mut usize) -> c_int;
type FnGetDeviceIDs = unsafe extern "system" fn(ClPlatformId, u64, c_uint, *mut ClDeviceId, *mut c_uint) -> c_int;
type FnGetDeviceInfo = unsafe extern "system" fn(ClDeviceId, c_uint, usize, *mut c_void, *mut usize) -> c_int;
type FnCreateContext = unsafe extern "system" fn(*const isize, c_uint, *const ClDeviceId, *const c_void, *mut c_void, *mut c_int) -> ClContext;
type FnReleaseContext = unsafe extern "system" fn(ClContext) -> c_int;
type FnCreateCommandQueue = unsafe extern "system" fn(ClContext, ClDeviceId, u64, *mut c_int) -> ClCommandQueue;
type FnReleaseCommandQueue = unsafe extern "system" fn(ClCommandQueue) -> c_int;
type FnFinish = unsafe extern "system" fn(ClCommandQueue) -> c_int;
type FnCreateProgramWithSource = unsafe extern "system" fn(ClContext, c_uint, *const *const c_char, *const usize, *mut c_int) -> ClProgram;
type FnBuildProgram = unsafe extern "system" fn(ClProgram, c_uint, *const ClDeviceId, *const c_char, *const c_void, *mut c_void) -> c_int;
type FnGetProgramBuildInfo = unsafe extern "system" fn(ClProgram, ClDeviceId, c_uint, usize, *mut c_void, *mut usize) -> c_int;
type FnReleaseProgram = unsafe extern "system" fn(ClProgram) -> c_int;
type FnCreateKernel = unsafe extern "system" fn(ClProgram, *const c_char, *mut c_int) -> ClKernel;
type FnReleaseKernel = unsafe extern "system" fn(ClKernel) -> c_int;
type FnSetKernelArg = unsafe extern "system" fn(ClKernel, c_uint, usize, *const c_void) -> c_int;
type FnEnqueueNDRangeKernel = unsafe extern "system" fn(
    ClCommandQueue,
    ClKernel,
    c_uint,
    *const usize,
    *const usize,
    *const usize,
    c_uint,
    *const *mut c_void,
    *mut *mut c_void,
) -> c_int;
type FnCreateBuffer = unsafe extern "system" fn(ClContext, u64, usize, *mut c_void, *mut c_int) -> ClMem;
type FnReleaseMemObject = unsafe extern "system" fn(ClMem) -> c_int;
type FnEnqueueWriteBuffer = unsafe extern "system" fn(
    ClCommandQueue,
    ClMem,
    c_uint,
    usize,
    usize,
    *const c_void,
    c_uint,
    *const *mut c_void,
    *mut *mut c_void,
) -> c_int;
type FnEnqueueReadBuffer = unsafe extern "system" fn(
    ClCommandQueue,
    ClMem,
    c_uint,
    usize,
    usize,
    *mut c_void,
    c_uint,
    *const *mut c_void,
    *mut *mut c_void,
) -> c_int;

struct ClApi {
    _lib: *mut c_void,
    cl_get_platform_ids: FnGetPlatformIDs,
    cl_get_platform_info: FnGetPlatformInfo,
    cl_get_device_ids: FnGetDeviceIDs,
    cl_get_device_info: FnGetDeviceInfo,
    cl_create_context: FnCreateContext,
    cl_release_context: FnReleaseContext,
    cl_create_command_queue: FnCreateCommandQueue,
    cl_release_command_queue: FnReleaseCommandQueue,
    cl_finish: FnFinish,
    cl_create_program_with_source: FnCreateProgramWithSource,
    cl_build_program: FnBuildProgram,
    cl_get_program_build_info: FnGetProgramBuildInfo,
    cl_release_program: FnReleaseProgram,
    cl_create_kernel: FnCreateKernel,
    cl_release_kernel: FnReleaseKernel,
    cl_set_kernel_arg: FnSetKernelArg,
    cl_enqueue_nd_range_kernel: FnEnqueueNDRangeKernel,
    cl_create_buffer: FnCreateBuffer,
    cl_release_mem_object: FnReleaseMemObject,
    cl_enqueue_write_buffer: FnEnqueueWriteBuffer,
    cl_enqueue_read_buffer: FnEnqueueReadBuffer,
}

unsafe impl Send for ClApi {}
unsafe impl Sync for ClApi {}

#[cfg(windows)]
unsafe fn load_sym<T: Copy>(lib: *mut c_void, name: &[u8]) -> Result<T, String> {
    let p = unsafe { GetProcAddress(lib, name.as_ptr().cast()) };
    if p.is_null() {
        return Err(format!("OpenCL missing symbol {}", String::from_utf8_lossy(name)));
    }
    Ok(unsafe { std::mem::transmute_copy(&p) })
}

impl ClApi {
    fn load() -> Result<Self, String> {
        #[cfg(not(windows))]
        {
            Err("OpenCL loader only enabled on Windows".into())
        }
        #[cfg(windows)]
        unsafe {
            let lib = LoadLibraryA(c"OpenCL.dll".as_ptr());
            if lib.is_null() {
                return Err("OpenCL.dll not found".into());
            }
            Ok(Self {
                _lib: lib,
                cl_get_platform_ids: load_sym(lib, b"clGetPlatformIDs\0")?,
                cl_get_platform_info: load_sym(lib, b"clGetPlatformInfo\0")?,
                cl_get_device_ids: load_sym(lib, b"clGetDeviceIDs\0")?,
                cl_get_device_info: load_sym(lib, b"clGetDeviceInfo\0")?,
                cl_create_context: load_sym(lib, b"clCreateContext\0")?,
                cl_release_context: load_sym(lib, b"clReleaseContext\0")?,
                cl_create_command_queue: load_sym(lib, b"clCreateCommandQueue\0")?,
                cl_release_command_queue: load_sym(lib, b"clReleaseCommandQueue\0")?,
                cl_finish: load_sym(lib, b"clFinish\0")?,
                cl_create_program_with_source: load_sym(lib, b"clCreateProgramWithSource\0")?,
                cl_build_program: load_sym(lib, b"clBuildProgram\0")?,
                cl_get_program_build_info: load_sym(lib, b"clGetProgramBuildInfo\0")?,
                cl_release_program: load_sym(lib, b"clReleaseProgram\0")?,
                cl_create_kernel: load_sym(lib, b"clCreateKernel\0")?,
                cl_release_kernel: load_sym(lib, b"clReleaseKernel\0")?,
                cl_set_kernel_arg: load_sym(lib, b"clSetKernelArg\0")?,
                cl_enqueue_nd_range_kernel: load_sym(lib, b"clEnqueueNDRangeKernel\0")?,
                cl_create_buffer: load_sym(lib, b"clCreateBuffer\0")?,
                cl_release_mem_object: load_sym(lib, b"clReleaseMemObject\0")?,
                cl_enqueue_write_buffer: load_sym(lib, b"clEnqueueWriteBuffer\0")?,
                cl_enqueue_read_buffer: load_sym(lib, b"clEnqueueReadBuffer\0")?,
            })
        }
    }

    fn platform_str(&self, p: ClPlatformId, info: c_uint) -> String {
        let mut sz = 0usize;
        let e = unsafe { (self.cl_get_platform_info)(p, info, 0, ptr::null_mut(), &mut sz) };
        if e != CL_SUCCESS || sz == 0 {
            return String::new();
        }
        let mut buf = vec![0u8; sz];
        let e = unsafe { (self.cl_get_platform_info)(p, info, sz, buf.as_mut_ptr().cast(), &mut sz) };
        if e != CL_SUCCESS {
            return String::new();
        }
        CStr::from_bytes_until_nul(&buf)
            .map(|c| c.to_string_lossy().into_owned())
            .unwrap_or_else(|_| String::from_utf8_lossy(&buf).into_owned())
    }

    fn device_str(&self, d: ClDeviceId, info: c_uint) -> String {
        let mut sz = 0usize;
        let e = unsafe { (self.cl_get_device_info)(d, info, 0, ptr::null_mut(), &mut sz) };
        if e != CL_SUCCESS || sz == 0 {
            return String::new();
        }
        let mut buf = vec![0u8; sz];
        let e = unsafe { (self.cl_get_device_info)(d, info, sz, buf.as_mut_ptr().cast(), &mut sz) };
        if e != CL_SUCCESS {
            return String::new();
        }
        CStr::from_bytes_until_nul(&buf)
            .map(|c| c.to_string_lossy().into_owned())
            .unwrap_or_else(|_| String::from_utf8_lossy(&buf).into_owned())
    }

    fn device_u32(&self, d: ClDeviceId, info: c_uint) -> u32 {
        let mut val = 0u32;
        let _ = unsafe { (self.cl_get_device_info)(d, info, size_of::<u32>(), (&raw mut val).cast(), ptr::null_mut()) };
        val
    }

    fn device_u64(&self, d: ClDeviceId, info: c_uint) -> u64 {
        let mut val = 0u64;
        let _ = unsafe { (self.cl_get_device_info)(d, info, size_of::<u64>(), (&raw mut val).cast(), ptr::null_mut()) };
        val
    }
}

#[derive(Clone, Debug)]
pub struct GpuDeviceInfo {
    pub name: String,
    pub platform: String,
    pub vendor: String,
    pub compute_units: u32,
    pub clock_mhz: u32,
    pub global_mem_bytes: u64,
    pub max_work_group_size: usize,
    pub has_fp64: bool,
}

struct InnerDevice {
    api: ClApi,
    device: ClDeviceId,
    context: ClContext,
    queue: ClCommandQueue,
    info: GpuDeviceInfo,
    queue_lock: Mutex<()>,
}

unsafe impl Send for InnerDevice {}
unsafe impl Sync for InnerDevice {}

impl Drop for InnerDevice {
    fn drop(&mut self) {
        unsafe {
            let _ = (self.api.cl_finish)(self.queue);
            let _ = (self.api.cl_release_command_queue)(self.queue);
            let _ = (self.api.cl_release_context)(self.context);
        }
    }
}

#[derive(Clone)]
pub struct GpuDevice {
    inner: Arc<InnerDevice>,
}

impl GpuDevice {
    pub fn open_best() -> Result<Self, String> {
        let api = ClApi::load()?;
        let mut np = 0u32;
        let err = unsafe { (api.cl_get_platform_ids)(0, ptr::null_mut(), &mut np) };
        if err != CL_SUCCESS || np == 0 {
            return Err(format!("no OpenCL platforms (err={err}, count={np})"));
        }
        let mut platforms = vec![ptr::null_mut(); np as usize];
        let err = unsafe { (api.cl_get_platform_ids)(np, platforms.as_mut_ptr(), &mut np) };
        if err != CL_SUCCESS {
            return Err(format!("clGetPlatformIDs failed: {err}"));
        }

        let mut best: Option<(u64, ClDeviceId, GpuDeviceInfo)> = None;
        for &p in &platforms {
            let plat_name = api.platform_str(p, CL_PLATFORM_NAME);
            let vendor = api.platform_str(p, CL_PLATFORM_VENDOR);
            let mut nd = 0u32;
            let e = unsafe { (api.cl_get_device_ids)(p, CL_DEVICE_TYPE_GPU, 0, ptr::null_mut(), &mut nd) };
            if e != CL_SUCCESS || nd == 0 {
                continue;
            }
            let mut devs = vec![ptr::null_mut(); nd as usize];
            if unsafe { (api.cl_get_device_ids)(p, CL_DEVICE_TYPE_GPU, nd, devs.as_mut_ptr(), &mut nd) } != CL_SUCCESS {
                continue;
            }
            for &d in &devs {
                let cu = api.device_u32(d, CL_DEVICE_MAX_COMPUTE_UNITS);
                let mem = api.device_u64(d, CL_DEVICE_GLOBAL_MEM_SIZE);
                let clk = api.device_u32(d, CL_DEVICE_MAX_CLOCK_FREQUENCY);
                let mut wg = 0usize;
                let _ = unsafe { (api.cl_get_device_info)(d, CL_DEVICE_MAX_WORK_GROUP_SIZE, size_of::<usize>(), (&raw mut wg).cast(), ptr::null_mut()) };
                let name = api.device_str(d, CL_DEVICE_NAME);
                let exts = api.device_str(d, CL_DEVICE_EXTENSIONS);
                let has_fp64 = exts.split_whitespace().any(|x| x == "cl_khr_fp64");
                let score = u64::from(cu).saturating_mul(mem >> 20).saturating_mul(if has_fp64 { 2 } else { 1 });
                if best.as_ref().is_none_or(|(bs, _, _)| score > *bs) {
                    best = Some((
                        score,
                        d,
                        GpuDeviceInfo {
                            name,
                            platform: plat_name.clone(),
                            vendor: vendor.clone(),
                            compute_units: cu,
                            clock_mhz: clk,
                            global_mem_bytes: mem,
                            max_work_group_size: wg,
                            has_fp64,
                        },
                    ));
                }
            }
        }

        let (_, device, info) = best.ok_or_else(|| "no OpenCL GPU device found".to_owned())?;
        let mut cerr = 0i32;
        let context = unsafe { (api.cl_create_context)(ptr::null(), 1, &device, ptr::null(), ptr::null_mut(), &mut cerr) };
        if context.is_null() || cerr != CL_SUCCESS {
            return Err(format!("clCreateContext failed: {cerr}"));
        }
        let mut qerr = 0i32;
        let queue = unsafe { (api.cl_create_command_queue)(context, device, 0, &mut qerr) };
        if queue.is_null() || qerr != CL_SUCCESS {
            unsafe { (api.cl_release_context)(context) };
            return Err(format!("clCreateCommandQueue failed: {qerr}"));
        }
        Ok(Self {
            inner: Arc::new(InnerDevice {
                api,
                device,
                context,
                queue,
                info,
                queue_lock: Mutex::new(()),
            }),
        })
    }

    pub fn info(&self) -> &GpuDeviceInfo {
        &self.inner.info
    }

    pub fn finish(&self) -> Result<(), String> {
        let _g = self.inner.queue_lock.lock().unwrap();
        let err = unsafe { (self.inner.api.cl_finish)(self.inner.queue) };
        if err != CL_SUCCESS {
            Err(format!("clFinish failed: {err}"))
        } else {
            Ok(())
        }
    }

    pub fn build_program(&self, source: &str) -> Result<GpuProgram, String> {
        let csrc = CString::new(source).map_err(|_| "interior NUL in OpenCL source".to_owned())?;
        let src_ptr = csrc.as_ptr();
        let src_len = source.len();
        let mut err = 0i32;
        let prog = unsafe {
            (self.inner.api.cl_create_program_with_source)(
                self.inner.context,
                1,
                &src_ptr,
                &src_len,
                &mut err,
            )
        };
        if prog.is_null() || err != CL_SUCCESS {
            return Err(format!("clCreateProgramWithSource failed: {err}"));
        }
        let berr = unsafe {
            (self.inner.api.cl_build_program)(
                prog,
                1,
                &self.inner.device,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
            )
        };
        if berr != CL_SUCCESS {
            let mut log_len = 0usize;
            unsafe {
                let _ = (self.inner.api.cl_get_program_build_info)(
                    prog,
                    self.inner.device,
                    CL_PROGRAM_BUILD_LOG,
                    0,
                    ptr::null_mut(),
                    &mut log_len,
                );
            }
            let mut log_buf = vec![0u8; log_len.max(1)];
            unsafe {
                let _ = (self.inner.api.cl_get_program_build_info)(
                    prog,
                    self.inner.device,
                    CL_PROGRAM_BUILD_LOG,
                    log_len,
                    log_buf.as_mut_ptr().cast(),
                    &mut log_len,
                );
                let _ = (self.inner.api.cl_release_program)(prog);
            }
            let msg = String::from_utf8_lossy(&log_buf);
            return Err(format!("clBuildProgram failed ({berr}): {msg}"));
        }
        Ok(GpuProgram {
            dev: self.clone(),
            prog,
        })
    }

    pub fn alloc_rw(&self, bytes: usize) -> Result<GpuBuffer, String> {
        let bytes = bytes.max(4);
        let mut err = 0i32;
        let mem = unsafe {
            (self.inner.api.cl_create_buffer)(
                self.inner.context,
                CL_MEM_READ_WRITE,
                bytes,
                ptr::null_mut(),
                &mut err,
            )
        };
        if mem.is_null() || err != CL_SUCCESS {
            return Err(format!("clCreateBuffer rw({bytes}) failed: {err}"));
        }
        Ok(GpuBuffer { dev: self.clone(), mem, bytes })
    }

    pub fn alloc_ro_slice<T: Copy>(&self, data: &[T]) -> Result<GpuBuffer, String> {
        self.alloc_copy_flags(data, CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR)
    }

    pub fn alloc_rw_slice<T: Copy>(&self, data: &[T]) -> Result<GpuBuffer, String> {
        self.alloc_copy_flags(data, CL_MEM_READ_WRITE | CL_MEM_COPY_HOST_PTR)
    }

    fn alloc_copy_flags<T: Copy>(&self, data: &[T], flags: u64) -> Result<GpuBuffer, String> {
        let bytes = std::mem::size_of_val(data);
        if bytes == 0 {
            return self.alloc_rw(4);
        }
        let mut err = 0i32;
        let mem = unsafe {
            (self.inner.api.cl_create_buffer)(
                self.inner.context,
                flags,
                bytes,
                data.as_ptr().cast::<c_void>().cast_mut(),
                &mut err,
            )
        };
        if mem.is_null() || err != CL_SUCCESS {
            return Err(format!("clCreateBuffer copy({bytes}) failed: {err}"));
        }
        Ok(GpuBuffer { dev: self.clone(), mem, bytes })
    }

    pub fn measure_roundtrip_us(&self) -> Result<(f64, f64, f64), String> {
        let prog = self.build_program("__kernel void noop(__global uint* out) { if (get_global_id(0) == 0) out[0] = 1; }")?;
        let mut kern = prog.kernel("noop")?;
        let buf = self.alloc_rw(4)?;
        let mut host = [0u32; 1];
        for _ in 0..8 {
            kern.set_buf(0, &buf)?;
            kern.enqueue_1d(1, Some(1))?;
            buf.read_slice(&mut host)?;
        }
        let mut samples = Vec::with_capacity(64);
        for _ in 0..64 {
            let t0 = Instant::now();
            kern.set_buf(0, &buf)?;
            kern.enqueue_1d(1, Some(1))?;
            buf.read_slice(&mut host)?;
            samples.push(t0.elapsed().as_secs_f64() * 1e6);
        }
        samples.sort_by(f64::total_cmp);
        Ok((samples[0], samples[samples.len() / 2], samples[samples.len() - 1]))
    }
}

pub struct GpuBuffer {
    dev: GpuDevice,
    mem: ClMem,
    bytes: usize,
}

unsafe impl Send for GpuBuffer {}
unsafe impl Sync for GpuBuffer {}

impl Drop for GpuBuffer {
    fn drop(&mut self) {
        unsafe {
            let _ = (self.dev.inner.api.cl_release_mem_object)(self.mem);
        }
    }
}

impl GpuBuffer {
    pub fn write_slice<T: Copy>(&self, data: &[T]) -> Result<(), String> {
        let bytes = std::mem::size_of_val(data);
        if bytes > self.bytes {
            return Err(format!("write_slice overflow: {} > {}", bytes, self.bytes));
        }
        if bytes == 0 {
            return Ok(());
        }
        let _g = self.dev.inner.queue_lock.lock().unwrap();
        let err = unsafe {
            (self.dev.inner.api.cl_enqueue_write_buffer)(
                self.dev.inner.queue,
                self.mem,
                CL_TRUE,
                0,
                bytes,
                data.as_ptr().cast(),
                0,
                ptr::null(),
                ptr::null_mut(),
            )
        };
        if err != CL_SUCCESS {
            Err(format!("clEnqueueWriteBuffer failed: {err}"))
        } else {
            Ok(())
        }
    }

    pub fn write_slice_async<T: Copy>(&self, data: &[T]) -> Result<(), String> {
        let bytes = std::mem::size_of_val(data);
        if bytes == 0 {
            return Ok(());
        }
        let _g = self.dev.inner.queue_lock.lock().unwrap();
        let err = unsafe {
            (self.dev.inner.api.cl_enqueue_write_buffer)(
                self.dev.inner.queue,
                self.mem,
                CL_FALSE,
                0,
                bytes,
                data.as_ptr().cast(),
                0,
                ptr::null(),
                ptr::null_mut(),
            )
        };
        if err != CL_SUCCESS {
            Err(format!("clEnqueueWriteBuffer(async) failed: {err}"))
        } else {
            Ok(())
        }
    }

    pub fn read_slice<T: Copy>(&self, out: &mut [T]) -> Result<(), String> {
        let bytes = std::mem::size_of_val(out);
        if bytes > self.bytes {
            return Err(format!("read_slice overflow: {} > {}", bytes, self.bytes));
        }
        if bytes == 0 {
            return Ok(());
        }
        let _g = self.dev.inner.queue_lock.lock().unwrap();
        let err = unsafe {
            (self.dev.inner.api.cl_enqueue_read_buffer)(
                self.dev.inner.queue,
                self.mem,
                CL_TRUE,
                0,
                bytes,
                out.as_mut_ptr().cast(),
                0,
                ptr::null(),
                ptr::null_mut(),
            )
        };
        if err != CL_SUCCESS {
            Err(format!("clEnqueueReadBuffer failed: {err}"))
        } else {
            Ok(())
        }
    }
}

pub struct GpuProgram {
    dev: GpuDevice,
    prog: ClProgram,
}

unsafe impl Send for GpuProgram {}
unsafe impl Sync for GpuProgram {}

impl Drop for GpuProgram {
    fn drop(&mut self) {
        unsafe {
            let _ = (self.dev.inner.api.cl_release_program)(self.prog);
        }
    }
}

impl GpuProgram {
    pub fn kernel(&self, name: &str) -> Result<GpuKernel, String> {
        let cname = CString::new(name).map_err(|_| "NUL in kernel name".to_owned())?;
        let mut err = 0i32;
        let kernel = unsafe { (self.dev.inner.api.cl_create_kernel)(self.prog, cname.as_ptr(), &mut err) };
        if kernel.is_null() || err != CL_SUCCESS {
            return Err(format!("clCreateKernel({name}) failed: {err}"));
        }
        Ok(GpuKernel {
            dev: self.dev.clone(),
            kernel,
        })
    }
}

pub struct GpuKernel {
    dev: GpuDevice,
    kernel: ClKernel,
}

unsafe impl Send for GpuKernel {}
unsafe impl Sync for GpuKernel {}

impl Drop for GpuKernel {
    fn drop(&mut self) {
        unsafe {
            let _ = (self.dev.inner.api.cl_release_kernel)(self.kernel);
        }
    }
}

impl GpuKernel {
    pub fn set_buf(&mut self, idx: u32, buf: &GpuBuffer) -> Result<(), String> {
        let mem = buf.mem;
        let err = unsafe {
            (self.dev.inner.api.cl_set_kernel_arg)(
                self.kernel,
                idx,
                size_of::<ClMem>(),
                (&raw const mem).cast(),
            )
        };
        if err != CL_SUCCESS {
            Err(format!("clSetKernelArg(buf idx={idx}) failed: {err}"))
        } else {
            Ok(())
        }
    }

    pub fn set_val<T: Copy>(&mut self, idx: u32, val: &T) -> Result<(), String> {
        let err = unsafe {
            (self.dev.inner.api.cl_set_kernel_arg)(
                self.kernel,
                idx,
                size_of::<T>(),
                (val as *const T).cast(),
            )
        };
        if err != CL_SUCCESS {
            Err(format!("clSetKernelArg(val idx={idx}) failed: {err}"))
        } else {
            Ok(())
        }
    }

    pub fn enqueue_1d(&self, items: usize, local: Option<usize>) -> Result<(), String> {
        let items = items.max(1);
        let global = match local {
            Some(lw) if lw > 0 => items.div_ceil(lw) * lw,
            _ => items,
        };
        let local_ptr = match &local {
            Some(lw) if *lw > 0 => lw as *const usize,
            _ => ptr::null(),
        };
        let _g = self.dev.inner.queue_lock.lock().unwrap();
        let err = unsafe {
            (self.dev.inner.api.cl_enqueue_nd_range_kernel)(
                self.dev.inner.queue,
                self.kernel,
                1,
                ptr::null(),
                &global,
                local_ptr,
                0,
                ptr::null(),
                ptr::null_mut(),
            )
        };
        if err != CL_SUCCESS {
            Err(format!("clEnqueueNDRangeKernel(global={global}) failed: {err}"))
        } else {
            Ok(())
        }
    }
}
