mod clipboard;

use std::borrow::Cow;
use std::path::Path;
use glium::{glutin, implement_vertex, Texture2d};
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use glium::texture::TextureCreationError;
use image::DynamicImage;
use imgui_winit_support::winit::event::{Event, VirtualKeyCode, WindowEvent};
use imgui_winit_support::winit::event_loop::{ControlFlow, EventLoop};
use imgui_winit_support::winit::window::{Fullscreen, WindowBuilder};
use winapi::um::wingdi;


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
    let raw_image = glium::texture::RawImage2d {
        data: Cow::Borrowed(screenshot.unwrap().as_ref()),
        width: screen_size.0 as u32,
        height: screen_size.1 as u32,
        format: glium::texture::ClientFormat::U8U8U8,
    };

    let texture = glium::texture::Texture2d::new(display, raw_image)?;
    Ok(texture)
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


pub fn init(title: &str) -> System {
    let title = match Path::new(&title).file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => title,
    };
    let event_loop = EventLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = WindowBuilder::new()
        .with_title(title.to_owned())
        // .with_fullscreen(Some(Fullscreen::Borderless(None))) // 全屏窗口
        .with_inner_size(glutin::dpi::LogicalSize::new(1024f64, 768f64));
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

impl System {
    pub fn main_loop<F: FnMut(&mut bool, &mut Ui, &Display) + 'static>(self, mut run_ui: F) {
        let System {
            event_loop,
            display,
            mut imgui,
            mut platform,
            mut renderer,
            ..
        } = self;
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


        event_loop.run(move |event, _, control_flow| match event {
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
            }
            Event::RedrawRequested(_) => {
                let ui = imgui.frame();

                let mut run = true;
                run_ui(&mut run, ui, &display);
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
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input,
                    ..
                },
                ..
            } => {
                // 处理按键事件，可以在这里添加自定义逻辑
                if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                    *control_flow = ControlFlow::Exit;
                }
            }
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
            }
        })
    }
}