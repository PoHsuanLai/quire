//! Opening the GPU a hybrid harness paints on, as shell-host's offscreen context does: a
//! non-empty `WGPU_ADAPTER_NAME` names the adapter (a case-insensitive substring of its name),
//! else [`AdapterPref`] ranks the kinds; Vulkan before GL, software rasterisers last. The first
//! adapter that yields a device wins. The ranking and the device request are `blitz_kit::adapter`.

use crate::error::HarnessError;
use crate::gpu_diagnostics::GpuDiagnostics;
use blitz_kit::adapter::{ADAPTER_ENV, AdapterPref, block_on, ranked, request_device};
use std::sync::{Mutex, PoisonError};
use wgpu_context::DeviceHandle;

/// Held while a harness opens its instance, enumerates adapters and requests its device. The
/// Vulkan loader's ICD scan and device creation are not safe to run from several threads at once
/// (a SIGSEGV in `terminator_EnumerateInstanceExtensionProperties` under parallel tests, a few
/// runs in a thousand once debug naming was off), so harnesses open one at a time. It guards no
/// data and no device is shared: each harness still owns its own instance and device, and which
/// harness opens first changes nothing.
static OPENING: Mutex<()> = Mutex::new(());

/// A device on the preferred adapter, and the adapter's `name (backend)`.
pub(crate) fn open_device(
    pref: &AdapterPref,
    diagnostics: GpuDiagnostics,
) -> Result<(DeviceHandle, String), HarnessError> {
    let _one_at_a_time = OPENING.lock().unwrap_or_else(PoisonError::into_inner);
    let pref = pref
        .clone()
        .with_env(std::env::var(ADAPTER_ENV).ok().as_deref());
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle().with_env();
    // Not the environment's or the build's flags: a harness is headless and many run at once.
    desc.flags = diagnostics.flags();
    if std::env::var_os("WGPU_BACKEND").is_none() {
        desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
    }
    let instance = wgpu::Instance::new(desc);
    let adapters = block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    let mut failed = Vec::new();
    for adapter in ranked(adapters, &pref) {
        let info = adapter.get_info();
        let label = format!("{} ({:?})", info.name, info.backend);
        match request_device(&adapter, "ds-harness") {
            Ok((device, queue)) => {
                let handle = DeviceHandle {
                    instance: instance.clone(),
                    adapter,
                    device,
                    queue,
                };
                return Ok((handle, label));
            }
            Err(error) => failed.push(format!("{label}: {error}")),
        }
    }
    Err(HarnessError::Renderer(if failed.is_empty() {
        "no GPU adapter".into()
    } else {
        format!("no GPU device: {}", failed.join("; "))
    }))
}
