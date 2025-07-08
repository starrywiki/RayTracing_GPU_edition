//main.rs
use {
    anyhow::{Context, Result},
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    wgpu,
};
pub mod render;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct AppState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: render::PathTracer,
}

impl AppState {
    async fn new(window: &Window) -> Result<(Self, wgpu::Surface)> {
        let (device, queue, surface, config) = connect_to_gpu(window).await?;
        let mut renderer = render::PathTracer::new(&device, &queue, config.format, WIDTH, HEIGHT);
        
        let state = Self {
            device,
            queue,
            config,
            renderer,
        };
        
        Ok((state, surface))
    }

    fn resize(&mut self, surface: &wgpu::Surface, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self, surface: &wgpu::Surface) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render_frame(&self.device, &self.queue, &view);
        output.present();
        Ok(())
    }
}
// wgpu::Device : connection to the GPU
// wgpu::Queue : issue commands to the GPU
// wgpu::Surface : present frames to the window.
async fn connect_to_gpu(window: &Window) -> Result<(wgpu::Device, wgpu::Queue, wgpu::Surface, wgpu::SurfaceConfiguration)> {
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window)?; //Surface 是一个用于显示图形的区域，通常与显示设备（如显示器）相关联
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,//偏好选择性能更高的 GPU
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .context("failed to find a compatible adapter")?;
    // 请求 GPU 设备和命令队列
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .context("failed to connect to the GPU")?;
    //  配置 Surface 以进行渲染
    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .into_iter()
        .find(|it| matches!(it, wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm))
        .context("could not find preferred texture format")?;
    // 配置并初始化surface
    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 3,
    };
    surface.configure(&device, &config);
    
    Ok((device, queue, surface, config))
}

#[pollster::main]
async fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window_size = winit::dpi::PhysicalSize::new(WIDTH, HEIGHT);
    let window = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_resizable(false)
        .with_title("GPU Path Tracer".to_string())
        .build(&event_loop)?;

    let (mut state, surface) = AppState::new(&window).await?;

    event_loop.run(|event, control_handle| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_handle.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(&surface, *physical_size);
                }
                WindowEvent::RedrawRequested => {
                    match state.render(&surface) {
                        Ok(_) => {}
                        // 重新配置surface如果过时
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(&surface, window.inner_size())
                        }
                        // 系统内存不足，退出
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("OutOfMemory");
                            control_handle.exit();
                        }
                        // 其他错误
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                // RedrawRequested 只会在手动请求时触发
                // 除非用户请求重绘
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}