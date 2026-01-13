/*
 * uclcli lib.rs - safe rust bindings for libucl
 * Copyright (C) 2020-2021  BMW Group
 * Copyright (C) 2026  Andrew Rogoff <andrew@andrewbrogoff.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::mem;
use std::ptr;
use std::sync::OnceLock;

use libc::{c_int, c_long, c_short, c_uint, c_void};
use thiserror::Error;

/// UCL library version in format 0xVVRRMM (Version.Revision.Minor).
/// 0x01_0300 = version 1.03.00
const UCL_VERSION: u32 = 0x01_0300;

/// Maximum allowed destination capacity for decompression (1GB).
/// This prevents memory exhaustion attacks (DoS) from malicious input.
const MAX_DST_CAPACITY: u32 = 1_073_741_824;

/// Default compression level for NRV2B algorithm.
/// Level 6 provides a good balance between compression ratio and speed.
const DEFAULT_COMPRESSION_LEVEL: c_int = 6;

#[link(name = "ucl")]
unsafe extern "C" {
    #[must_use]
    fn __ucl_init2(
        version: u32,
        short: i32,
        int: i32,
        long: i32,
        ucl_uint32: i32,
        ucl_uint: i32,
        minus_one: i32,
        pchar: i32,
        ucl_voidp: i32,
        ucl_compress_t: i32,
    ) -> c_int;

    #[must_use]
    fn ucl_nrv2b_decompress_safe_8(
        src: *const u8,
        src_len: c_uint,
        dst: *mut u8,
        dst_len: *mut c_uint,
        wrkmem: *const c_void,
    ) -> c_int;

    #[must_use]
    fn ucl_nrv2b_99_compress(
        src: *const u8,
        src_len: c_uint,
        dst: *mut u8,
        dst_len: *mut c_uint,
        cb: *const c_void,
        level: c_int,
        conf: *const c_void,
        result: *const c_void,
    ) -> c_int;
}

static INIT_RESULT: OnceLock<i32> = OnceLock::new();

/// Initializes libucl.
///
/// Call this once before calling any other function in this package.
/// This function is thread-safe and will only initialize the library once,
/// even when called from multiple threads concurrently.
///
/// # Panics
/// If initialization failed for some reason, this function will panic.
pub fn ucl_init() {
    let result = INIT_RESULT.get_or_init(|| {
        // SAFETY: We're passing correct size parameters for the current
        // architecture to initialize the libucl library.
        unsafe {
            __ucl_init2(
                UCL_VERSION,
                mem::size_of::<c_short>() as i32,
                mem::size_of::<c_int>() as i32,
                mem::size_of::<c_long>() as i32,
                mem::size_of::<u32>() as i32,
                mem::size_of::<c_uint>() as i32,
                -1i32,
                mem::size_of::<*mut u8>() as i32,
                mem::size_of::<*mut c_void>() as i32,
                mem::size_of::<*mut c_void>() as i32, // function ptr
            )
        }
    });

    assert!(
        *result == 0,
        "ucl init failed. incompatible library version or architecture?"
    );
}

#[derive(Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum UclErrorKind {
    #[error("generic UCL error")]
    GenericError,
    #[error("invalid argument")]
    InvalidArgument,
    #[error("out of memory")]
    OutOfMemory,
    #[error("not compressible")]
    NotCompressible,
    #[error("input overrun")]
    InputOverrun,
    #[error("output overrun")]
    OutputOverrun,
    #[error("look-behind overrun")]
    LookbehindOverrun,
    #[error("EOF not found")]
    EofNotFound,
    #[error("input not consumed")]
    InputNotConsumed,
    #[error("overlap overrun")]
    OverlapOverrun,
    #[error("src buffer too large")]
    SrcTooLarge,
    #[error("dst buffer too large")]
    DstTooLarge,
    #[error("dst buffer too small")]
    DstTooSmall,
}

impl UclErrorKind {
    fn from(code: i32) -> Self {
        match code {
            -2 => UclErrorKind::InvalidArgument,
            -3 => UclErrorKind::OutOfMemory,
            -101 => UclErrorKind::NotCompressible,
            -201 => UclErrorKind::InputOverrun,
            -202 => UclErrorKind::OutputOverrun,
            -203 => UclErrorKind::LookbehindOverrun,
            -204 => UclErrorKind::EofNotFound,
            -205 => UclErrorKind::InputNotConsumed,
            -206 => UclErrorKind::OverlapOverrun,
            _ => UclErrorKind::GenericError,
        }
    }
}

/// Converts a usize length to u32, returning an appropriate error if conversion fails.
///
/// This helper function centralizes the checked conversion pattern used throughout
/// the library for buffer size validation.
#[inline]
fn to_u32_src_len(len: usize) -> Result<u32, UclErrorKind> {
    len.try_into().map_err(|_| UclErrorKind::SrcTooLarge)
}

#[inline]
fn to_u32_dst_len(len: usize) -> Result<u32, UclErrorKind> {
    len.try_into().map_err(|_| UclErrorKind::DstTooLarge)
}

/// # Safety
///
/// - `dst` must be a valid pointer to a buffer of at least `dst_capacity` bytes.
/// - The caller must ensure `dst` remains valid for the duration of the call.
/// - `ucl_init()` must have been called before invoking this function.
unsafe fn decompress_ptr(
    src: &[u8],
    dst: *mut u8,
    dst_capacity: u32,
) -> std::result::Result<u32, UclErrorKind> {
    assert!(
        INIT_RESULT.get().is_some(),
        "ucl_init was not called before attempting decompression"
    );

    let src_len = to_u32_src_len(src.len())?;

    let mut dst_len = dst_capacity;

    // SAFETY: The caller guarantees that `dst` is valid for `dst_capacity` bytes.
    // `src` is a valid slice reference. We pass null for wrkmem as the safe
    // decompression variant doesn't require it.
    let res = unsafe {
        ucl_nrv2b_decompress_safe_8(src.as_ptr(), src_len, dst, &mut dst_len, ptr::null())
    };
    match res {
        0 => {
            // Defensive check: if the library reports writing more data than the buffer
            // can hold, treat this as an output overrun error. This catches cases where
            // the library might have a bug or the compressed data is malformed.
            if dst_len > dst_capacity {
                return Err(UclErrorKind::OutputOverrun);
            }
            Ok(dst_len)
        }
        _ => Err(UclErrorKind::from(res)),
    }
}

/// Decompress a NRV compressed buffer into another buffer.
///
/// If `dst` is not big enough to hold the
/// decompressed buffer, this will return `Err(UclErrorKind::OutputOverrun)`.
/// If decompression succeeded, this will return the number of usable bytes in `dst`.
///
/// # Panics
/// If [ucl_init] was not called prior to calling this function, this function will panic.
///
/// ```
/// # uclcli::ucl_init();
/// let mut buf = [0xffu8; 1024];
/// assert_eq!(uclcli::decompress_into_buffer(b"\x92\xa5\xaa\xa1\x00\x00\x00\x00\x00\x04\x80\xff", &mut buf), Ok(1024));
/// assert_eq!(buf, [0xa5u8; 1024]);
/// ```
pub fn decompress_into_buffer(
    src: &[u8],
    dst: &mut [u8],
) -> std::result::Result<u32, UclErrorKind> {
    let dst_len = to_u32_dst_len(dst.len())?;

    // SAFETY: dst is a valid mutable slice, and dst_len is its length.
    // The slice provides a valid pointer for the buffer.
    unsafe { decompress_ptr(src, dst.as_mut_ptr(), dst_len) }
}

/// Decompress a NRV compressed buffer into a newly allocated buffer.
///
/// If `dst_capacity` is not enough to hold the decompressed buffer, this will
/// return `Err(UclErrorKind::OutputOverrun)`.
/// If `dst_capacity` exceeds the maximum allowed size (1GB), this will return
/// `Err(UclErrorKind::DstTooLarge)` to prevent memory exhaustion attacks.
/// If decompression succeeded, this will return the decompressed buffer.
///
/// # Panics
/// If [ucl_init] was not called prior to calling this function, this function will panic.
///
/// ```
/// # uclcli::ucl_init();
/// assert_eq!(uclcli::decompress(b"\x92\xa5\xaa\xa1\x00\x00\x00\x00\x00\x04\x80\xff", 1024).unwrap(), [0xa5u8; 1024]);
/// ```
pub fn decompress(src: &[u8], dst_capacity: u32) -> std::result::Result<Vec<u8>, UclErrorKind> {
    // Validate dst_capacity to prevent memory exhaustion attacks
    if dst_capacity > MAX_DST_CAPACITY {
        return Err(UclErrorKind::DstTooLarge);
    }

    let mut dst = Vec::with_capacity(dst_capacity as usize);

    // SAFETY: We've allocated `dst_capacity` bytes of capacity, and
    // `decompress_ptr` will write at most that many bytes. After successful
    // decompression, we set the length to the actual bytes written.
    unsafe {
        let new_length = decompress_ptr(src, dst.as_mut_ptr(), dst_capacity)?;
        dst.set_len(new_length as usize);
    }

    Ok(dst)
}

/// Determine the destination buffer size requirement for [compress_into_buffer].
///
/// citing from libucl's README:
///
/// > UCL will expand non-compressible data by a little amount. I suggest
/// > using this formula for a worst-case expansion calculation:
/// >
/// >   output_block_size = input_block_size + (input_block_size / 8) + 256
pub const fn minimum_compression_buffer_size(src_len: usize) -> usize {
    src_len + (src_len / 8) + 256
}

/// # Safety
///
/// - `dst` must be a valid pointer to a buffer of at least `dst_capacity` bytes.
/// - `dst_capacity` must be >= `minimum_compression_buffer_size(src.len())`.
/// - The caller must ensure `dst` remains valid for the duration of the call.
/// - `ucl_init()` must have been called before invoking this function.
unsafe fn compress_ptr(
    src: &[u8],
    dst: *mut u8,
    dst_capacity: u32,
) -> std::result::Result<u32, UclErrorKind> {
    assert!(
        INIT_RESULT.get().is_some(),
        "ucl_init was not called before attempting compression"
    );

    let src_len = to_u32_src_len(src.len())?;

    let mut dst_len = dst_capacity;

    // SAFETY: The caller guarantees that `dst` is valid for at least
    // `dst_capacity` bytes, which must be >= minimum_compression_buffer_size.
    // `src` is a valid slice reference. We pass null for callback, config,
    // and result as we don't need progress reporting or detailed statistics.
    let res = unsafe {
        ucl_nrv2b_99_compress(
            src.as_ptr(),
            src_len,
            dst,
            &mut dst_len,
            ptr::null(), /* no progress callback */
            DEFAULT_COMPRESSION_LEVEL,
            ptr::null(), /* default compression config */
            ptr::null(), /* no statistical output */
        )
    };
    match res {
        0 => {
            // Defensive check: if the library reports writing more data than the buffer
            // can hold, treat this as an output overrun error.
            if dst_len > dst_capacity {
                return Err(UclErrorKind::OutputOverrun);
            }
            Ok(dst_len)
        }
        _ => Err(UclErrorKind::from(res)),
    }
}

/// NRV compress a buffer into another buffer.
///
/// If `dst` is not big enough to hold the compressed
/// buffer, this will return `Err(UclErrorKind::DstTooSmall)`. See also
/// [minimum_compression_buffer_size] to find out how big `dst` should be.
/// If compression succeeded, this will return the number of usable bytes in `dst`.
///
/// # Panics
/// If [ucl_init] was not called prior to calling this function, this function will panic.
/// ```
/// # uclcli::ucl_init();
/// let src = [0; 1024];
/// let mut dst = vec![0xffu8; uclcli::minimum_compression_buffer_size(src.len())];
///
/// let result = uclcli::compress_into_buffer(&src, &mut dst);
/// assert_eq!(result, Ok(12));
///
/// let nb = result.unwrap() as usize;
///
/// assert_eq!(&dst[..nb], b"\x92\x00\xaa\xa1\x00\x00\x00\x00\x00\x04\x80\xff");
/// assert_eq!(&dst[nb..], &vec![0xffu8; dst.len() - nb]);
/// ```
pub fn compress_into_buffer(src: &[u8], dst: &mut [u8]) -> std::result::Result<u32, UclErrorKind> {
    if dst.len() < minimum_compression_buffer_size(src.len()) {
        return Err(UclErrorKind::DstTooSmall);
    }

    let dst_len = to_u32_dst_len(dst.len())?;

    // SAFETY: We've verified dst.len() >= minimum_compression_buffer_size(src.len()),
    // and dst_len fits in u32. The slice provides a valid pointer for the buffer.
    unsafe { compress_ptr(src, dst.as_mut_ptr(), dst_len) }
}

/// NRV compress a buffer into a newly allocated buffer.
///
/// # Panics
/// If [ucl_init] was not called prior to calling this function, this function will panic.
/// ```
/// # uclcli::ucl_init();
/// let src = [0; 1024];
///
/// assert_eq!(uclcli::compress(&src).unwrap(), b"\x92\x00\xaa\xa1\x00\x00\x00\x00\x00\x04\x80\xff");
/// ```
pub fn compress(src: &[u8]) -> std::result::Result<Vec<u8>, UclErrorKind> {
    let capacity = minimum_compression_buffer_size(src.len());
    let mut dst = Vec::with_capacity(capacity);

    let dst_len = to_u32_dst_len(capacity)?;

    // SAFETY: We've allocated `capacity` bytes, which equals
    // minimum_compression_buffer_size(src.len()). After successful
    // compression, we set the length to the actual bytes written.
    unsafe {
        let new_length = compress_ptr(src, dst.as_mut_ptr(), dst_len)?;
        dst.set_len(new_length as usize);
    }
    Ok(dst)
}

#[cfg(test)]
mod tests {
    use super::{compress_into_buffer, decompress, decompress_into_buffer, ucl_init, UclErrorKind};

    #[test]
    fn compress_buffer_nothing() {
        ucl_init();
        let mut buf = [0xa5u8; 256];
        assert_eq!(compress_into_buffer(&[], &mut buf).unwrap(), 8);
        assert_eq!(&buf[..8], b"\x00\x00\x00\x00\x00\x04\x80\xff");
        assert!(&buf[8..].iter().all(|b| *b == 0xa5));
    }

    #[test]
    fn compress_buffer_too_small() {
        ucl_init();
        let mut buf = [0xa5u8; 4];
        assert_eq!(
            compress_into_buffer(b"\xde\xad\xbe\xef", &mut buf).unwrap_err(),
            UclErrorKind::DstTooSmall
        );
        assert!(buf.iter().all(|b| *b == 0xa5));
    }

    #[test]
    fn compress_buffer_too_big() {
        ucl_init();
        let mut buf = vec![0u8; 4 * 1024 * 1024 * 1024];
        assert_eq!(
            compress_into_buffer(b"\xde\xad\xbe\xef", &mut buf).unwrap_err(),
            UclErrorKind::DstTooLarge
        );
    }

    #[test]
    fn decompress_buffer_dst_too_big() {
        ucl_init();
        let mut buf = vec![0u8; 4 * 1024 * 1024 * 1024];
        assert_eq!(
            decompress_into_buffer(b"\xde\xad\xbe\xef", &mut buf).unwrap_err(),
            UclErrorKind::DstTooLarge
        );
    }

    #[test]
    fn decompress_buffer_src_too_big() {
        ucl_init();
        let mut buf = vec![0u8; 4];
        let input = vec![0u8; 4 * 1024 * 1024 * 1024];
        assert_eq!(
            decompress_into_buffer(&input, &mut buf).unwrap_err(),
            UclErrorKind::SrcTooLarge
        );
    }

    #[test]
    fn decompress_buffer_nothing() {
        ucl_init();
        let compressed = b"\x00\x00\x00\x00\x00\x04\x80\xff";
        let mut buf = [0xa5u8; 8];
        assert_eq!(
            decompress_into_buffer(compressed.as_ref(), &mut buf).unwrap(),
            0
        );
        assert!(buf.iter().all(|b| *b == 0xa5));
    }

    #[test]
    fn decompress_buffer_8k_too_small() {
        ucl_init();
        let compressed =
            b"\x92\x00\xaa\xa8\xc9\x55\x54\x64\xaa\xaa\x32\x55\x55\x08\x00\x00\x00\x00\x00\x24\xff";
        let mut buf = [0xa5u8; 8191];
        assert_eq!(
            decompress_into_buffer(compressed.as_ref(), &mut buf).unwrap_err(),
            UclErrorKind::OutputOverrun
        );
    }

    #[test]
    fn decompress_src_too_big() {
        ucl_init();
        let input = vec![0u8; 4 * 1024 * 1024 * 1024];
        assert_eq!(
            decompress(&input, 4).unwrap_err(),
            UclErrorKind::SrcTooLarge
        );
    }

    #[test]
    fn decompress_nothing() {
        ucl_init();
        let compressed = b"\x00\x00\x00\x00\x00\x04\x80\xff";
        assert_eq!(decompress(compressed.as_ref(), 8).unwrap(), b"");
    }

    #[test]
    fn decompress_8k_too_small() {
        ucl_init();
        let compressed =
            b"\x92\x00\xaa\xa8\xc9\x55\x54\x64\xaa\xaa\x32\x55\x55\x08\x00\x00\x00\x00\x00\x24\xff";
        assert_eq!(
            decompress(compressed.as_ref(), 8191).unwrap_err(),
            UclErrorKind::OutputOverrun
        );
    }

    // Integration tests for round-trip compression/decompression
    use super::compress;

    #[test]
    fn round_trip_empty() {
        ucl_init();
        let original: Vec<u8> = vec![];
        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, 1024).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn round_trip_zeros() {
        ucl_init();
        let original = vec![0u8; 1024];
        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len() as u32).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn round_trip_ones() {
        ucl_init();
        let original = vec![0xffu8; 1024];
        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len() as u32).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn round_trip_pattern() {
        ucl_init();
        let original: Vec<u8> = (0..=255).cycle().take(4096).collect();
        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len() as u32).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn round_trip_random_like() {
        ucl_init();
        // Pseudo-random pattern using a simple PRNG seed
        let mut original = Vec::with_capacity(8192);
        let mut state: u32 = 0xDEADBEEF;
        for _ in 0..8192 {
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            original.push((state >> 16) as u8);
        }
        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len() as u32).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn round_trip_large() {
        ucl_init();
        let original = vec![0xa5u8; 1024 * 1024]; // 1MB
        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len() as u32).unwrap();
        assert_eq!(original, decompressed);
    }
}
