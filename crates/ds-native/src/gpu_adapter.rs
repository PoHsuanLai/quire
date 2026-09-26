//! Choosing and opening the GPU a hybrid [`Harness`](crate::Harness) paints on, as shell-host's
//! offscreen context does (`GpuContext::offscreen`): a non-empty `WGPU_ADAPTER_NAME` names the
//! adapter (a case-insensitive substring of its name), else [`AdapterPref`] ranks the kinds;
//! Vulkan before GL, software rasterisers last. The first adapter that yields a device wins.

use crate::error::NativeError;
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use wgpu_context::DeviceHandle;

/// The environment variable that names the adapter, overriding any [`AdapterPref`].
pub const ADAPTER_ENV: &str = "WGPU_ADAPTER_NAME";

/// Which GPU a hybrid harness prefers (shell-host's `AdapterPref`, the same contract).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AdapterPref {
    /// wgpu's high-performance order: discrete, integrated, virtual, other, software.
    #[default]
    Auto,
    /// A discrete GPU first.
    Discrete,
    /// An integrated GPU first (the AMD iGPU on the dev machine).
    Integrated,
    /// The adapter whose name contains this, ignoring case; then as `Auto`.
    Named(String),
}

impl AdapterPref {
    /// The preference in force once the environment is read: a non-empty `WGPU_ADAPTER_NAME`
    /// wins.
    fn with_env(self, env_value: Option<&str>) -> AdapterPref {
        match env_value.map(str::trim) {
            Some(name) if !name.is_empty() => AdapterPref::Named(name.to_owned()),
            _ => self,
        }
    }
}

/// A device on the preferred adapter, and the adapter's `name (backend)`.
pub(crate) fn open_device(pref: &AdapterPref) -> Result<(DeviceHandle, String), NativeError> {
    let pref = pref
        .clone()
        .with_env(std::env::var(ADAPTER_ENV).ok().as_deref());
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle().with_env();
    if std::env::var_os("WGPU_BACKEND").is_none() {
        desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
    }
    let instance = wgpu::Instance::new(desc);
    let mut adapters = block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    adapters.sort_by_key(|adapter| sort_key(&pref, &adapter.get_info()));
    let mut failed = Vec::new();
    for adapter in adapters {
        let info = adapter.get_info();
        let label = format!("{} ({:?})", info.name, info.backend);
        let features =
            adapter.features() & (wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE);
        let device = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("ds-native harness"),
            required_features: features,
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }));
        match device {
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
    Err(NativeError::Renderer(if failed.is_empty() {
        "no GPU adapter".into()
    } else {
        format!("no GPU device: {}", failed.join("; "))
    }))
}

/// Smaller is tried first: (name miss, software, kind rank, backend rank).
fn sort_key(pref: &AdapterPref, info: &wgpu::AdapterInfo) -> (u8, u8, u8, u8) {
    let name_miss = match pref {
        AdapterPref::Named(wanted) => {
            u8::from(!info.name.to_lowercase().contains(&wanted.to_lowercase()))
        }
        _ => 0,
    };
    let software = u8::from(info.device_type == wgpu::DeviceType::Cpu);
    let backend = u8::from(info.backend != wgpu::Backend::Vulkan);
    (
        name_miss,
        software,
        kind_rank(pref, info.device_type),
        backend,
    )
}

fn kind_rank(pref: &AdapterPref, kind: wgpu::DeviceType) -> u8 {
    use wgpu::DeviceType::{Cpu, DiscreteGpu, IntegratedGpu, Other, VirtualGpu};
    let order = match pref {
        AdapterPref::Integrated => [IntegratedGpu, DiscreteGpu, VirtualGpu, Other, Cpu],
        AdapterPref::Auto | AdapterPref::Discrete | AdapterPref::Named(_) => {
            [DiscreteGpu, IntegratedGpu, VirtualGpu, Other, Cpu]
        }
    };
    order
        .iter()
        .position(|k| *k == kind)
        .map_or(u8::MAX, |p| p as u8)
}

struct Unpark(std::thread::Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Drive wgpu's setup futures on this thread (they resolve without an executor on native).
pub(crate) fn block_on<T>(future: impl Future<Output = T>) -> T {
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        std::thread::park();
    }
}

#[cfg(test)]
mod tests {
    use super::AdapterPref;

    #[test]
    fn a_named_adapter_in_the_environment_wins() {
        let cases: &[(AdapterPref, Option<&str>, AdapterPref)] = &[
            (AdapterPref::Integrated, None, AdapterPref::Integrated),
            (AdapterPref::Auto, Some("  "), AdapterPref::Auto),
            (
                AdapterPref::Discrete,
                Some("AMD"),
                AdapterPref::Named("AMD".into()),
            ),
        ];
        for (pref, env, expected) in cases {
            assert_eq!(pref.clone().with_env(*env), *expected, "{env:?}");
        }
    }
}
