use bytemuck::{Pod, Zeroable};
use game_core::{Scene, ShapeKind};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

mod embedded;
pub use embedded::EmbeddedRenderer;

/// The host must drop the renderer and recreate its wgpu surface on this error.
pub const SURFACE_LOST: &str = "surface lost; recreate the surface and renderer";
const CIRCLE_SEGMENTS: usize = 64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub struct PreparedFrame {
    commands: wgpu::CommandBuffer,
    texture: wgpu::SurfaceTexture,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    config: wgpu::SurfaceConfiguration,
    size: [u32; 2],
    needs_configure: bool,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: u64,
    presented_frames: u64,
    device_lost: Arc<AtomicBool>,
}

impl Renderer {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|e| format!("request adapter: {e}"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("2D renderer"),
                // Shape rendering does not need WebGPU's higher shader limits.
                required_limits: wgpu::Limits::downlevel_defaults()
                    .using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .map_err(|e| format!("request device: {e}"))?;
        let config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or("surface has no supported configuration")?;
        let device_lost = Arc::new(AtomicBool::new(false));
        let lost = Arc::clone(&device_lost);
        device.set_device_lost_callback(move |_, _| lost.store(true, Ordering::Release));
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("2D shapes"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shapes.wgsl").into()),
        });
        let pipeline = shape_pipeline(&device, config.format);
        if let Some(error) = validation.pop().await {
            return Err(format!("create shape pipeline: {error}"));
        }
        let mut renderer = Self {
            surface,
            device,
            queue,
            pipeline,
            config,
            size: [width, height],
            needs_configure: true,
            vertex_buffer: None,
            vertex_capacity: 0,
            presented_frames: 0,
            device_lost,
        };
        renderer.configure()?;
        Ok(renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = [width, height];
        self.needs_configure = true;
    }

    /// Counts submitted presentations; skipped/occluded/zero-size frames do not count.
    pub fn presented_frames(&self) -> u64 {
        self.presented_frames
    }

    fn configure(&mut self) -> Result<(), String> {
        if self.size.contains(&0) {
            return Ok(());
        }
        if self
            .size
            .iter()
            .any(|&v| v > self.device.limits().max_texture_dimension_2d)
        {
            return Err("surface size exceeds device limits".into());
        }
        self.config.width = self.size[0];
        self.config.height = self.size[1];
        self.surface.configure(&self.device, &self.config);
        #[cfg(target_vendor = "apple")]
        {
            // SAFETY: The guard keeps the Metal surface alive. Only its locked
            // layer's acquisition timeout is changed; no GPU resource is freed.
            if let Some(surface) = unsafe { self.surface.as_hal::<wgpu::hal::api::Metal>() } {
                let layer = surface.render_layer().lock();
                // wgpu disables this on configure. Bound acquisition so a
                // cancelled worker can release its retained layer after a stall.
                layer.setAllowsNextDrawableTimeout(true);
                if !layer.allowsNextDrawableTimeout() {
                    return Err("Metal drawable acquisition timeout is not enabled".into());
                }
            }
        }
        self.needs_configure = false;
        Ok(())
    }

    pub fn render(&mut self, scene: &Scene) -> Result<(), String> {
        if let Some(frame) = self.prepare_frame(scene)? {
            self.submit_frame(frame);
        }
        Ok(())
    }

    pub fn prepare_frame(&mut self, scene: &Scene) -> Result<Option<PreparedFrame>, String> {
        if self.size.contains(&0) {
            return Ok(None);
        }
        if self.device_lost.load(Ordering::Acquire) {
            return Err(SURFACE_LOST.into());
        }
        let vertices = tessellate(scene)?;
        let bytes = bytemuck::cast_slice(&vertices);
        let byte_len = bytes.len() as u64;
        let count = u32::try_from(vertices.len()).map_err(|_| "too many vertices")?;
        if byte_len > self.device.limits().max_buffer_size {
            return Err("scene exceeds device buffer limits".into());
        }
        if self.needs_configure {
            self.configure()?;
        }
        let mut acquired = self.surface.get_current_texture();
        if matches!(acquired, wgpu::CurrentSurfaceTexture::Outdated) {
            self.configure()?;
            acquired = self.surface.get_current_texture();
        }
        let frame = match acquired {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                // Reconfigure on the next frame, after this texture has been presented.
                self.needs_configure = true;
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.needs_configure = true;
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err(SURFACE_LOST.into()),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface acquisition validation error".into());
            }
        };
        if byte_len > self.vertex_capacity {
            self.vertex_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("2D vertices"),
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
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("2D shapes"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.025,
                            g: 0.035,
                            b: 0.06,
                            a: 1.0,
                        }),
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
        }
        Ok(Some(PreparedFrame {
            commands: encoder.finish(),
            texture: frame,
        }))
    }

    pub fn submit_frame(&mut self, frame: PreparedFrame) {
        self.queue.submit([frame.commands]);
        self.queue.present(frame.texture);
        self.presented_frames += 1;
    }
}


/// A single pipeline definition shared by standalone and host-owned embedded rendering.
/// Both entrypoints use the same 2D shader and blend behavior.
fn shape_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D shapes"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                })],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
    })
}

fn tessellate(scene: &Scene) -> Result<Vec<Vertex>, String> {
    if scene.size.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return Err("scene size must be finite and positive".into());
    }
    let mut vertices = Vec::new();
    for shape in &scene.shapes {
        if shape
            .bounds
            .iter()
            .chain(&shape.color)
            .any(|v| !v.is_finite())
        {
            return Err("shape contains non-finite geometry or color".into());
        }
        let [x, y, a, b] = shape.bounds.map(f64::from);
        let color = shape.color.map(|v| v.clamp(0.0, 1.0));
        let mut push = |point: [f64; 2]| -> Result<(), String> {
            // f64 intermediates prevent overflow when adding finite f32 bounds.
            let position = [
                (point[0] / f64::from(scene.size[0]) * 2.0 - 1.0) as f32,
                (1.0 - point[1] / f64::from(scene.size[1]) * 2.0) as f32,
            ];
            if position.iter().any(|v| !v.is_finite()) {
                return Err("shape exceeds representable clip coordinates".into());
            }
            vertices.push(Vertex { position, color });
            Ok(())
        };
        match shape.kind {
            ShapeKind::Rectangle => {
                if a < 0.0 || b < 0.0 {
                    return Err("rectangle dimensions must be nonnegative".into());
                }
                if a == 0.0 || b == 0.0 {
                    continue;
                }
                for point in [
                    [x, y],
                    [x + a, y],
                    [x, y + b],
                    [x, y + b],
                    [x + a, y],
                    [x + a, y + b],
                ] {
                    push(point)?;
                }
            }
            ShapeKind::Circle => {
                if a < 0.0 {
                    return Err("circle radius must be nonnegative".into());
                }
                if a == 0.0 {
                    continue;
                }
                for segment in 0..CIRCLE_SEGMENTS {
                    push([x, y])?;
                    for endpoint in [segment, segment + 1] {
                        let angle = (endpoint % CIRCLE_SEGMENTS) as f64 * std::f64::consts::TAU
                            / CIRCLE_SEGMENTS as f64;
                        push([x + a * angle.cos(), y + a * angle.sin()])?;
                    }
                }
            }
        }
    }
    Ok(vertices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::Shape;

    fn scene(kind: ShapeKind, bounds: [f32; 4]) -> Scene {
        Scene {
            size: [200.0, 100.0],
            shapes: vec![Shape {
                kind,
                bounds,
                color: [1.0, 0.5, 0.0, 0.75],
            }],
        }
    }

    #[test]
    fn rectangle_maps_logical_corners_to_clip_space_without_changing_scene() {
        let scene = scene(ShapeKind::Rectangle, [0.0, 0.0, 200.0, 100.0]);
        let original = scene.clone();
        let vertices = tessellate(&scene).unwrap();
        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].position, [-1.0, 1.0]);
        assert_eq!(vertices[1].position, [1.0, 1.0]);
        assert_eq!(vertices[5].position, [1.0, -1.0]);
        assert_eq!(vertices[0].color, scene.shapes[0].color);
        assert_eq!(scene, original);
    }

    #[test]
    fn circle_fan_closes_and_preserves_radius_on_non_square_viewport() {
        let vertices = tessellate(&scene(ShapeKind::Circle, [100.0, 50.0, 25.0, 0.0])).unwrap();
        assert_eq!(vertices.len(), CIRCLE_SEGMENTS * 3);
        assert_eq!(vertices[1].position, vertices.last().unwrap().position);
        for triangle in vertices.as_chunks::<3>().0 {
            assert_eq!(triangle[0].position, [0.0, 0.0]);
            for vertex in &triangle[1..] {
                let x = vertex.position[0] * 100.0;
                let y = vertex.position[1] * 50.0;
                assert!((x.hypot(y) - 25.0).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn degenerate_shapes_are_empty_and_invalid_geometry_is_rejected() {
        for (kind, bounds) in [
            (ShapeKind::Circle, [0.0; 4]),
            (ShapeKind::Rectangle, [1.0, 2.0, 0.0, 4.0]),
        ] {
            assert!(tessellate(&scene(kind, bounds)).unwrap().is_empty());
        }
        for (kind, bounds) in [
            (ShapeKind::Circle, [0.0, 0.0, -1.0, 0.0]),
            (ShapeKind::Rectangle, [0.0, 0.0, 1.0, -1.0]),
            (ShapeKind::Circle, [f32::NAN, 0.0, 1.0, 0.0]),
        ] {
            assert!(tessellate(&scene(kind, bounds)).is_err());
        }
        let mut scene = scene(ShapeKind::Rectangle, [0.0, 0.0, 1.0, 1.0]);
        for size in [[0.0, 1.0], [1.0, f32::INFINITY], [-1.0, 1.0]] {
            scene.size = size;
            assert!(tessellate(&scene).is_err());
        }
    }

    #[test]
    fn large_finite_geometry_never_emits_infinite_vertices() {
        let mut scene = scene(ShapeKind::Rectangle, [f32::MAX, 0.0, f32::MAX, 1.0]);
        scene.size = [f32::MAX, 1.0];
        assert!(
            tessellate(&scene)
                .unwrap()
                .iter()
                .all(|v| v.position.iter().all(|x| x.is_finite()))
        );
        scene.size = [f32::MIN_POSITIVE, 1.0];
        assert!(tessellate(&scene).is_err());
    }
}
