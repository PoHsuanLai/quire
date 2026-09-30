//! Opening the GPU a hybrid harness paints on, as shell-host's offscreen context does: a
//! non-empty `WGPU_ADAPTER_NAME` names the adapter (a case-insensitive substring of its name),
//! else [`AdapterPref`] ranks the kinds; Vulkan before GL, software rasterisers last. The first
//! adapter that yields a device wins. The ranking and the device request are `blitz_kit::adapter`.

use crate::error::HarnessError;
use blitz_kit::adapter::{ADAPTER_ENV, AdapterPref, block_on, ranked, request_device};
use wgpu_context::DeviceHandle;

/// A device on the preferred adapter, and the adapter's `name (backend)`.
pub(crate) fn open_device(pref: &AdapterPref) -> Result<(DeviceHandle, String), HarnessError> {
    let pref = pref
        .clone()
        .with_env(std::env::var(ADAPTER_ENV).ok().as_deref());
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle().with_env();
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
