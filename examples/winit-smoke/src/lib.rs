use dpi::PhysicalSize;
use pollster::block_on;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor,
    Features, FragmentState, Instance, InstanceDescriptor, LoadOp, MemoryHints, MultisampleState,
    Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp,
    Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, VertexState,
};
use winit_core::application::ApplicationHandler;
use winit_core::event::{StartCause, WindowEvent};
use winit_core::event_loop::ActiveEventLoop as CoreActiveEventLoop;
use winit_core::window::{Window as CoreWindow, WindowAttributes, WindowId};
use winit_ohos::{Window as OhosWindow, export_ohos_winit_app};

const TRIANGLE_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.72),
        vec2<f32>(-0.72, -0.52),
        vec2<f32>(0.72, -0.52),
    );

    let xy = positions[vertex_index];
    return vec4<f32>(xy, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.98, 0.40, 0.19, 1.0);
}
"#;

#[derive(Default)]
struct SmokeApp {
    window: Option<Box<dyn CoreWindow>>,
    renderer: Option<Renderer>,
}

struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: wgpu::Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
}

enum RenderStatus {
    Rendered,
    Skipped,
    OutOfMemory,
}

impl SmokeApp {
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn ensure_window(
        &mut self,
        event_loop: &dyn CoreActiveEventLoop,
    ) -> Result<&dyn CoreWindow, String> {
        if self.window.is_none() {
            let window = event_loop
                .create_window(WindowAttributes::default().with_title("winit-ohos triangle"))
                .map_err(|err| format!("create_window failed: {err}"))?;
            self.window = Some(window);
        }

        Ok(self.window.as_deref().expect("window was just created"))
    }

    fn ensure_renderer(&mut self, event_loop: &dyn CoreActiveEventLoop) -> Result<(), String> {
        if self.renderer.is_some() {
            return Ok(());
        }

        let renderer = {
            let window = self.ensure_window(event_loop)?;
            let backend_window = window
                .cast_ref::<OhosWindow>()
                .ok_or_else(|| String::from("window is not an OHOS backend window"))?;
            let size = window.surface_size();
            block_on(Renderer::new(backend_window, size))?
        };

        eprintln!(
            "winit-smoke renderer ready: {}x{}",
            renderer.config.width, renderer.config.height
        );
        self.renderer = Some(renderer);
        self.request_redraw();
        Ok(())
    }
}

impl ApplicationHandler for SmokeApp {
    fn new_events(&mut self, _event_loop: &dyn CoreActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            eprintln!("winit-smoke booting renderer");
        }
    }

    fn resumed(&mut self, _event_loop: &dyn CoreActiveEventLoop) {
        eprintln!("winit-smoke surface resumed");
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn CoreActiveEventLoop) {
        eprintln!("winit-smoke creating renderer");
        if let Err(err) = self.ensure_renderer(event_loop) {
            eprintln!("winit-smoke renderer init failed: {err}");
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn CoreActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::SurfaceResized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
                self.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(window) = self.window.as_ref() {
                    let size = window.surface_size();
                    if let Some(renderer) = self.renderer.as_mut() {
                        renderer.resize(size.width, size.height);
                    }
                }
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    match renderer.render() {
                        Ok(RenderStatus::Rendered) => eprintln!("winit-smoke triangle rendered"),
                        Ok(RenderStatus::Skipped) => {
                            eprintln!("winit-smoke frame skipped; retrying");
                            self.request_redraw();
                        }
                        Ok(RenderStatus::OutOfMemory) => {
                            eprintln!("winit-smoke wgpu surface ran out of memory");
                            event_loop.exit();
                        }
                        Err(err) => {
                            eprintln!("winit-smoke render failed: {err}");
                            event_loop.exit();
                        }
                    }
                }
            }
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                eprintln!("winit-smoke window closing");
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _event_loop: &dyn CoreActiveEventLoop) {
        eprintln!("winit-smoke surface suspended");
    }

    fn destroy_surfaces(&mut self, _event_loop: &dyn CoreActiveEventLoop) {
        self.renderer = None;
        eprintln!("winit-smoke renderer released");
    }

    fn memory_warning(&mut self, _event_loop: &dyn CoreActiveEventLoop) {
        eprintln!("winit-smoke memory warning");
    }
}

impl Renderer {
    async fn new(window: &OhosWindow, size: PhysicalSize<u32>) -> Result<Self, String> {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let raw_display_handle = window
            .display_handle()
            .map_err(|err| format!("display handle unavailable: {err}"))?
            .as_raw();
        let raw_window_handle = window
            .window_handle()
            .map_err(|err| format!("window handle unavailable: {err}"))?
            .as_raw();

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })
        }
        .map_err(|err| format!("create_surface failed: {err}"))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|err| format!("request_adapter failed: {err}"))?;
        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("winit-smoke device"),
                required_features: Features::empty(),
                required_limits: adapter.limits(),
                memory_hints: MemoryHints::Performance,
                ..Default::default()
            })
            .await
            .map_err(|err| format!("request_device failed: {err}"))?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .or_else(|| capabilities.formats.first().copied())
            .ok_or_else(|| String::from("surface reported no supported formats"))?;
        let alpha_mode = capabilities
            .alpha_modes
            .iter()
            .copied()
            .find(|mode| matches!(mode, CompositeAlphaMode::Opaque))
            .or_else(|| capabilities.alpha_modes.first().copied())
            .ok_or_else(|| String::from("surface reported no alpha modes"))?;

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("triangle shader"),
            source: ShaderSource::Wgsl(TRIANGLE_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("triangle pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("triangle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        eprintln!(
            "winit-smoke wgpu ready: backend={:?}, device={}, format={:?}, size={}x{}",
            adapter_info.backend, adapter_info.name, config.format, config.width, config.height
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.config.width == width && self.config.height == height {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) -> Result<RenderStatus, String> {
        if self.config.width == 0 || self.config.height == 0 {
            return Ok(RenderStatus::Skipped);
        }

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(SurfaceError::Timeout) => return Ok(RenderStatus::Skipped),
            Err(SurfaceError::Outdated | SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(RenderStatus::Skipped);
            }
            Err(SurfaceError::OutOfMemory) => return Ok(RenderStatus::OutOfMemory),
            Err(err) => return Err(format!("surface frame acquisition failed: {err}")),
        };

        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("triangle encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("triangle pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.02,
                            g: 0.36,
                            b: 0.48,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(RenderStatus::Rendered)
    }
}

export_ohos_winit_app!(SmokeApp::default);
