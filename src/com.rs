use {
    anyhow::{Context, Result}, // used for handling errors
    winit::{
        // initialize window
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    wgpu,
};
pub mod render;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

// wgpu::Device : connection to the GPU
// wgpu::Queue : issue commands to the GPU
// wgpu::Surface : present frames to the window.
async fn connect_to_gpu(window: &Window) ->Result<(wgpu::Device, wgpu::Queue, wgpu::Surface)>{
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window)?; //Surface 是一个用于显示图形的区域，通常与显示设备（如显示器）相关联
    let adapter = instance
                        .request_adapter(&wgpu::RequestAdapterOptions {
                            power_preference: wgpu::PowerPreference::HighPerformance, //偏好选择性能更高的 GPU
                            force_fallback_adapter: false,
                            compatible_surface: Some(&surface),
                        })
                        .await
                        .context("failed to find a compatible adapter")?;
    //请求 GPU 设备和命令队列
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
                    .context("could not find preferred texture format (Rgba8Unorm or Bgra8Unorm)")?;
    //配置并初始化surface
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
    Ok((device, queue, surface))
}

#[pollster::main]
async fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window_size = winit::dpi::PhysicalSize::new(WIDTH, HEIGHT);
    let window = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_resizable(false)
        .with_title("GPU Path Tracer".to_string()) //set title
        .build(&event_loop)?;
    let (device,queue,surface) = connect_to_gpu(&window).await?;
    let renderer = render::PathTracer::new(device,queue);

    window.request_redraw();

    event_loop.run(|event, control_handle| {
        //start loop
        control_handle.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_handle.exit(),
                WindowEvent::RedrawRequested => {
                    // Wait for the next available frame buffer.
                    let frame : wgpu::SurfaceTexture = surface
                        .get_current_texture()
                        .expect("failed to get current texture");

                    // let render_target = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    // renderer.render_frame(&render_target);
                    frame.present();
                    window.request_redraw();
                }
                // Event::AboutToWait => {
                //     // 在事件处理完毕后请求重绘
                //     window.request_redraw();
                // }
                _ => (),
            },
            _ => (),
        }
    })?;
    Ok(())
}