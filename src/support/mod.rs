mod clipboard;

use std::borrow::Cow;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use glium::{glutin, implement_vertex, Texture2d};
use glium::{Display, Surface};
use imgui::{Condition, Context, FontConfig, FontGlyphRanges, FontSource, Image, ImColor32, MouseButton, StyleColor, Ui};
use imgui_glium_renderer::{Renderer, Texture};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use glium::texture::TextureCreationError;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior, SamplerWrapFunction};
use image::DynamicImage;
use imgui::sys::{igBullet};
use imgui_winit_support::winit::dpi::PhysicalPosition;
use imgui_winit_support::winit::event::{Event, VirtualKeyCode, WindowEvent};
use imgui_winit_support::winit::event_loop::{ControlFlow, EventLoop};
use imgui_winit_support::winit::platform::windows::WindowBuilderExtWindows;
use imgui_winit_support::winit::window::{CursorIcon, Fullscreen, WindowBuilder, WindowId};
use winapi::um::wingdi;
use winit::dpi::LogicalPosition;
use crate::util;


pub struct System {
    pub event_loop: EventLoop<()>,
    pub display: glium::Display,
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
}

// 定义顶点数据结构
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

fn capture_screenshot(display: &glium::Display) -> Result<glium::texture::Texture2d, glium::texture::TextureCreationError> {
    let screen_size = display.get_framebuffer_dimensions();

    // 获取桌面截图的像素数据
    let screenshot = get_screenshot(screen_size.0 as u32, screen_size.1 as u32);
    // 创建纹理
    let binding = screenshot.unwrap();
    let raw_image = glium::texture::RawImage2d {
        data: Cow::Borrowed(binding.as_ref()),
        width: screen_size.0 as u32,
        height: screen_size.1 as u32,
        format: glium::texture::ClientFormat::U8U8U8,
    };

    let texture = glium::texture::Texture2d::new(display, raw_image)?;
    Ok(texture)
}

fn euclidean_distance(a: PhysicalPosition<f64>, b: PhysicalPosition<f64>) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    f64::sqrt(dx * dx + dy * dy)
}

#[cfg(target_os = "windows")]
pub fn get_screenshot(width: u32, height: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::mem;
    use winapi::um::winuser;

    let hwnd = unsafe { winuser::GetDesktopWindow() };
    let hdc_source = unsafe { winuser::GetDC(hwnd) };
    let hdc_dest = unsafe { wingdi::CreateCompatibleDC(hdc_source) };
    let mut pixels = vec![0; (width * height * 3) as usize];

    let mut bitmap_info = wingdi::BITMAPINFO {
        bmiHeader: wingdi::BITMAPINFOHEADER {
            biSize: std::mem::size_of::<wingdi::BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: height as i32, // 负数表示从顶部到底部扫描
            biPlanes: 1,
            biBitCount: 24, // 每个像素占 24 位，每个通道占 8 位
            biCompression: wingdi::BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [wingdi::RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }],
    };

    let bitmap =
        unsafe { wingdi::CreateCompatibleBitmap(hdc_source, width as i32, height as i32) };
    unsafe {
        wingdi::SelectObject(hdc_dest, bitmap as winapi::shared::windef::HGDIOBJ);
        wingdi::BitBlt(
            hdc_dest,
            0,
            0,
            width as i32,
            height as i32,
            hdc_source,
            0,
            0,
            winapi::um::wingdi::SRCCOPY,
        );

        wingdi::GetDIBits(
            hdc_dest,
            bitmap,
            0,
            height as u32,
            pixels.as_mut_ptr() as *mut _,
            &mut bitmap_info as *mut _ as *mut wingdi::BITMAPINFO,
            wingdi::DIB_RGB_COLORS,
        );
        wingdi::DeleteDC(hdc_dest);
        winuser::ReleaseDC(hwnd, hdc_source);
        wingdi::DeleteObject(bitmap as winapi::shared::windef::HGDIOBJ);
    }

    Ok(pixels)
}

struct DrawPoint {
    position: (f32, f32),
    color: [f32; 4],
}

struct DrawState {
    is_drawing: bool,
    points: Vec<DrawPoint>,
    color: [f32; 4],
}

pub fn init(title: &str) -> System {
    let title = match Path::new(&title).file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => title,
    };
    let event_loop = EventLoop::new();
    /*
        如果你已经开启了垂直同步（VSync），它将根据显示器的刷新率来限制帧速率。
        在这种情况下，通常不需要额外控制帧率，因为 VSync 会自动将帧速率与显示器的刷新率同步，避免出现撕裂和过度消耗资源的情况。
        VSync 的目的是为了在图像渲染时消除撕裂效应，并提供平滑的视觉体验。
        开启 VSync 可以让图像与显示器的刷新率同步，每秒刷新次数不会超过显示器的最大刷新率（通常为 60Hz 或 120Hz）。
     */
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = WindowBuilder::new()
        .with_title(title.to_owned())
        .with_visible(false)
        .with_position(glutin::dpi::LogicalPosition::new(0, 0))
        .with_decorations(false)
        .with_undecorated_shadow(false)
        // .with_max_inner_size(glutin::dpi::LogicalSize::new(524f64, 468f64))
        // .with_min_inner_size(glutin::dpi::LogicalSize::new(524f64, 468f64))
        .with_fullscreen(Some(Fullscreen::Borderless(None))); // 全屏窗口
    // .with_inner_size(glutin::dpi::LogicalSize::new(524f64, 468f64));
    let display =
        Display::new(builder, context, &event_loop).expect("Failed to initialize display");

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    if let Some(backend) = clipboard::init() {
        imgui.set_clipboard_backend(backend);
    } else {
        eprintln!("Failed to initialize clipboard");
    }

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();

        let dpi_mode = if let Ok(factor) = std::env::var("IMGUI_EXAMPLE_FORCE_DPI_FACTOR") {
            // Allow forcing of HiDPI factor for debugging purposes
            match factor.parse::<f64>() {
                Ok(f) => HiDpiMode::Locked(f),
                Err(e) => panic!("Invalid scaling factor: {}", e),
            }
        } else {
            HiDpiMode::Default
        };

        platform.attach_window(imgui.io_mut(), window, dpi_mode);
    }

    // Fixed font size. Note imgui_winit_support uses "logical
    // pixels", which are physical pixels scaled by the devices
    // scaling factor. Meaning, 13.0 pixels should look the same size
    // on two different screens, and thus we do not need to scale this
    // value (as the scaling is handled by winit)
    let font_size = 13.0;

    // imgui.fonts().add_font(&[
    //     FontSource::TtfData {
    //         data: include_bytes!("../../../resources/Roboto-Regular.ttf"),
    //         size_pixels: font_size,
    //         config: Some(FontConfig {
    //             // As imgui-glium-renderer isn't gamma-correct with
    //             // it's font rendering, we apply an arbitrary
    //             // multiplier to make the font a bit "heavier". With
    //             // default imgui-glow-renderer this is unnecessary.
    //             rasterizer_multiply: 1.5,
    //             // Oversampling font helps improve text rendering at
    //             // expense of larger font atlas texture.
    //             oversample_h: 4,
    //             oversample_v: 4,
    //             ..FontConfig::default()
    //         }),
    //     },
    //     FontSource::TtfData {
    //         data: include_bytes!("../../../resources/mplus-1p-regular.ttf"),
    //         size_pixels: font_size,
    //         config: Some(FontConfig {
    //             // Oversampling font helps improve text rendering at
    //             // expense of larger font atlas texture.
    //             oversample_h: 4,
    //             oversample_v: 4,
    //             // Range of glyphs to rasterize
    //             glyph_ranges: FontGlyphRanges::japanese(),
    //             ..FontConfig::default()
    //         }),
    //     },
    // ]);

    // 设置 ImGui 样式
    let mut style = imgui.style_mut();
    style.child_border_size = 0.0;

    // 修改按钮的默认颜色
    style.colors[StyleColor::Button as usize] = imgui::ImColor32::from_rgb(255, 255, 255).to_rgba_f32s();


    // style[ImGuiStyleVar::ButtonHovered] = *(&[1.0, 0.5, 0.0, 1.0] as *const [f32; 4]
    //     as *const [f32; 4]); // 将悬停时的颜色设置为橙色
    let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    System {
        event_loop,
        display,
        imgui,
        platform,
        renderer,
        font_size,
    }
}

pub struct UiInfo {
    copy_to_clipboard_texture_id: imgui::TextureId,
    background_texture_id: imgui::TextureId,
}

// 异步加载纹理的函数
// fn load_texture_async(display: Arc<Display>) -> Receiver<Texture2d> {
//     let (tx, rx) = channel();
//     thread::spawn(move || {
//         // 异步加载纹理的逻辑
//         // 例如，从文件系统或网络中加载纹理
//
//         // 在这里将纹理数据传递给主线程
//         let texture2d = util::svg::load_data_to_2d(include_bytes!("../../fuzhidaojiantieban.png"), &*display);
//         tx.send(texture2d).unwrap();
//     });
//
//     rx
// }

impl System {
    pub fn main_loop<F: FnMut(&mut bool, &mut Ui, &Display, &UiInfo) + 'static>(self, mut run_ui: F) {
        let System {
            event_loop,
            display,
            mut imgui,
            mut platform,
            mut renderer,
            ..
        } = self;

        // imgui.io_mut().mouse_draw_cursor = false;
        // imgui.io_mut().config_flags |= imgui::ConfigFlags::NO_MOUSE_CURSOR_CHANGE;

        imgui.style_mut().frame_border_size = 0.0;
        imgui.style_mut().frame_padding = [0.0, 0.0];
        imgui.style_mut().item_inner_spacing = [0.0, 0.0];

        let mut last_frame = Instant::now();

        // 创建一个顶点缓冲对象（VBO）
        let vertex_buffer = {
            #[rustfmt::skip]
                let vertex_data: [Vertex; 4] = [
                Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] },
                Vertex { position: [-1.0, 1.0], tex_coords: [0.0, 0.0] },
                Vertex { position: [1.0, -1.0], tex_coords: [1.0, 1.0] },
                Vertex { position: [1.0, 1.0], tex_coords: [1.0, 0.0] },
            ];

            glium::VertexBuffer::new(&display, &vertex_data).unwrap()
        };

        // 创建一个索引缓冲对象（IBO）
        let index_buffer = glium::IndexBuffer::new(
            &display,
            glium::index::PrimitiveType::TriangleStrip,
            &[0 as u16, 1, 2, 3],
        )
            .unwrap();

        let screenshot = capture_screenshot(&display);
        let my_texture_id = renderer.textures().insert(Texture {
            texture: Rc::new(screenshot.unwrap()),
            sampler: SamplerBehavior {
                // minify_filter: MinifySamplerFilter::NearestMipmapLinear,
                // magnify_filter: MagnifySamplerFilter::Nearest,
                // wrap_function: (
                //     SamplerWrapFunction::BorderClamp,
                //     SamplerWrapFunction::BorderClamp,
                //     SamplerWrapFunction::BorderClamp,
                // ),
                ..Default::default()
            },
        });

        // 加载图标纹理...
        let icon_texture_data = include_bytes!("../../haha.png");
        let icon_texture = glium::texture::RawImage2d::from_raw_rgba_reversed(icon_texture_data, (16, 16));
        // let icon_texture = glium::texture::Texture2d::new(&display, icon_texture).unwrap();
        // let icon_texture_id = renderer.textures().insert(Texture {
        //     texture: Rc::new(icon_texture),
        //     sampler: SamplerBehavior {
        //         // minify_filter: MinifySamplerFilter::NearestMipmapLinear,
        //         // magnify_filter: MagnifySamplerFilter::Nearest,
        //         // wrap_function: (
        //         //     SamplerWrapFunction::BorderClamp,
        //         //     SamplerWrapFunction::BorderClamp,
        //         //     SamplerWrapFunction::BorderClamp,
        //         // ),
        //         ..Default::default()
        //     },
        // });

        // imgui.io_mut().mouse_draw_cursor = false;


        // 加载纹理
        // let facade: &dyn glium::backend::Facade = &display;
        // let x = Box::new(facade);
        // let texture2d = util::svg::load_data_to_2d(include_bytes!("../../haha.png"), Box::new(facade));

        println!("srart load_data_to_2d");

        let texture2d = util::svg::load_data_to_2d(include_bytes!("../../fuzhidaojiantieban.png"), &display);
        let copy_to_clipboard_texture_id = renderer.textures().insert(Texture {
            texture: Rc::new((texture2d)),
            sampler: Default::default(),
        });


        let texture2d = util::svg::capture(&display);
        let background_texture_id = renderer.textures().insert(Texture {
            texture: Rc::new((texture2d)),
            sampler: Default::default(),
        });

        let ui_info = UiInfo {
            copy_to_clipboard_texture_id,
            background_texture_id,
        };

        println!("will into event_loop");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            // *control_flow = ControlFlow::Wait;
            println!("Event:{:?}", event);
            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                }
                Event::MainEventsCleared => {
                    let gl_window = display.gl_window();
                    platform
                        .prepare_frame(imgui.io_mut(), gl_window.window())
                        .expect("Failed to prepare frame");
                    gl_window.window().request_redraw();
                    println!("=======================");
                    let ui = imgui.frame();

                    let mut run = true;
                    run_ui(&mut run, ui, &display, &ui_info);
                    if !run {
                        *control_flow = ControlFlow::Exit;
                    }

                    let gl_window = display.gl_window();
                    let mut target = display.draw();
                    target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                    platform.prepare_render(ui, gl_window.window());
                    let draw_data = imgui.render();
                    renderer
                        .render(&mut target, draw_data)
                        .expect("Rendering failed");
                    target.finish().expect("Failed to swap buffers");
                    display.gl_window().window().set_visible(true);
                    // display.gl_window().window().set_visible(true);
                }
                Event::RedrawRequested(_) => {
                    return;
                    println!("=======================");
                    let ui = imgui.frame();

                    let mut run = true;
                    run_ui(&mut run, ui, &display, &ui_info);
                    if !run {
                        *control_flow = ControlFlow::Exit;
                    }

                    let gl_window = display.gl_window();
                    let mut target = display.draw();
                    target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                    platform.prepare_render(ui, gl_window.window());
                    let draw_data = imgui.render();
                    renderer
                        .render(&mut target, draw_data)
                        .expect("Rendering failed");
                    display.gl_window().window().set_visible(true);
                    target.finish().expect("Failed to swap buffers");
                }
                Event::WindowEvent {
                    event,
                    window_id
                } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        // 处理按键事件，可以在这里添加自定义逻辑
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        let gl_window = display.gl_window();
                        let window = gl_window.window();
                        println!("Mouse position: ({}, {})", position.x, position.y);
                        // todo 研究一下
                        let scale = window.scale_factor(); // 2
                        println!("window.scale_factor():{}", scale);
                        let position = position.to_logical(scale);
                        println!("position.to_logical(scale):{:?}", position);
                        let position = platform.scale_pos_from_winit(window, position);
                        println!("platform.scale_pos_from_winit(window, position):{:?}", position);
                        // io.add_mouse_pos_event([position.x as f32, position.y as f32]);
                        imgui.io_mut().add_mouse_pos_event([position.x as f32, position.y as f32]);
                        // if draw_state.is_drawing {
                        //     // let pos = position.to_logical(display.gl_window().window().get_hidpi_factor());
                        //
                        //     draw_state.points.push(DrawPoint {
                        //         position: (position.x as f32, position.y as f32),
                        //         color: draw_state.color,
                        //     });
                        // }
                        // display.gl_window().window().request_redraw();
                    }
                    event => {
                        let event1: Event<WindowEvent> = Event::WindowEvent {
                            window_id,
                            event,

                        };
                        let gl_window = display.gl_window();
                        platform.handle_event(imgui.io_mut(), gl_window.window(), &event1);
                    }
                }
                event => {
                    let gl_window = display.gl_window();
                    platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
                }
            }
        })
    }
}

// 判断鼠标位置是否在矩形框内
pub fn is_mouse_in_rect(mouse_x: f32, mouse_y: f32, rect_x: f32, rect_y: f32, rect_width: f32, rect_height: f32) -> bool {
    mouse_x >= rect_x && mouse_x <= rect_x + rect_width && mouse_y >= rect_y && mouse_y <= rect_y + rect_height
}


pub fn run() {
    let system = init(file!());

    let mut value = 0;
    let choices = ["test test this is 1", "test test this is 2"];


    // 定义截图范围的变量
    let mut screenshot_area = (0.0, 0.0, 0.0, 0.0);
    let mut is_selecting = false;
    let mut start_pos = [0.0, 0.0];
    let mut end_pos = [0.0, 0.0];
    let mut has_selection = false;
    // 定义拉伸点的大小
    let resize_handle_radius = 6.0;
    let select_paint = false;

    let mut draw_state = DrawState {
        is_drawing: false,
        points: Vec::new(),
        color: [1.0, 0.0, 0.0, 1.0],
    };

    #[derive(Copy, Clone)]
    enum ResizeHandle {
        TopLeft,
        TopRight,
        BottomLeft,
        BottomRight,
        Inner,
        Outer,
    }

    let mut active_resize_handle: Rc<RefCell<Option<ResizeHandle>>> = Rc::new(RefCell::new(None));


    system.main_loop(move |_,
                           mut ui,
                           display,
                           ui_info,
    | {
        ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.0, 0.0]));
        ui.push_style_var(imgui::StyleVar::ItemSpacing([0.0, 0.0]));
        ui.push_style_var(imgui::StyleVar::FrameBorderSize(0.0));
        ui.push_style_var(imgui::StyleVar::FramePadding([0.0, 0.0]));


        // 这里获得是物理像素值: 2880x1800 我的电脑缩放是200%，因此需要/2
        let (width, height) = display.get_framebuffer_dimensions();

        let scale_factor = display.gl_window().window().scale_factor();

        let logical_width = (width as f64 / scale_factor) as u32;
        let logical_height = (height as f64 / scale_factor) as u32;

        ui.window("Hello world")
            .title_bar(true)
            .resizable(false)
            // .always_use_window_padding(false)
            .no_decoration()
            .position([0 as f32, 0 as f32], Condition::Always)
            .collapsible(false)
            .always_use_window_padding(false)
            .content_size([logical_width as f32, logical_height as f32])
            // .content_size([width as f32, height as f32])
            .movable(false)
            .scrollable(false)
            // .bg_alpha(bg_alpha as f32) // 设置窗口背景透明度
            .size([logical_width as f32, logical_height as f32], Condition::FirstUseEver)
            // .size([width as f32, height as f32], Condition::FirstUseEver)
            .build(|| {
                // ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.0, 0.0, 1.0]); // 设置文本颜色为红色
                // const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
                /*
                    push_style_color 和 push_style_var 都是 ImGui 中用于在绘制过程中临时修改样式的函数。
                    push_style_color 函数用于临时修改颜色样式。它允许你指定一个索引，例如 StyleColor::Text，以及对应的颜色值，然后在调用 pop_style_color 函数之前，ImGui 将使用新的颜色值进行渲染。
                    push_style_var 函数用于临时修改通用样式变量。
                    它允许你指定一个样式变量，例如按钮间距 (StyleVar::ButtonSpacing)，以及对应的新值。
                    在调用 pop_style_var 函数之前，ImGui 将使用新的样式变量值进行渲染。
                 */
                // let color = ui.push_style_color(imgui::StyleColor::Text, RED);
                // 没用 使用 None 来临时重置按钮间距
                ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.0, 0.0]));
                ui.push_style_var(imgui::StyleVar::ItemSpacing([0.0, 0.0]));
                ui.push_style_var(imgui::StyleVar::FramePadding([0.0, 0.0]));
                ui.push_style_var(imgui::StyleVar::FrameBorderSize(0.0));

                // ui.text("I'm red!");
                // color.pop();
                // // 在同一行放置两个按钮
                // ui.button("Button 1");
                // ui.same_line_with_spacing(0.0, 0.0);
                // ui.button("Button 2");
                // let window_pos = ui.window_pos();
                let window_size = ui.window_size();
                println!("window_size:{:?}", window_size);
                // // ui.io().mouse_draw_cursor = true;
                // ui.text_wrapped("Hello world!");

                // 没用
                // ui.push_style_var(imgui::StyleVar::FramePadding([0.0, 0.0]));
                // ui.push_style_var(imgui::StyleVar::FrameBorderSize(0.0));

                // ui.set_mouse_cursor(Some(imgui::MouseCursor::Hand));
                // println!("draw_list ...");
                // // 绘制一个矩形
                let draw_list = ui.get_window_draw_list();
                //
                // println!("ui_info.background_texture_id:{:?}", ui_info.background_texture_id);
                draw_list
                    .add_image(
                        ui_info.background_texture_id,
                        [0.0, 0.0],
                        // [window_size[0] as f32, window_size[1] as f32],
                        [window_size[0] as f32, window_size[1] as f32],
                    )
                    .col([1.0, 1.0, 1.0, 0.6])
                    .build();
                //
                // draw_list.add_rect(
                //     [20 as f32, 20 as f32],
                //     [100 as f32, 100 as f32],
                //     [1.0, 0.0, 0.0, 1.0],
                // )
                //     .filled(false)
                //     .thickness(2.0)
                //     .build();
                //
                // draw_list
                //     .add_line([10.0 - 5.0, 20.0],
                //               [10.0 + 5.0, 20.0], [1.0, 0.0, 0.0, 1.0]).thickness(3.0).build();
                let cursor_pos = ui.io().mouse_pos;
                let mouse_pos = cursor_pos;
                // println!("====================cursor_pos:{:?}", cursor_pos);
                // println!("====================cursor_pos:{:?}", ui.io().mouse_draw_cursor);
                // let cursor_size = 5.0;
                // let cursor_half_size = cursor_size * 0.5;
                // draw_list
                //     .add_line([cursor_pos[0] - cursor_half_size, cursor_pos[1]],
                //               [cursor_pos[0] + cursor_half_size, cursor_pos[1]], [1.0, 0.0, 0.0, 1.0]).thickness(3.0).build();
                // draw_list.add_line([cursor_pos[0], cursor_pos[1] - cursor_half_size],
                //                    [cursor_pos[0], cursor_pos[1] + cursor_half_size], [1.0, 0.0, 0.0, 1.0]).build();
                if !is_selecting && !has_selection {
                    display.gl_window().window().set_cursor_visible(false);
                    let cursor_size = 20.0;
                    let cursor_half_size = cursor_size * 0.5;
                    draw_list
                        .add_line([cursor_pos[0] - cursor_half_size, cursor_pos[1]],
                                  [cursor_pos[0] + cursor_half_size, cursor_pos[1]], [1.0, 1.0, 1.0, 1.0]).build();
                    draw_list.add_line([cursor_pos[0], cursor_pos[1] - cursor_half_size],
                                       [cursor_pos[0], cursor_pos[1] + cursor_half_size], [1.0, 1.0, 1.0, 1.0]).build();
                }
                println!("ui.is_mouse_clicked(MouseButton::Left):{}", ui.is_mouse_clicked(MouseButton::Left));
                // 鼠标左键按下开始选择
                if ui.is_mouse_clicked(MouseButton::Left) && !is_selecting && !has_selection {
                    println!("==sdsssddssd");
                    display.gl_window().window().set_cursor_visible(true);
                    ui.set_mouse_cursor(Some(imgui::MouseCursor::Arrow));
                    is_selecting = true;
                    start_pos = mouse_pos;
                    end_pos = mouse_pos;
                }

                // 绘制选择框
                // if is_selecting || has_selection {
                if is_selecting || has_selection {
                    if is_selecting {
                        end_pos = mouse_pos;
                    }

                    draw_list.add_rect(
                        [start_pos[0], start_pos[1]],
                        [end_pos[0], end_pos[1]],
                        // [1.0, 0.0, 0.0, 1.0],
                        ImColor32::from(0xffffffff).to_rgba_f32s(),
                    )
                        .filled(false)
                        .thickness(2.0)
                        .build();

                    // 绘制背景图像
                    draw_list
                        .add_image(ui_info.background_texture_id, [start_pos[0], start_pos[1]], [end_pos[0], end_pos[1]])
                        // .add_image(ui_info.copy_to_clipboard_texture_id, [0.0, 0.0], [100.0, 100.0])
                        .uv_min([start_pos[0] / logical_width as f32, start_pos[1] / logical_height as f32])
                        .uv_max([end_pos[0] / logical_width as f32, end_pos[1] / logical_height as f32])
                        .col([1.0, 1.0, 1.0, 1.0])
                        .build();

                    // println!("copy_to_clipboard_texture_id:{:?}",ui_info.copy_to_clipboard_texture_id);

                    // 绘制拉伸点
                    let top_left = [start_pos[0], start_pos[1]];
                    let top_right = [start_pos[0], end_pos[1]];
                    let bottom_left = [end_pos[0], start_pos[1]];
                    let bottom_right = [end_pos[0], end_pos[1]];
                    draw_list
                        .add_circle(top_left, resize_handle_radius, [0.0, 0.0, 0.4, 1.0])
                        .filled(true)
                        .build();
                    draw_list
                        .add_circle(top_right, resize_handle_radius, [0.0, 0.0, 0.4, 1.0])
                        .filled(true)
                        .build();
                    draw_list
                        .add_circle(bottom_left, resize_handle_radius, [0.0, 0.0, 0.4, 1.0])
                        .filled(true)
                        .build();
                    draw_list
                        .add_circle(bottom_right, resize_handle_radius, [0.0, 0.0, 0.4, 1.0])
                        .filled(true)
                        .build();

                    if has_selection {
                        // 绘制一个按钮...
                        // if ui.button("Click me!") {
                        //     // 处理按钮点击事件的逻辑...
                        //     println!("Button clicked!");
                        // }
                        println!("bottom_right:{:?}", bottom_right);
                        ui.set_cursor_pos([bottom_right[0] - 6.0 * 20.0, bottom_right[1] + 10.0]);
                        ui.image_button("hello1", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                        ui.same_line_with_spacing(0.0, 0.0);
                        ui.image_button("hello2", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                        ui.same_line_with_spacing(0.0, 0.0);
                        ui.image_button("hello3", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                        ui.same_line_with_spacing(0.0, 0.0);
                        ui.image_button("hello4", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                        ui.same_line_with_spacing(0.0, 0.0);
                        ui.image_button("hello5", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                        ui.same_line_with_spacing(0.0, 0.0);
                        ui.image_button("hello6", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);

                        println!("ui.item_rect_size():{:?}", ui.item_rect_size());
                    }
                    // // 将列设置为矩形框下方
                    // ui.set_column_offset(0, 60.0); // 调整起始偏移量
                    // // 在界面中绘制一排可以点击的图标...
                    // let icon_size = [24.0, 24.0];
                    // let icon_spacing = 10.0;
                    // let num_icons = 5;
                    // ui.same_line_with_spacing(0.0, -1.0);
                    // for i in 0..num_icons {
                    //     if ui.button("##icon_button") {
                    //         // 点击图标按钮的处理逻辑...
                    //         println!("Clicked icon {}", i);
                    //     }
                    //     ui.same_line_with_spacing(10.0, -1.0);
                    //     // let draw_list = ui.get_window_draw_list();
                    //     // draw_list.add_image(icon_texture_id, [x, y], [x + icon_size[0], y + icon_size[1]]).build();
                    // }

                    let mut handle = active_resize_handle.borrow_mut();
                    // 检查是否有鼠标悬停在拉伸点上
                    let handle_top_left = top_left;
                    let handle_top_right = top_right;
                    let handle_bottom_left = bottom_left;
                    let handle_bottom_right = bottom_right;

                    if euclidean_distance(
                        PhysicalPosition::new(mouse_pos[0] as f64, mouse_pos[1] as f64),
                        PhysicalPosition::new(handle_top_left[0] as f64, handle_top_left[1] as f64),
                    ) <= 10.0 {
                        *handle = Some(ResizeHandle::TopLeft);
                    } else if euclidean_distance(
                        PhysicalPosition::new(mouse_pos[0] as f64, mouse_pos[1] as f64),
                        PhysicalPosition::new(handle_top_right[0] as f64, handle_top_right[1] as f64),
                    ) <= 10.0 {
                        *handle = Some(ResizeHandle::TopRight);
                    } else if euclidean_distance(
                        PhysicalPosition::new(mouse_pos[0] as f64, mouse_pos[1] as f64),
                        PhysicalPosition::new(handle_bottom_left[0] as f64, handle_bottom_left[1] as f64),
                    ) <= 10.0 {
                        *handle = Some(ResizeHandle::BottomLeft);
                    } else if euclidean_distance(
                        PhysicalPosition::new(mouse_pos[0] as f64, mouse_pos[1] as f64),
                        PhysicalPosition::new(handle_bottom_right[0] as f64, handle_bottom_right[1] as f64),
                    ) <= 10.0 {
                        *handle = Some(ResizeHandle::BottomRight);
                    } else {
                        println!("mouse_pos:{:?}", mouse_pos);
                        println!("handle_top_left:{:?}", handle_top_left);
                        println!("handle_bottom_right[0] - handle_bottom_left[0]:{:?}", handle_bottom_right[0] - handle_bottom_left[0]);
                        println!("handle_bottom_right[0] - handle_top_right[0]:{:?}", handle_bottom_right[0] - handle_top_right[0]);
                        if is_mouse_in_rect(
                            mouse_pos[0],
                            mouse_pos[1],
                            handle_top_left[0],
                            handle_top_left[1],
                            handle_bottom_right[0] - handle_top_left[0],
                            handle_bottom_right[1] - handle_top_left[1],
                        ) {
                            println!("=====================in");
                            *handle = Some(ResizeHandle::Inner);
                        } else {
                            *handle = Some(ResizeHandle::Outer);
                        }

                        if has_selection {
                            // let color = ui.push_style_color(imgui::StyleColor::Button, imgui::ImColor32::from(0xffffffff).to_rgba_f32s());
                            // let color = ui.push_style_color(imgui::StyleColor::ButtonHovered, imgui::ImColor32::from(0xff0000).to_rgba_f32s());
                            // ui.image_button("hello", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                            // ui.image_button("hello", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                            // ui.image_button("hello", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                            // ui.image_button("hello", ui_info.copy_to_clipboard_texture_id, [20 as f32, 20 as f32]);
                        }
                    }

                    // 根据激活的拉伸点更新矩形的位置和大小
                    if let Some(resize_handle) = *handle {
                        match resize_handle {
                            ResizeHandle::TopLeft => {
                                ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeNWSE));
                            }
                            ResizeHandle::TopRight => {
                                ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeNESW));
                            }
                            ResizeHandle::BottomLeft => {
                                ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeNESW));
                            }
                            ResizeHandle::BottomRight => {
                                ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeNWSE));
                            }
                            ResizeHandle::Inner => {
                                ui.set_mouse_cursor(Some(imgui::MouseCursor::ResizeAll));
                            }
                            ResizeHandle::Outer => {
                                ui.set_mouse_cursor(Some(imgui::MouseCursor::Arrow));
                            }
                        }
                    }
                }


                // 鼠标左键松开停止选择
                if ui.is_mouse_released(MouseButton::Left) && is_selecting {
                    is_selecting = false;
                    end_pos = mouse_pos;
                    has_selection = true;
                    // 在这里可以获取选取的矩形范围，即 start_pos 和 end_pos
                    println!("Selected area: {:?} - {:?}", start_pos, end_pos);
                }


                if select_paint {
                    // 在界面中绘制用户绘制的图形...
                    for point in &draw_state.points {
                        let draw_list_mut = ui.get_window_draw_list();
                        draw_list_mut.add_circle([point.position.0, point.position.1], 5.0, point.color).build();
                    }
                }

                // 将桌面截图渲染到UI上的一个矩形中
                // Image::new(my_texture_id, [1024 as f32, 768 as f32]).build(ui);
                // ui.image(
                //     screenshot_texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                //     [window_size[0], window_size[1]],
                // );
                // ui.text_wrapped("Hello world!");
                // if ui.button(choices[value]) {
                //     value += 1;
                //     value %= 2;
                // }
                //
                // ui.button("This...is...imgui-rs!");
                // ui.separator();
                // let mouse_pos = ui.io().mouse_pos;
                // ui.text(format!(
                //     "Mouse Position: ({:.1},{:.1})",
                //     mouse_pos[0], mouse_pos[1]
                // ));
            });
    });
}