// Original odht 0.3.1 target selection, without cfg-if or an unused no_simd feature.
// The original operation is chosen at compiler build time, not per request.
#[cfg(all(target_feature = "sse2", any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
mod sse2;
#[cfg(all(target_feature = "sse2", any(target_arch = "x86", target_arch = "x86_64"), not(miri)))]
pub(crate) use sse2::{GROUP_SIZE, GroupQuery};
#[cfg(not(all(target_feature = "sse2", any(target_arch = "x86", target_arch = "x86_64"), not(miri))))]
mod no_simd;
#[cfg(not(all(target_feature = "sse2", any(target_arch = "x86", target_arch = "x86_64"), not(miri))))]
pub(crate) use no_simd::{GROUP_SIZE, GroupQuery};

pub(crate) const REFERENCE_GROUP_SIZE: usize = 16;
