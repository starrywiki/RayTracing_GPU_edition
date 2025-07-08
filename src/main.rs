//main.rs
use {
    anyhow::{Context, Result},
    winit::{
        event::{DeviceEvent, Event, MouseScrollDelta, WindowEvent,ElementState},       
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    wgpu,
};
use crate::{algebra::Vec3, camera::Camera};

pub mod render;
pub mod algebra;
pub mod camera;
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

    fn render(&mut self, surface: &wgpu::Surface, camera: &Camera) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render_frame(&camera, &self.device, &self.queue, &view);
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

    let mut mouse_button_pressed = false;
    let mut last_mouse_pos: Option<winit::dpi::PhysicalPosition<f64>> = None; 

    let mut camera = Camera::look_at(
        Vec3::new(0., 0.75, 1.),
        Vec3::new(0., -0.5, -1.),
        Vec3::new(0., 1., 0.),
    );
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
                    match state.render(&surface, &camera) {
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
                    window.request_redraw();
                }
                // 添加鼠标按钮处理
                WindowEvent::MouseInput { state, button, .. } => {
                    if *button == winit::event::MouseButton::Left {
                        mouse_button_pressed = *state == ElementState::Pressed;
                        // if mouse_button_pressed {
                        //     println!("鼠标按钮按下");
                        // } else {
                        //     println!("鼠标按钮释放");
                        //     last_mouse_pos = None;  // 重置鼠标位置
                        // }
                        if !mouse_button_pressed{
                            last_mouse_pos = None;
                        }
                    }
                }
                // 添加鼠标移动处理
                WindowEvent::CursorMoved { position, .. } => {
                    if mouse_button_pressed {
                        if let Some(last_pos) = last_mouse_pos {
                            let dx = position.x - last_pos.x;
                            let dy = position.y - last_pos.y;
                            
                            // println!("=== 鼠标移动 ===");
                            // println!("当前位置: ({}, {})", position.x, position.y);
                            // println!("上次位置: ({}, {})", last_pos.x, last_pos.y);
                            // println!("增量: dx={}, dy={}", dx, dy);
                            
                            let sensitivity = 0.01;
                            let du = dx as f32 * sensitivity;
                            let dv = dy as f32 * (-sensitivity);  // 翻转Y轴
                            
                            // println!("计算的du={}, dv={}", du, dv);
                            
                            camera.pan(du, dv);
                            state.renderer.reset_samples();
                        }
                        last_mouse_pos = Some(*position);
                    }
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseWheel { delta } => {
                    let delta = match delta {
                        MouseScrollDelta::PixelDelta(delta) => 0.001 * delta.y as f32,
                        MouseScrollDelta::LineDelta(_, y) => y * 0.1,
                    };
                    camera.zoom(delta);
                    state.renderer.reset_samples();
                }
                // DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                //     if mouse_button_pressed {
                //         camera.pan(dx as f32 * 0.00001, dy as f32 * (-0.00001));
                //         state.renderer.reset_samples();
                //     }
                // }
                // DeviceEvent::Button { state, .. } => {
                //     // NOTE: If multiple mouse buttons are pressed, releasing any of them will
                //     // set this to false.
                //     mouse_button_pressed = state == ElementState::Pressed;
                // }
                _ => (),
            },
            // Event::AboutToWait => {
            //     // RedrawRequested 只会在手动请求时触发
            //     // 除非用户请求重绘
            //     window.request_redraw();
            // }
            _ => {},
        }
        // window.request_redraw();

    })?;
    
    Ok(())
}