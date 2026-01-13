
use super::{to_u32_dst_len, to_u32_src_len, UclErrorKind};

#[test]
fn test_to_u32_src_len_valid() {
    assert_eq!(to_u32_src_len(1024), Ok(1024));
}

#[test]
fn test_to_u32_src_len_too_large() {
    assert_eq!(
        to_u32_src_len(u32::MAX as usize + 1),
        Err(UclErrorKind::SrcTooLarge)
    );
}

#[test]
fn test_to_u32_dst_len_valid() {
    assert_eq!(to_u32_dst_len(1024), Ok(1024));
}

#[test]
fn test_to_u32_dst_len_too_large() {
    assert_eq!(
        to_u32_dst_len(u32::MAX as usize + 1),
        Err(UclErrorKind::DstTooLarge)
    );
}
