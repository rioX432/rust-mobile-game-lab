//! Renderer used inside a larger Rust host that owns the GPU and presentation.
//!
//! This is intentionally a Rust API, not a C ABI: wgpu handles must never cross
//! independent native library / JNI / Swift binary boundaries.
//! The host retains responsibility for the Surface, frame scheduling, final
//! Queue submission and presentation. This renderer only encodes a render pass.
use super::{shape_pipeline, tessellate};
use game_core::Scene;

/// A host-owned GPU renderer for the existing 2D baseline.
///
/// `new` takes clones of the *existing* host Device/Queue. It never requests
/// an adapter, creates another Device, configures a Surface or presents.
///
/// The host must provide a single-sampled render target with the `format`
/// selected at construction. The caller owns the target view and encoder.
/// This type is an initial 2D integration seam; future 3D backends must
/// preserve the host-ownership contract without assuming identical APIs.
pub struct EmbeddedRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: u64,
}

impl EmbeddedRenderer {
    /// Supply the host's Device/Queue and color attachment format.
    ///
    /// Share these objects in a single Rust link unit. This does not define
    /// an ABI for transferring wgpu objects across separate shared libraries.
    pub async fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let pipeline = shape_pipeline(&device, format);
        if let Some(error) = validation.pop().await {
            return Err(format!("create embedded shape pipeline: {error}"));
        }
        Ok(Self {
            device,
            queue,
            pipeline,
            vertex_buffer: None,
            vertex_capacity: 0,
        })
    }

    /// Encode game drawing into a view already owned by the host.
    ///
    /// The host owns frame ordering and *must* submit `encoder` via the same
    /// Queue, then decide when to composite/present. A transparent Clear is
    /// appropriate when compositing this layer later; Load requires previously
    /// initialized content. `Scene::size` must match the intended viewport.
    ///
    /// No per-frame readback, `Surface` access or `Queue::submit` happens here.
    pub fn encode(
        &mut self,
        scene: &Scene,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        load: wgpu::LoadOp<wgpu::Color>,
    ) -> Result<(), String> {
        let vertices = tessellate(scene)?;
        let bytes = bytemuck::cast_slice(&vertices);
        let byte_len = bytes.len() as u64;
        let count = u32::try_from(vertices.len()).map_err(|_| "too many vertices")?;
        if byte_len > self.device.limits().max_buffer_size {
            return Err("scene exceeds device buffer limits".into());
        }
        if byte_len > self.vertex_capacity {
            self.vertex_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("embedded 2D vertices"),
                size: byte_len,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.vertex_capacity = byte_len;
        }
        if let Some(buffer) = &self.vertex_buffer
            && !bytes.is_empty()
        {
            self.queue.write_buffer(buffer, 0, bytes);
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("embedded 2D game shapes"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        if let Some(buffer) = &self.vertex_buffer
            && count > 0
        {
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, buffer.slice(..byte_len));
            pass.draw(0..count, 0..1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::{Shape, ShapeKind};

    // This test needs a compatible GPU adapter. It is kept compile-checked by
    // the Rust test target and can be explicitly enabled on hardware runners.
    #[test]
    #[ignore = "requires compatible GPU adapter"]
    fn host_can_encode_a_game_frame_without_a_surface() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("GPU adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("embedded renderer test"),
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        }))
        .expect("GPU device");
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("host game offscreen layer"),
            size: wgpu::Extent3d {
                width: 200,
                height: 100,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let mut game = pollster::block_on(EmbeddedRenderer::new(
            device.clone(),
            queue.clone(),
            wgpu::TextureFormat::Rgba8Unorm,
        ))
        .expect("embedded renderer");
        let scene = Scene {
            size: [200.0, 100.0],
            shapes: vec![Shape {
                kind: ShapeKind::Rectangle,
                bounds: [20.0, 15.0, 60.0, 40.0],
                color: [1.0, 0.5, 0.0, 0.7],
            }],
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("host frame"),
        });
        game.encode(
            &scene,
            &target.create_view(&wgpu::TextureViewDescriptor::default()),
            &mut encoder,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }),
        )
        .expect("game frame");
        // The host, not the game, submits the frame.
        queue.submit([encoder.finish()]);
    }
}
