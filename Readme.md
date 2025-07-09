# Week3 Report
## 概述
这周我想实现基于WGPU的GPU实时渲染，本周主要参照RayTracing——GPU edition这本书，完成对第一本书RayTracing in the weekend的GPU版本。
基于RayTracing——GPU edition，主要不同点在于webGPU需要将许多东西在wsgl文件中实现，比如像之前写的结构体Ray、sphere等。更重要的一点是包含了CPU和GPU两端的交互。
## Challenges & Solutions
最开始比较困难的点在于如何正确的打开一个窗口，完全依照C书的实现会导致error，最后我通过刷新窗口并用这个指令编译 `env WINIT_UNIX_BACKEND=x11 WAYLAND_DISPLAY= cargo run` 解决了问题。
特别的一点是我在mian.rs文件中添加了对鼠标移动等按键的捕捉，以此实现在前端通过鼠标对图像放缩、不同角度观察等相关功能。

```rust
// 添加鼠标按钮处理
WindowEvent::MouseInput { state, button, .. } => {
    if *button == winit::event::MouseButton::Left {
         mouse_button_pressed = *state == ElementState::Pressed;
        
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
            let sensitivity = 0.01;
            let du = dx as f32 * sensitivity;
            let dv = dy as f32 * (-sensitivity);  // 翻转Y轴
            camera.pan(du, dv);
            state.renderer.reset_samples();
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
需要注意的是鼠标相关功能按键处理不能完全照着书上的写（我将DeviceEvent::MouseMotion改成了WindowEvent::CursorMoved，将DeviceEvent::Button改成了WindowEvent::MouseInput）因为我发现如果用MouseMotion的话不能正确捕捉我的鼠标移动距离，导致无论往哪个方向图像都只往一个方向移动。改了后运行良好，具体可见附件视频/我的github中videos文件夹中的[004.mp4](https://github.com/starrywiki/RayTracing_GPU_edition/blob/master/videos/004.mp4) 

<video controls width="100%">
  <source src="./videos/004-1.mp4" type="video/mp4">
</video>