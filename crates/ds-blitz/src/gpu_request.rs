//! What an app asks of the GPU the window draws with: wgpu features and limits beyond the
//! renderer's defaults, such as timestamp queries for a frame timer. The renderer asks the
//! adapter for them when it creates the device; an adapter without a feature fails the device
//! request, so an app asks only for what it needs and has a fallback ready.
//!
//! It also names the backends wgpu probes for an adapter. Probing every backend costs well over
//! 100 ms at launch, so the default is the one the platform draws with; a machine where that one
//! finds no adapter falls back once to wgpu's full set.

use dioxus_native::{DioxusNativeWindowRenderer, RendererOptions};

/// The wgpu features and limits an app wants its window's device created with. `None` leaves the
/// renderer's own choice.
#[derive(Debug, Clone, Default)]
pub struct GpuRequest {
    /// Features to request, in addition to what the renderer needs.
    pub features: Option<wgpu::Features>,
    /// Limits to request in place of the renderer's.
    pub limits: Option<wgpu::Limits>,
    /// The backends to probe for an adapter. `None` leaves quire's choice, the platform's
    /// primary backend. The `WGPU_BACKEND` environment variable wins
    /// over both.
    pub backends: Option<wgpu::Backends>,
}

/// The backend a platform draws with, so that the others are not probed at launch: Vulkan on
/// Linux and the BSDs (GL cannot present there, since the renderer gives wgpu no display
/// handle), Metal on Apple platforms, and DX12 on Windows (every supported Windows has it,
/// and a machine without an adapter for it falls back to the full set). `None` elsewhere
/// (Android, the web) leaves wgpu's default.
fn primary_backends() -> Option<wgpu::Backends> {
    if cfg!(target_vendor = "apple") {
        Some(wgpu::Backends::METAL)
    } else if cfg!(windows) {
        Some(wgpu::Backends::DX12)
    } else if cfg!(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    )) {
        Some(wgpu::Backends::VULKAN)
    } else {
        None
    }
}

impl GpuRequest {
    /// A window renderer that will create its device with this request.
    pub(crate) fn renderer(&self) -> DioxusNativeWindowRenderer {
        DioxusNativeWindowRenderer::with_options(RendererOptions {
            features: self.features,
            limits: self.limits.clone(),
            backends: self.backends.or_else(primary_backends),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::GpuRequest;
    use crate::launch::AppConfig;
    use crate::window_size::WindowSize;

    #[test]
    fn an_app_config_carries_the_request_to_the_windows_it_opens() {
        let request = GpuRequest {
            features: Some(wgpu::Features::TIMESTAMP_QUERY),
            ..Default::default()
        };
        let config = AppConfig::new("t", WindowSize::new(400, 300)).with_gpu(request.clone());
        assert_eq!(config.gpu().features, request.features);
        assert!(
            AppConfig::new("t", WindowSize::new(400, 300))
                .gpu()
                .features
                .is_none()
        );
    }
}
