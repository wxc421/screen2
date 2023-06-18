use std::fs;
use std::io::Cursor;
use std::ops::Deref;
use std::time::Instant;
use glium::{Display, Texture2d};
use glium::texture::{RawImage2d, Texture2dDataSource};
use imgui_glium_renderer::{Renderer, Texture};
use screenshots::{Image, Screen};
use usvg::TreeParsing;


// pub fn load_svg_icon(renderer: &mut Renderer, svg_path: &str) -> Result<imgui::TextureId, String> {
//     // 读取SVG文件
//     let svg_data = std::fs::read(svg_path).unwrap();
//     let rtree = usvg::Tree::from_data(&svg_data, &usvg::Options::default()).unwrap();
//
//     // 创建渲染目标
//     let mut pixmap = tiny_skia::Pixmap::new(512, 512).unwrap();
//     pixmap.fill(tiny_skia::Color::TRANSPARENT);
//
//     // 设置渲染参数
//     let mut options = usvg::FitToOptions {
//         padding: usvg::FitToPadding::Default,
//         alignment: usvg::FitToAlignment::Center,
//         ..Default::default()
//     };
//
//     // 渲染SVG到渲染目标
//     let mut rast = usvg::Rasterizer::new();
//     usvg::NodeExt::render_to(&rtree.root(), tiny_skia::PixmapMut::from(&mut pixmap), &options).unwrap();
//
//     // 转换渲染目标为OpenGL纹理
//     let texture = tiny_skia::rescale_surface(&pixmap, 512, 512);
//     let texture_id = renderer.textures().insert(texture);
//
//     Ok(texture_id)
// }


/// Load data to Texture2d
///
/// `Box<&dyn glium::backend::Facade>` 代表一个 `Box` 智能指针，指向一个实现了 `glium::backend::Facade` trait 的引用。
///
/// - `&dyn glium::backend::Facade` 表示一个对 `glium::backend::Facade` trait 类型的引用。
/// - `Box` 是一个智能指针类型，用于在堆上分配并存储数据。
///
/// 使用 `Box<&dyn glium::backend::Facade>` 可以将一个实现了 `glium::backend::Facade` trait 的引用包装在堆上的 `Box` 中，从而实现在堆上存储和传递数据的能力。
///
/// 通常情况下，将引用放入 `Box` 中的原因是需要在所有权转移的同时确保资源的生命周期延长，或者是为了在动态分配内存时保持特定的数据结构。
///
/// 例如，如果某个函数需要一个接收 `Box<&dyn glium::backend::Facade>` 类型参数，可以这样调用该函数：
///
/// ```rust
/// fn my_function(facade: Box<&dyn glium::backend::Facade>) {
///     // 函数体
/// }
///
/// fn main() {
///     let facade: &dyn glium::backend::Facade = ...; // 获取一个实现了 Facade trait 的引用
///     let boxed_facade: Box<&dyn glium::backend::Facade> = Box::new(facade); // 将引用包装在 Box 中
///     my_function(boxed_facade); // 传递 Box<&dyn glium::backend::Facade> 参数
/// }
/// ```
///
/// 在这个示例中，我们首先获取了一个实现了 `glium::backend::Facade` trait 的引用 `facade`。然后，我们用 `Box::new(facade)` 将引用放入 `Box` 中，创建一个 `Box<&dyn glium::backend::Facade>` 对象。最后，我们将这个 `Box` 对象传递给 `my_function` 函数。
///
/// 需要注意的是，`Box<&dyn glium::backend::Facade>` 类型的对象需要保证在其使用的整个生命周期内，被引用的对象都是有效的。
#[doc(alias = "loadDataTo2d")]
pub fn load_data_to_2d(data: &[u8], display: &Display) -> Texture2d {
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
    let texture = Texture2d::new(display, image).unwrap();

    texture
}


pub(crate) fn capture(display: &Display) -> Texture2d {
    // 创建一个Instant对象，用于记录开始时间
    let start = Instant::now();

    let screens = Screen::all().unwrap();
    // todo 这里拿到的值不对
    /*
        error: capturer [Screen { display_info: DisplayInfo { id: 2776250164, x: 0, y: 0, width: 2880, height: 1800, rotation: 0.0, scale_factor: 1.0, is_primary: true } }]
        正确的: capturer [Screen { display_info: DisplayInfo { id: 2776250164, x: 0, y: 0, width: 1440, height: 900, rotation: 0.0, scale_factor: 2.0, is_primary: true } }]
     */
    println!("capturer {screens:?}");
    let mut image = screens[0].capture().unwrap();
    println!("capture方法执行耗时: {:?}", start.elapsed());
    let width = image.width();
    let height = image.height();
    let vec: Vec<u8> = image.into();
    let image = glium::texture::RawImage2d::from_raw_rgba(vec, (width, height));


    // 计算方法执行所花费的总时间

    // 创建纹理对象
    let texture = Texture2d::new(display, image).unwrap();
    return texture;
}