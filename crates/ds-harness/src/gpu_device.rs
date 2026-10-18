//! Opening the GPU a hybrid harness paints on, as shell-host's offscreen context does: a
//! non-empty `WGPU_ADAPTER_NAME` names the adapter (a case-insensitive substring of its name),
//! else [`AdapterPref`] ranks the kinds; Vulkan before GL, software rasterisers last. The first
//! adapter that yields a device wins. The ranking and the device request are `blitz_kit::adapter`.

use crate::error::HarnessError;
use crate::gpu_diagnostics::GpuDiagnostics;
use blitz_kit::adapter::{ADAPTER_ENV, AdapterPref, block_on, ranked, request_device};
use std::sync::{Mutex, PoisonError};
use wgpu_context::DeviceHandle;

/// The devices this process opened, one per adapter preference and diagnostics setting, kept
/// until the process exits. Harnesses on the same preference share one instance and device.
///
/// Opening is serialised by this lock because the Vulkan loader's ICD scan and device creation
/// are not safe from several threads at once (a SIGSEGV in
/// `terminator_EnumerateInstanceExtensionProperties` under parallel tests). Devices are shared
/// and never dropped because tearing one down while another test creates its own deadlocks the
/// NVIDIA driver: `vkDestroyDevice` and EGL `destroy_context` on one thread against
/// `vkEnumerateDeviceExtensionProperties` and `create_pipeline_layout` on others hung a test
/// binary for hours (quire ds-blitz and anyview, 2026-10-15). A wgpu device is safe to use from
/// many threads, and each harness still owns its renderer, target and resources.
static OPENED: Mutex<Vec<Opened>> = Mutex::new(Vec::new());

/// One device this process opened, and what it was opened for.
struct Opened {
    pref: AdapterPref,
    diagnostics: GpuDiagnostics,
    handle: DeviceHandle,
    adapter: String,
}

/// A device on the preferred adapter, and the adapter's `name (backend)`: the one this process
/// already opened for the same preference and diagnostics, else a new one.
pub(crate) fn open_device(
    pref: &AdapterPref,
    diagnostics: GpuDiagnostics,
) -> Result<(DeviceHandle, String), HarnessError> {
    let pref = pref
        .clone()
        .with_env(std::env::var(ADAPTER_ENV).ok().as_deref());
    let mut opened = OPENED.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(known) = opened
        .iter()
        .find(|known| known.pref == pref && known.diagnostics == diagnostics)
    {
        return Ok((known.handle.clone(), known.adapter.clone()));
    }
    let (handle, adapter) = open_new(&pref, diagnostics)?;
    opened.push(Opened {
        pref,
        diagnostics,
        handle: handle.clone(),
        adapter: adapter.clone(),
    });
    Ok((handle, adapter))
}

/// A new instance and device on the preferred adapter. Called with [`OPENED`] held.
fn open_new(
    pref: &AdapterPref,
    diagnostics: GpuDiagnostics,
) -> Result<(DeviceHandle, String), HarnessError> {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle().with_env();
    // Not the environment's or the build's flags: a harness is headless and many run at once.
    desc.flags = diagnostics.flags();
    if std::env::var_os("WGPU_BACKEND").is_none() {
        desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
    }
    let instance = wgpu::Instance::new(desc);
    let adapters = block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    let mut failed = Vec::new();
    for adapter in ranked(adapters, pref) {
        let info = adapter.get_info();
        let label = format!("{} ({:?})", info.name, info.backend);
        match request_device(&adapter, "ds-harness") {
            Ok((device, queue)) => {
                let handle = DeviceHandle {
                    instance,
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
