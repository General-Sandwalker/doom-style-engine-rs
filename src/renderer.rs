use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use crate::raycaster::{SCREEN_W, SCREEN_H, cast_rays, compute_column_height, wall_color};
use crate::map::{Map, MAP_WIDTH, MAP_HEIGHT};
use crate::player::Player;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader_src = include_str!("shaders/flat.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self { surface, device, queue, config, pipeline }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&self, player: &Player, map: &Map) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut vertices: Vec<Vertex> = Vec::new();

        build_3d_view(&mut vertices, player, map);
        build_minimap(&mut vertices, player, map);

        let vbuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.02, b: 0.02, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, vbuf.slice(..));
            pass.draw(0..vertices.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn ndc_x(px: f32, sw: f32) -> f32 {
    (px / sw) * 2.0 - 1.0
}

fn ndc_y(py: f32, sh: f32) -> f32 {
    1.0 - (py / sh) * 2.0
}

fn push_quad(verts: &mut Vec<Vertex>, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 4]) {
    let tl = Vertex { position: [x0, y1], color };
    let tr = Vertex { position: [x1, y1], color };
    let bl = Vertex { position: [x0, y0], color };
    let br = Vertex { position: [x1, y0], color };
    verts.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
}

fn build_3d_view(verts: &mut Vec<Vertex>, player: &Player, map: &Map) {
    let sw = SCREEN_W as f32;
    let sh = SCREEN_H as f32;

    let ceiling_color = [0.15, 0.15, 0.25, 1.0f32];
    let floor_color   = [0.25, 0.20, 0.15, 1.0f32];
    push_quad(verts, -1.0, 0.0, 1.0, 1.0, ceiling_color);
    push_quad(verts, -1.0, -1.0, 1.0, 0.0, floor_color);

    let hits = cast_rays(player.x, player.y, player.angle, map);

    for (i, hit) in hits.iter().enumerate() {
        let col_h = compute_column_height(hit.distance) as f32;
        let top    = (sh / 2.0) - (col_h / 2.0);
        let bottom = (sh / 2.0) + (col_h / 2.0);

        let x0 = ndc_x(i as f32, sw);
        let x1 = ndc_x(i as f32 + 1.0, sw);
        let y0 = ndc_y(bottom.min(sh), sh);
        let y1 = ndc_y(top.max(0.0), sh);

        let color = wall_color(&hit.cell, &hit.side);
        push_quad(verts, x0, y0, x1, y1, color);
    }
}

fn build_minimap(verts: &mut Vec<Vertex>, player: &Player, map: &Map) {
    let scale = 0.012f32;
    let ox = -1.0f32;
    let oy = -1.0f32;

    for row in 0..MAP_HEIGHT {
        for col in 0..MAP_WIDTH {
            let color = match map.walls[row][col] {
                crate::map::Cell::Empty    => [0.1, 0.1, 0.1, 0.7],
                crate::map::Cell::Wall(1)  => [0.7, 0.7, 0.7, 0.9],
                crate::map::Cell::Wall(2)  => [0.7, 0.4, 0.2, 0.9],
                crate::map::Cell::Wall(3)  => [0.4, 0.4, 0.7, 0.9],
                crate::map::Cell::Door     => [0.8, 0.7, 0.1, 0.9],
                _                          => [0.5, 0.5, 0.5, 0.9],
            };
            let x0 = ox + col as f32 * scale;
            let y0 = oy + row as f32 * scale;
            let x1 = x0 + scale * 0.95;
            let y1 = y0 + scale * 0.95;
            push_quad(verts, x0, y0, x1, y1, color);
        }
    }

    let px = ox + player.x * scale;
    let py = oy + player.y * scale;
    let ps = scale * 0.4;
    push_quad(verts, px - ps, py - ps, px + ps, py + ps, [1.0, 0.0, 0.0, 1.0]);

    let dir_len = scale * 1.5;
    let ex = px + player.angle.cos() * dir_len;
    let ey = py + player.angle.sin() * dir_len;
    let lw = scale * 0.12;
    push_quad(verts, px - lw, py - lw, ex + lw, ey + lw, [1.0, 1.0, 0.0, 1.0]);
}
