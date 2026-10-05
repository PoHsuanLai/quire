//! Whether a hybrid harness's wgpu instance names its objects and validates calls.
//!
//! Every hybrid harness opens its own instance and device. With wgpu's `DEBUG` flag (on by
//! default in debug builds) the Vulkan backend loads `VK_EXT_debug_utils` and names every
//! texture, buffer and view through `vkSetDebugUtilsObjectNameEXT`; the Vulkan loader walks its
//! device list without a lock while it does, so naming from many harnesses in one test process
//! can crash in `loader_get_icd_and_device` (a SIGSEGV seen with parallel tests, anyview#17).
//! A harness has no use for the names, so it opens its instance without them unless a test
//! asks.

use wgpu::InstanceFlags;

/// Whether a hybrid harness's GPU instance carries debug names and validation
/// ([`HarnessConfig::with_gpu_diagnostics`](crate::HarnessConfig::with_gpu_diagnostics)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GpuDiagnostics {
    /// No object names and no validation layer: the instance flags are empty whatever
    /// `WGPU_DEBUG` and `WGPU_VALIDATION` say, so parallel harnesses cannot reach the loader's
    /// naming race.
    #[default]
    Off,
    /// wgpu's debugging flags (names, validation, indirect-call validation), refined by the
    /// `WGPU_*` environment variables as wgpu reads them. For one debugging session, not for a
    /// suite that runs harnesses in parallel.
    On,
}

impl GpuDiagnostics {
    /// The instance flags this setting opens an instance with.
    pub(crate) fn flags(self) -> InstanceFlags {
        match self {
            GpuDiagnostics::Off => InstanceFlags::empty(),
            GpuDiagnostics::On => InstanceFlags::debugging().with_env(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GpuDiagnostics;
    use wgpu::InstanceFlags;

    #[test]
    fn a_harness_names_nothing_and_validates_nothing_by_default() {
        let flags = GpuDiagnostics::default().flags();
        assert!(flags.is_empty(), "{flags:?}");
        assert!(!flags.contains(InstanceFlags::DEBUG));
        assert!(!flags.contains(InstanceFlags::VALIDATION));
    }

    #[test]
    fn on_asks_for_names_and_validation() {
        // `with_env` can only remove them if the test's environment says so.
        let flags = GpuDiagnostics::On.flags();
        let env_off = |key: &str| std::env::var(key).is_ok_and(|v| v == "0");
        assert!(flags.contains(InstanceFlags::DEBUG) || env_off("WGPU_DEBUG"));
        assert!(flags.contains(InstanceFlags::VALIDATION) || env_off("WGPU_VALIDATION"));
    }
}
