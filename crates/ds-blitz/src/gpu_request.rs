//! What an app asks of the GPU the window draws with: wgpu features and limits beyond the
//! renderer's defaults, such as timestamp queries for a frame timer. The renderer asks the
//! adapter for them when it creates the device; an adapter without a feature fails the device
//! request, so an app asks only for what it needs and has a fallback ready.

use dioxus_native::DioxusNativeWindowRenderer;

/// The wgpu features and limits an app wants its window's device created with. `None` leaves the
/// renderer's own choice.
#[derive(Debug, Clone, Default)]
pub struct GpuRequest {
    /// Features to request, in addition to what the renderer needs.
    pub features: Option<wgpu::Features>,
    /// Limits to request in place of the renderer's.
    pub limits: Option<wgpu::Limits>,
}

impl GpuRequest {
    /// A window renderer that will create its device with this request.
    pub(crate) fn renderer(&self) -> DioxusNativeWindowRenderer {
        DioxusNativeWindowRenderer::with_features_and_limits(self.features, self.limits.clone())
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
            limits: None,
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
