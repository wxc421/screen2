use std::rc::Rc;
use std::time::Instant;
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::{Display, glutin, Texture2d};
use glium::glutin::event::{Event, VirtualKeyCode, WindowEvent};
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::glutin::platform::windows::EventLoopBuilderExtWindows;
use imgui::{Condition, Context, StyleColor};
use imgui_glium_renderer::{Renderer, Texture};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui_winit_support::HiDpiMode::Default;
use screenshots::Screen;

struct Cropper {
    pub display: Display,
    pub event_loop: EventLoop<()>,
    pub screen: Screen,
    pub ui_info: UiInfo,
    pub renderer: Renderer,
}

struct UiInfo {
    copy_to_clipboard_texture_id: imgui::TextureId,
    background_texture_id: imgui::TextureId,
}

impl Cropper {
    pub fn capture(&self) -> Texture2d {
        let mut image = self.screen.capture().unwrap();
        let vec: Vec<u8> = image.into();
        let image = glium::texture::RawImage2d::from_raw_rgba(vec, (image.width(), image.height()));
        // 创建纹理对象
        let texture = Texture2d::new(&self.display, image).unwrap();
        return texture;
    }

    pub fn init(&mut self) {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
        // 设置 ImGui 样式
        let mut style = imgui.style_mut();
        style.child_border_size = 0.0;
        // 修改按钮的默认颜色
        style.colors[StyleColor::Button as usize] = imgui::ImColor32::from_rgb(255, 255, 255).to_rgba_f32s();
        self.renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        imgui.style_mut().frame_border_size = 0.0;
        imgui.style_mut().frame_padding = [0.0, 0.0];
        imgui.style_mut().item_inner_spacing = [0.0, 0.0];

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

        self.ui_info = UiInfo {
            copy_to_clipboard_texture_id,
            background_texture_id,
        };
    }

    pub fn start_event_loop(&mut self) {
        self.event_loop.run_return(move |event, _, control_flow| {
            let gl_window = display.gl_window();
            *control_flow = ControlFlow::Poll;
            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                }
                Event::MainEventsCleared => {
                    platform
                        .prepare_frame(imgui.io_mut(), gl_window.window())
                        .expect("Failed to prepare frame");
                    let ui = imgui.frame();
                    let mut run = true;
                    self.run_ui();
                    if !run {
                        *control_flow = ControlFlow::Exit;
                    }

                    // let gl_window = display.gl_window();
                    let mut target = display.draw();
                    target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                    platform.prepare_render(ui, gl_window.window());
                    let draw_data = imgui.render();
                    renderer
                        .render(&mut target, draw_data)
                        .expect("Rendering failed");
                    target.finish().expect("Failed to swap buffers");
                    // display.gl_window().window().set_visible(true);
                    display.gl_window().window().set_visible(true);
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
                        let window = gl_window.window();
                        let scale = window.scale_factor(); // 2
                        let position = position.to_logical(scale);
                        let position = platform.scale_pos_from_winit(window, position);
                        imgui.io_mut().add_mouse_pos_event([position.x as f32, position.y as f32]);
                    }
                    event => {
                        let event1: Event<WindowEvent> = Event::WindowEvent {
                            window_id,
                            event,

                        };
                        // let gl_window = display.gl_window();
                        platform.handle_event(imgui.io_mut(), gl_window.window(), &event1);
                    }
                }
                event => {
                    let gl_window = display.gl_window();
                    platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
                }
            }
        });
    }

    fn run_ui(&mut self) {
        // 这里获得是物理像素值: 2880x1800 我的电脑缩放是200%，因此需要/2
        let (width, height) = display.get_framebuffer_dimensions();

        let scale_factor = display.gl_window().window().scale_factor();

        let logical_width = (width as f64 / scale_factor) as u32;
        let logical_height = (height as f64 / scale_factor) as u32;

        self.ui.window("Hello world")
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
            .build(|| {
                let window_size = ui.window_size();
                // 绘制一个矩形
                let draw_list = ui.get_window_draw_list();
                draw_list
                    .add_image(
                        self.ui_info.background_texture_id,
                        [0.0, 0.0],
                        // [window_size[0] as f32, window_size[1] as f32],
                        [window_size[0] as f32, window_size[1] as f32],
                    )
                    .col([1.0, 1.0, 1.0, 0.6])
                    .build();
            });
    }

    pub fn new(display: Display, event_loop: EventLoop<()>, screen: screenshots::Screen) -> Self {
        Self { display, event_loop, screen, ui_info: Default::default(), renderer: Default::default() }
    }
}


pub fn run() {
    let screens = Screen::all().unwrap();
    let croppers: Vec<_> = screens.into_iter().map(|screen| {
        /*
            如果你已经开启了垂直同步（VSync），它将根据显示器的刷新率来限制帧速率。
            在这种情况下，通常不需要额外控制帧率，因为 VSync 会自动将帧速率与显示器的刷新率同步，避免出现撕裂和过度消耗资源的情况。
            VSync 的目的是为了在图像渲染时消除撕裂效应，并提供平滑的视觉体验。
            开启 VSync 可以让图像与显示器的刷新率同步，每秒刷新次数不会超过显示器的最大刷新率（通常为 60Hz 或 120Hz）。
         */
        let context = glutin::ContextBuilder::new()
            .with_vsync(false)
            .with_double_buffer(Some(false));
        let builder = glutin::window::WindowBuilder::new()
            .with_visible(false)
            .with_always_on_top(true)
            .with_decorations(false)
            .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(None)));
        // 初始化 EventLoop
        let event_loop = glium::glutin::event_loop::EventLoopBuilder::new()
            .with_any_thread(true)
            .build();
        let display = Display::new(builder, context, &event_loop).unwrap();

        Cropper {
            display,
            event_loop,
            screen,
            ui_info: Default::default(),
            renderer: Default::default(),
        }
    }).collect();

    for cropper in croppers.iter() {
        cropper.start_event_loop()
    }
}

