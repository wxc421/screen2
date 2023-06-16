use glium::{implement_vertex, Texture2d};
use imgui::*;

mod support;



fn main() {
    let system = support::init(file!());

    let mut value = 0;
    let choices = ["test test this is 1", "test test this is 2"];

    system.main_loop(move |_, ui, display| {
        // // 计算背景矩形的透明度
        // let (width, height) = display.get_framebuffer_dimensions();
        // let bg_alpha = 0.5;
        // ui.window("Hello world")
        //     .title_bar(false)
        //     .resizable(false)
        //     .always_use_window_padding(false)
        //     .no_decoration()
        //     .position([0 as f32, 0 as f32], Condition::Always)
        //     .collapsible(false)
        //     .movable(false)
        //     .scrollable(false)
        //     .bg_alpha(bg_alpha as f32) // 设置窗口背景透明度
        //     .size([width as f32, height as f32], Condition::FirstUseEver)
        //     .build(|| {
        //         let window_pos = ui.window_pos();
        //         let window_size = ui.window_size();
        //
        //         // 将桌面截图渲染到UI上的一个矩形中
        //         ui.image(
        //             screenshot_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        //             [window_size[0], window_size[1]],
        //         );
        //         // ui.text_wrapped("Hello world!");
        //         // if ui.button(choices[value]) {
        //         //     value += 1;
        //         //     value %= 2;
        //         // }
        //         //
        //         // ui.button("This...is...imgui-rs!");
        //         // ui.separator();
        //         // let mouse_pos = ui.io().mouse_pos;
        //         // ui.text(format!(
        //         //     "Mouse Position: ({:.1},{:.1})",
        //         //     mouse_pos[0], mouse_pos[1]
        //         // ));
        //     });
    });
}