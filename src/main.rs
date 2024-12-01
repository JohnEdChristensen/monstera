use iced::{theme::palette, Theme};
use iced_wgpu::wgpu::util::DeviceExt;
use monstera::bench::Bench;
use monstera::world::World;

mod scene;
use scene::Scene;

use iced_wgpu::graphics::{Antialiasing, Viewport};
use iced_wgpu::{wgpu, Engine, Renderer};
use iced_winit::conversion;
use iced_winit::core::mouse;
use iced_winit::core::renderer;
use iced_winit::core::{Color, Font, Pixels, Size};
use iced_winit::futures;
use iced_winit::runtime::program;
use iced_winit::runtime::Debug;
use iced_winit::winit;
use iced_winit::Clipboard;

use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::ModifiersState,
};

use std::sync::Arc;
use std::time::Instant;

pub fn main() -> Result<(), winit::error::EventLoopError> {
    tracing_subscriber::fmt::init();

    // Initialize winit
    let event_loop = EventLoop::new()?;
    let mut runner = Runner::Loading;

    event_loop.run_app(&mut runner)
}

#[allow(clippy::large_enum_variant)]
enum Runner {
    Loading,
    Ready {
        window: Arc<winit::window::Window>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        screen_dimensions_buffer: wgpu::Buffer,
        format: wgpu::TextureFormat,
        engine: Engine,
        renderer: Renderer,
        scene: Scene,
        state: program::State<World>,
        theme: iced::Theme,
        cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
        clipboard: Clipboard,
        viewport: Viewport,
        modifiers: ModifiersState,
        resized: bool,
        debug: Debug,
        bench: Bench,
    },
}

impl winit::application::ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Loading = self {
            let window = Arc::new(
                event_loop
                    .create_window(winit::window::WindowAttributes::default())
                    .expect("Create window"),
            );

            let physical_size = window.inner_size();
            let viewport = Viewport::with_physical_size(
                Size::new(physical_size.width, physical_size.height),
                window.scale_factor(),
            );
            let clipboard = Clipboard::connect(window.clone());

            let backend = wgpu::util::backend_bits_from_env().unwrap_or_default();

            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: backend,
                ..Default::default()
            });
            let surface = instance
                .create_surface(window.clone())
                .expect("Create window surface");

            let (format, adapter, device, queue) = futures::futures::executor::block_on(async {
                let adapter =
                    wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                        .await
                        .expect("Create adapter");

                let adapter_features = adapter.features();

                let capabilities = surface.get_capabilities(&adapter);

                let (device, queue) = adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            required_features: adapter_features & wgpu::Features::default(),
                            required_limits: wgpu::Limits::default(),
                        },
                        None,
                    )
                    .await
                    .expect("Request device");

                (
                    capabilities
                        .formats
                        .iter()
                        .copied()
                        .find(wgpu::TextureFormat::is_srgb)
                        .or_else(|| capabilities.formats.first().copied())
                        .expect("Get preferred format"),
                    adapter,
                    device,
                    queue,
                )
            });

            surface.configure(
                &device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format,
                    width: physical_size.width,
                    height: physical_size.height,
                    present_mode: wgpu::PresentMode::AutoVsync,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                },
            );

            let screen_dimensions = [1000.0f32, 1000.0f32];
            // Create a buffer for the uniform
            let screen_dimensions_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Screen Dimensions Buffer"),
                    contents: bytemuck::cast_slice(&screen_dimensions),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
            // Create a bind group layout
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("bind_group_layout"),
                });

            // Create a bind group
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: screen_dimensions_buffer.as_entire_binding(),
                }],
                label: Some("bind_group"),
            });

            // Initialize scene and GUI controls
            let scene = Scene::new(&device, format, bind_group, &bind_group_layout);

            //let controls = Controls::new();
            let world = World::default();

            // Initialize iced
            let mut debug = Debug::new();
            let engine = Engine::new(
                &adapter,
                &device,
                &queue,
                format,
                Some(Antialiasing::MSAAx4),
            );
            let mut renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));

            let state = program::State::new(
                //controls,
                world,
                viewport.logical_size(),
                &mut renderer,
                &mut debug,
            );

            let palette = palette::Palette {
                background: [0.1, 0.15, 0.15, 1.0].into(),
                primary: [0.4, 0.7, 0.5, 1.0].into(),
                text: [0.95, 0.9, 0.9, 1.0].into(),
                success: [0.5, 0.6, 0.8, 1.0].into(),
                danger: [0.9, 0.8, 0.6, 1.0].into(),
            };
            let theme = Theme::custom("my_theme".into(), palette);

            // You should change this if you want to render continuously
            event_loop.set_control_flow(ControlFlow::Wait);

            *self = Self::Ready {
                window,
                device,
                queue,
                surface,
                screen_dimensions_buffer,
                format,
                engine,
                renderer,
                scene,
                state,
                theme,
                cursor_position: None,
                modifiers: ModifiersState::default(),
                clipboard,
                viewport,
                resized: false,
                debug,
                bench: Bench::default(),
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Self::Ready {
            window,
            device,
            queue,
            surface,
            screen_dimensions_buffer,
            format,
            engine,
            renderer,
            scene,
            state,
            theme,
            viewport,
            cursor_position,
            modifiers,
            clipboard,
            resized,
            debug,
            bench,
        } = self
        else {
            return;
        };
        let event_start = Instant::now();

        match event {
            WindowEvent::RedrawRequested => {
                if *resized {
                    let size = window.inner_size();

                    *viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor(),
                    );

                    surface.configure(
                        device,
                        &wgpu::SurfaceConfiguration {
                            format: *format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        },
                    );
                    queue.write_buffer(
                        screen_dimensions_buffer,
                        0,
                        bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
                    );
                    *resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let start_present = Instant::now();
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let program = state.program();

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            //TODO: can we only re-render damaged regions?
                            // We clear the frame
                            let mut render_pass =
                                Scene::clear(&view, &mut encoder, program.background_color());

                            // Draw the scene
                            scene.draw(&mut render_pass);
                        }

                        bench.add_present(start_present, Instant::now());
                        //And then iced on top
                        renderer.present(
                            engine,
                            device,
                            queue,
                            &mut encoder,
                            None,
                            frame.texture.format(),
                            &view,
                            viewport,
                            &debug.overlay(),
                        );

                        // Then we submit the work
                        engine.submit(queue, encoder);
                        frame.present();

                        // Update the mouse cursor
                        window.set_cursor(iced_winit::conversion::mouse_interaction(
                            state.mouse_interaction(),
                        ));
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!(
                                "Swapchain error: {error}. \
                            Rendering cannot continue."
                            )
                        }
                        _ => {
                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            //WindowEvent::MouseInput { .. } => {
            //    request_redraw = true;
            //}
            WindowEvent::CursorMoved { position, .. } => {
                *cursor_position = Some(position);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                *modifiers = new_modifiers.state();
            }
            WindowEvent::Resized(_) => {
                *resized = true;
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
        let update_start = Instant::now();

        // Map window event to iced event
        if let Some(event) =
            iced_winit::conversion::window_event(event, window.scale_factor(), *modifiers)
        {
            state.queue_event(event);
        }

        // If there are events pending
        if !state.is_queue_empty() {
            // We update iced
            let _task = state.update(
                viewport.logical_size(),
                cursor_position
                    .map(|p| conversion::cursor_position(p, viewport.scale_factor()))
                    .map(mouse::Cursor::Available)
                    .unwrap_or(mouse::Cursor::Unavailable),
                renderer,
                theme,
                &renderer::Style {
                    text_color: Color::WHITE,
                },
                clipboard,
                debug,
            );

            // and request a redraw
            window.request_redraw();
        }

        bench.add_update(update_start, Instant::now());
        bench.add_total(event_start, Instant::now());
        // println!("{}", bench.summary());
    }
}
