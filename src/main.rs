#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;

#[macro_use]
mod events;

struct_events!{
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down,
        key_left: Left,
        key_right: Right
    },
    else: {
        quit: Closed
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);


const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

#[derive(Debug)]
enum GameState {
    Active,
    Menu,
    Win,
}

#[derive(Debug)]
struct Game {
    state: GameState,
    width: u32,
    height: u32,
}

impl Game {
    pub fn new(width: u32, height: u32) -> Game {
        Game {
            state: GameState::Menu,
            width: width,
            height: height,
        }
    }

    fn init(&self) {
        println!("Init Game");
    }

    fn process_input(&self, events: &Events, elapsed: f32) {
        // println!("Process Input for Game");
    }

    fn update(&self, elapsed: f32) {
        // println!("Update Game");
    }

    fn render(&self) {
        // println!("Render Game");
    }
}

fn load_texture_from_file(path: &str, display: &glium::backend::glutin_backend::GlutinFacade) -> glium::Texture2d {
    use std::path::Path;
    let image = image::open(Path::new(path)).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);

    let texture = glium::texture::Texture2d::new(display, image).unwrap();
    texture
}

fn transform_model_matrix(position: cgmath::Vector2<f32>, scale: cgmath::Vector2<f32>, rotation: cgmath::Rad<f32>) -> cgmath::Matrix4<f32> {
    use cgmath::Matrix4;
    let translation_matrix: Matrix4<f32> = Matrix4::from_translation(position.extend(0.0));
    let rotation_matrix: Matrix4<f32> = Matrix4::from_angle_z(rotation);

    let scale = scale.extend(1.0);
    let scale_matrix: Matrix4<f32> = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
    translation_matrix * rotation_matrix * scale_matrix
}

fn main() {
    use glium::{DisplayBuild, Surface};
    use std::time::Instant;

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
        .with_title("Breakout!")
        .build_glium()
        .unwrap();

    let vertex_shader_src = r#"
    #version 140

    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;

    uniform mat4 model;
    uniform mat4 projection;

    void main() {
        v_tex_coords = tex_coords;
        gl_Position = projection * model * vec4(position, 0.0, 1.0);
    }
    "#;

    let fragment_shader_src = r#"
    #version 140
    
    in vec2 v_tex_coords;
    out vec4 color;
    
    uniform sampler2D tex;
    uniform vec3 sprite_color;
    
    void main() {
        color = vec4(sprite_color, 1.0) * texture(tex, v_tex_coords);
    }"#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

    let vertex_buffer = glium::vertex::VertexBuffer::new(&display,
        &[
            Vertex{ position: [0.0, 1.0], tex_coords: [0.0, 1.0]},
            Vertex{ position: [0.0, 0.0], tex_coords: [0.0, 0.0]},
            Vertex{ position: [1.0, 0.0], tex_coords: [1.0, 0.0]},
            Vertex{ position: [1.0, 1.0], tex_coords: [1.0, 1.0]},
        ]).unwrap();

    let perspective: cgmath::Matrix4<f32> = cgmath::ortho(0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32, 0.0, -1.0, 1.0);
    let model: cgmath::Matrix4<f32> = transform_model_matrix(cgmath::vec2(200.0, 200.0), cgmath::vec2(300.0, 400.0), cgmath::Rad::from(cgmath::deg(45.0f32)));
    let texture = load_texture_from_file("images/face.png", &display);
    let sprite_color = [0.0, 1.0, 0.0f32];

    let uniforms = uniform! {
        model: Into::<[[f32; 4]; 4]>::into(model),
        projection: Into::<[[f32; 4]; 4]>::into(perspective),
        tex: &texture,
        sprite_color: sprite_color
    };

    let breakout = Game::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    breakout.init();

    let mut events = Events::new();

    let mut last_frame = Instant::now();
    let mut last_second = Instant::now();
    let mut fps = 0;

    loop {

        let dt = last_frame.elapsed().subsec_nanos() as f32 / 1.0e6; // ns -> ms
        let elapsed = dt / 1.0e3; // ms -> s
        last_frame = Instant::now();
        fps += 1;
        if last_frame.duration_since(last_second).as_secs() >= 1 {
            println!("FPS: {:?}", fps);
            last_second = Instant::now();
            fps = 0;
        }

        breakout.process_input(&events, elapsed);

        breakout.update(elapsed);

        let mut target = display.draw();
        // Clears to black
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        breakout.render();

        target.draw(&vertex_buffer,
            &indices,
            &program,
            &uniforms,
            &Default::default())
        .unwrap();

        target.finish().unwrap();

        events.poll(&display);
        if events.now.quit || events.now.key_escape == Some(true) {
            return;
        }
    }
}