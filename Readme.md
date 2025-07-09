# Readme（Week3 Report）
[仓库地址](https://github.com/starrywiki/RayTracing_GPU_edition)
## 概述
本周我计划实现一个基于 WGPU 的 GPU 实时渲染器，主要参考《Ray Tracing — GPU Edition》一书，将《Ray Tracing in One Weekend》的内容迁移并实现为 GPU 版本。

相较于原书中基于 CPU 的实现，WGPU 的开发模式要求我们将许多关键结构（如 Ray、Sphere 等）定义在 WGSL 着色器语言中。此外，更关键的区别在于：WGPU 涉及 CPU 与 GPU 之间的资源交互与同步，这对资源绑定（Bind Groups）、Uniform/Storage Buffer 的布局等提出了新的要求。因此，本周的主要目标是完成场景数据结构的迁移、光线路径追踪的基本流程，以及 CPU–GPU 通信框架的初步搭建。
## 成果展示
我减少了球的总数目实现了**week1_finalscene**的场景 具体可见[视频](https://github.com/starrywiki/RayTracing_GPU_edition/blob/master/week1_scene.mp4)
同时展现我在实现Raytraing GPU Edition这本书的一些效果图

<img src="https://notes.sjtu.edu.cn/uploads/upload_58278c6be3da4933cbfbeae0dd0f414a.png" alt="描述" width="300px" height=auto /><img src="https://notes.sjtu.edu.cn/uploads/upload_fc3f7c0960b29fc6b578581077314c9c.png" alt="描述" width="300px" height=auto /><img src="https://notes.sjtu.edu.cn/uploads/upload_2a6f40d33f7e8a603e6353140b42ef7a.png" alt="描述" width="300px" height=auto /><img src="https://notes.sjtu.edu.cn/uploads/upload_a127753a450284aeea039877fa5f0a04.png" alt="描述" width="300px" height=auto /><img src="https://notes.sjtu.edu.cn/uploads/upload_a8841bb9628a9c3103cd6e0a3bfe1662.png" alt="描述" width="300px" height=auto /><img src="https://notes.sjtu.edu.cn/uploads/upload_2a451477f96f5ee13d1ac9f94d23998f.png" alt="描述" width="300px" height=auto /><img src="https://notes.sjtu.edu.cn/uploads/upload_2fbb5a4fb2207980f716d51bd2e98389.png" alt="描述" width="300px" height=auto /><img src="https://notes.sjtu.edu.cn/uploads/upload_be993f65481fdef4454e28737904dad9.png" alt="描述" width="300px" height=auto />


## Challenges & Solutions
### 环境问题
最开始比较困难的点在于如何正确的打开一个窗口，完全依照书的实现会遇到类似重新配置surface过时的error，因为 surface 的配置可能在某些窗口系统（特别是 X11 与 Wayland）下发生失效。
为了解决这个问题，我在 match 渲染错误中加入了自动重配置逻辑：
```rust
Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
    state.resize(&surface, window.inner_size())
}
```
此外，在 Linux 环境中，还需要显式指定 X11 后端以避免兼容性问题： `env WINIT_UNIX_BACKEND=x11 WAYLAND_DISPLAY= cargo run` 编译解决了问题。
### 前端鼠标交互
我在main.rs中添加了对鼠标输入事件的监听逻辑

**实现要点**：
- 鼠标左键 + 拖动：轨道旋转
- 鼠标右键 + 拖动：相机平移
- 鼠标滚轮：缩放

```rust
// 添加鼠标按钮处理
WindowEvent::MouseInput { state, button, .. } => {
    mouse_button_pressed = *state == ElementState::Pressed;
    match button {
        winit::event::MouseButton::Left => left_mouse_button_pressed = mouse_button_pressed,
        winit::event::MouseButton::Right => right_mouse_button_pressed = mouse_button_pressed,
        _ => (),
    }
    if !mouse_button_pressed{
        last_mouse_pos = None;
    }
}

// 添加鼠标移动处理
WindowEvent::CursorMoved { position, .. } => {
    if mouse_button_pressed {
    if let Some(last_pos) = last_mouse_pos {
        let dx = position.x - last_pos.x;
        let dy = position.y - last_pos.y;
        let sensitivity = 0.01;
        let du = dx as f32 * sensitivity;
        let dv = dy as f32 * (-sensitivity);  // 翻转Y轴
        if left_mouse_button_pressed {
            camera.orbit(du,dv);
            state.renderer.reset_samples();
        }
        if right_mouse_button_pressed {
            camera.pan(du,dv);
            state.renderer.reset_samples();
        }
    }
    last_mouse_pos = Some(*position);
    }
}
                
// 鼠标滚轮处理
DeviceEvent::MouseWheel { delta } => {
    let delta = match delta {
    MouseScrollDelta::PixelDelta(delta) => 0.001 * delta.y as f32,
    MouseScrollDelta::LineDelta(_, y) => y * 0.1,
    };
    camera.zoom(delta);
    state.renderer.reset_samples();
}
```
需要注意的是鼠标相关功能按键处理不能完全照着书上的写
我最初尝试使用 `DeviceEvent::MouseMotion` 和 `DeviceEvent::Button`，但它们无法准确捕捉相对位置变化，导致图像始终只向一个方向移动。
最终改用 `WindowEvent::CursorMoved` 与 `WindowEvent::MouseInput` 处理相对位置和状态，效果稳定、交互自然。
具体效果可见附件视频/我的github中videos文件夹中的[004.mp4](https://github.com/starrywiki/RayTracing_GPU_edition/blob/master/videos/004.mp4) 
