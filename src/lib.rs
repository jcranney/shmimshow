use risio::{Accessor, RawImage};
use std::sync::Arc;
use wgpu::util::DeviceExt;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, OwnedDisplayHandle},
    window::{Window, WindowId},
};

struct State {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    rect: Rect,
}

impl State {
    async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display),
        ));
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let device_descriptor = wgpu::DeviceDescriptor::default();
        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];
        let shader = device.create_shader_module(wgpu::include_wgsl!("./bin/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let rect = Rect {
            image: RawImage::create_new("bob", &[3, 5])
                .or_else(|_| RawImage::open("bob"))
                .unwrap(),
        };

        let (vertices, indices) = rect.get_vertices_and_indices(100.0, 100.0);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        let state = State {
            instance,
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            rect,
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    fn render(&mut self) {
        // Create texture view.
        // NOTE: We must handle Timeout because the surface may be unavailable
        // (e.g., when the window is occluded on macOS).
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("No error scope registered, so validation errors will panic")
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.configure_surface();
                return;
            }
        };
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        // let view = self.vertex_buffer.get_mapped_range_mut(0..4).unwrap();
        let (v, _) = self
            .rect
            .get_vertices_and_indices(self.size.width as f32, self.size.height as f32);
        // self.vertices[0].position[0] = self.start_time.elapsed().as_secs_f32().sin();
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&v));
        // self.queue
        //     .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&v));
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 2.
        drop(render_pass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(surface_texture);
    }
}

#[derive(Default)]
pub struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

// lib.rs
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 1],
}

impl Vertex {
    fn desc() -> Option<wgpu::VertexBufferLayout<'static>> {
        Some(wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        })
    }
}

struct Rect {
    image: RawImage<'static, u8>,
}

// let's assume that the image has only 2 dimensions for now.
impl Rect {
    fn get_vertices_and_indices(&self, resx: f32, resy: f32) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = vec![];
        let mut indices: Vec<u16> = vec![];
        let size = self.image.metadata().size;
        let mut i = 0;
        let values = unsafe { self.image.array() };
        let pix_scale_y;
        let pix_scale_x;
        if (resx / resy) > (size[1] as f32 / size[0] as f32) {
            pix_scale_y = 1.75 / size[0] as f32; // y-screen units per image spaxel
            pix_scale_x = pix_scale_y * resy / resx; // x-screen units per image spaxel
        } else {
            pix_scale_x = 1.75 / size[1] as f32; // x-screen units per image spaxel
            pix_scale_y = pix_scale_x * resx / resy; // y-screen units per image spaxel
        }
        for y in 0..size[0] {
            for x in 0..size[1] {
                let color = [values[i] as f32 / 255.0];
                let mut this_pixel_vertices = vec![
                    Vertex {
                        position: [
                            (x as f32 - (size[1] as f32) * 0.5 + 0.5) * pix_scale_x
                                - 0.5 * pix_scale_x,
                            (y as f32 - (size[0] as f32) * 0.5 + 0.5) * pix_scale_y
                                - 0.5 * pix_scale_y,
                        ],
                        color,
                    },
                    Vertex {
                        position: [
                            (x as f32 - (size[1] as f32) * 0.5 + 0.5) * pix_scale_x
                                - 0.5 * pix_scale_x,
                            (y as f32 - (size[0] as f32) * 0.5 + 0.5) * pix_scale_y
                                + 0.5 * pix_scale_y,
                        ],
                        color,
                    },
                    Vertex {
                        position: [
                            (x as f32 - (size[1] as f32) * 0.5 + 0.5) * pix_scale_x
                                + 0.5 * pix_scale_x,
                            (y as f32 - (size[0] as f32) * 0.5 + 0.5) * pix_scale_y
                                - 0.5 * pix_scale_y,
                        ],
                        color,
                    },
                    Vertex {
                        position: [
                            (x as f32 - (size[1] as f32) * 0.5 + 0.5) * pix_scale_x
                                + 0.5 * pix_scale_x,
                            (y as f32 - (size[0] as f32) * 0.5 + 0.5) * pix_scale_y
                                + 0.5 * pix_scale_y,
                        ],
                        color,
                    },
                ];
                let offset = vertices.len() as u16;
                indices.append(&mut vec![
                    offset,
                    offset + 2,
                    offset + 1,
                    offset + 1,
                    offset + 2,
                    offset + 3,
                ]);
                vertices.append(&mut this_pixel_vertices);
                i += 1;
            }
        }
        (vertices, indices)
    }
}
