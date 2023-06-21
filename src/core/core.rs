use std::io::Cursor;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::{Display, glutin, Surface, Texture2d};
use glium::glutin::event::{Event, VirtualKeyCode, WindowEvent};
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::glutin::platform::windows::{EventLoopBuilderExtWindows, WindowBuilderExtWindows};
use imgui::{Condition, Context, ImColor32, StyleColor, Ui};
use imgui_glium_renderer::{Renderer, Texture};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui;
use screenshots::Screen;

struct Cropper {
    pub display: Display,
    pub event_loop: EventLoop<()>,
    pub screen: Screen,
    pub ui_info: Option<UiInfo>,
    pub platform: Option<WinitPlatform>,
    pub renderer: Option<Renderer>,
    pub context: Option<Context>,
}

struct UiInfo {
    copy_to_clipboard_texture_id: imgui::TextureId,
    background_texture_id: imgui::TextureId,
}

impl Cropper {
    pub fn capture(&self) -> Texture2d {
        let image = self.screen.capture().unwrap();
        let width = image.width();
        let height = image.height();
        let vec: Vec<u8> = image.into();
        let image = glium::texture::RawImage2d::from_raw_rgba(vec, (width, height));
        // 创建纹理对象
        let texture = Texture2d::new(&self.display, image).unwrap();
        return texture;
    }

    pub fn load_data_to_2d(&self, data: &[u8]) -> Texture2d {
        // 创建一个Instant对象，用于记录开始时间
        let start = Instant::now();


        let image = image::load(Cursor::new(data),
                                image::ImageFormat::Png).unwrap().to_rgba8();

        // 计算方法执行所花费的总时间
        println!("image::load(Cursor::new(data)方法执行耗时: {:?}", start.elapsed());

        let image_dimensions = image.dimensions();
        /*
            在绘制图像时，常见的约定是将图像的原点放在左上角。这意味着像素的索引值从左上角开始，并且行是从上到下依次递增的。
            然而，一些图形API和图像文件格式使用不同的约定，将图像的原点放在左下角。这意味着像素的索引值从左下角开始，并且行是从下到上递减的。
            当你从一个使用左下角原点的约定（比如某些图像文件格式）加载图像数据时，你可能需要将图像数据转换为使用左上角原点的约定。这就是所谓的“反转纹理的行”。
            在`glium`中，使用`RawImage2d::from_raw_rgba_reversed`函数时，会将行顺序从从左下到右上的像素数据进行反转，以符合使用左上角原点的约定。
            这样做是为了确保加载的图像在绘制到窗口或纹理时，按正确的方向显示。

            两个函数from_raw_rgba和from_raw_rgba_reversed都可以用于创建2D纹理对象，具体取决于图像数据的存储方式。
            from_raw_rgba函数将直接使用像素数据创建纹理对象，不会对图像行进行反转。这适用于像素数据按照从左上到右下的顺序排列的情况，如一些图形API的约定。
            from_raw_rgba_reversed函数将在创建纹理对象之前反转图像的行顺序，以符合从左上到右下的约定。这适用于像素数据按照从左下到右上的顺序排列的情况，如一些图像文件格式的约定。
            在使用这两个函数时，你需要根据图像的存储方式选择正确的函数来保证纹理在渲染时显示正确。如果你不确定图像数据的存储方式，可以尝试使用其中一个函数创建纹理，然后观察结果。如果结果呈现不正确，你可以尝试使用另一个函数。
            总之，使用 from_raw_rgba 或者 from_raw_rgba_reversed 函数都可以用来创建2D纹理对象，取决于图像数据的存储方式。
         */
        let image_vec = image.into_raw();
        let image = glium::texture::RawImage2d::from_raw_rgba(image_vec, image_dimensions);


        // 创建纹理对象
        let texture = Texture2d::new(&self.display, image).unwrap();

        texture
    }

    pub fn init(&mut self) {
        let mut context = Context::create();
        context.set_ini_filename(None);
        // 设置 ImGui 样式
        let mut style = context.style_mut();
        style.child_border_size = 0.0;
        // 修改按钮的默认颜色
        style.colors[StyleColor::Button as usize] = ImColor32::from_rgb(255, 255, 255).to_rgba_f32s();
        style.colors[StyleColor::WindowBg as usize] = ImColor32::from_rgb(0, 0, 255).to_rgba_f32s();
        style.frame_border_size = 0.0;
        style.frame_padding = [0.0, 0.0];
        style.item_inner_spacing = [0.0, 0.0];

        let mut platform = WinitPlatform::init(&mut context);

        platform.attach_window(context.io_mut(), self.display.gl_window().window(), HiDpiMode::Default);

        let mut renderer = Renderer::init(&mut context, &self.display).expect("Failed to initialize renderer");
        let texture2d = self.load_data_to_2d(include_bytes!("../../fuzhidaojiantieban.png"));
        let copy_to_clipboard_texture_id = renderer.textures().insert(Texture {
            texture: Rc::new(texture2d),
            sampler: Default::default(),
        });


        let texture2d = self.capture();
        let background_texture_id = renderer.textures().insert(Texture {
            texture: Rc::new((texture2d)),
            sampler: Default::default(),
        });

        self.context = Some(context);
        self.platform = Some(platform);
        self.renderer = Some(renderer);

        let ui_info = UiInfo {
            copy_to_clipboard_texture_id,
            background_texture_id,
        };
        self.ui_info = Some(ui_info);
    }

    pub fn start_event_loop(mut self) {
        // let Cropper {
        //     mut event_loop,
        //     ..
        // } = self;
        println!("start into event_loop");
        self.event_loop.run_return(|event, _, control_flow| {
            let gl_window = self.display.gl_window();
            *control_flow = ControlFlow::Wait;
            println!("{:?}", event);
            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                }
                Event::MainEventsCleared => {
                    self.platform.as_mut().unwrap()
                        .prepare_frame(self.context.as_mut().unwrap().io_mut(), gl_window.window())
                        .expect("Failed to prepare frame");
                    let ui = self.context.as_mut().unwrap().frame();
                    println!("thread::sleep(Duration::from_secs(5));");
                    // thread::sleep(Duration::from_secs(5));
                    let clear_color = [0.0, 0.0, 1.0, 1.0]; // 使用灰色作为背景色
                    let _ = ui
                        .push_style_color(
                            StyleColor::WindowBg,
                            clear_color);
                    // thread::sleep(Duration::from_secs(3));

                    // let gl_window = display.gl_window();
                    let mut target = self.display.draw();
                    target.clear_color_srgb(1.0, 0.0, 0.0, 1.0);
                    self.platform.as_mut().unwrap().prepare_render(ui, gl_window.window());

                    Cropper::run_ui(ui, &self.display, self.ui_info.as_ref().unwrap());

                    let draw_data = self.context.as_mut().unwrap().render();

                    self.renderer.as_mut().unwrap()
                        .render(&mut target, draw_data)
                        .expect("Rendering failed");
                    target.finish().expect("Failed to swap buffers");
                    println!("self.display.gl_window().window().set_visible(true);");
                    // thread::sleep(Duration::from_secs(3));
                    self.display.gl_window().window().set_visible(true);
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
                        let position = self.platform.as_mut().unwrap().scale_pos_from_winit(window, position);
                        self.context.as_mut().unwrap().io_mut().add_mouse_pos_event([position.x as f32, position.y as f32]);
                    }
                    event => {
                        let event1: Event<WindowEvent> = Event::WindowEvent {
                            window_id,
                            event,

                        };
                        // let gl_window = display.gl_window();
                        self.platform.as_mut().unwrap().handle_event(self.context.as_mut().unwrap().io_mut(), gl_window.window(), &event1);
                    }
                }
                event => {
                    let gl_window = self.display.gl_window();
                    self.platform.as_mut().unwrap().handle_event(self.context.as_mut().unwrap().io_mut(), gl_window.window(), &event);
                }
            }
        });
    }

    fn run_ui(ui: &mut Ui, display: &Display, ui_info: &UiInfo) {
        // 这里获得是物理像素值: 2880x1800 我的电脑缩放是200%，因此需要/2
        let (width, height) = display.get_framebuffer_dimensions();

        let scale_factor = display.gl_window().window().scale_factor();

        let logical_width = (width as f64 / scale_factor) as u32;
        let logical_height = (height as f64 / scale_factor) as u32;

        ui.window("window")
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
                        ui_info.background_texture_id,
                        [0.0, 0.0],
                        // [window_size[0] as f32, window_size[1] as f32],
                        [window_size[0] as f32, window_size[1] as f32],
                    )
                    .col([1.0, 1.0, 1.0, 0.6])
                    .build();
            });
    }
}


pub fn run() {
    let screens = Screen::all().unwrap();
    let mut croppers: Vec<_> = screens.into_iter().map(|screen| {
        /*
            如果你已经开启了垂直同步（VSync），它将根据显示器的刷新率来限制帧速率。
            在这种情况下，通常不需要额外控制帧率，因为 VSync 会自动将帧速率与显示器的刷新率同步，避免出现撕裂和过度消耗资源的情况。
            VSync 的目的是为了在图像渲染时消除撕裂效应，并提供平滑的视觉体验。
            开启 VSync 可以让图像与显示器的刷新率同步，每秒刷新次数不会超过显示器的最大刷新率（通常为 60Hz 或 120Hz）。
         */
        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_double_buffer(Some(true));
        let builder = glutin::window::WindowBuilder::new()
            .with_visible(false)
            .with_always_on_top(true)
            .with_decorations(false)
            .with_skip_taskbar(true) // 设置窗口构建器的跳过任务栏选项
            // .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(None)))
            ;
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
            platform: Default::default(),
            renderer: Default::default(),
            context: Default::default(),
        }
    }).collect();

    let mut handles = vec![];
    for mut cropper in croppers.into_iter() {
        unsafe {
            handles.push(thread::spawn(move || {
                cropper.init();
                cropper.start_event_loop()
            }));
        }
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

