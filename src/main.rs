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

#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);


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

fn main() {
    use glium::{DisplayBuild, Surface};
    use std::time::{Duration, Instant};
    use std::thread::sleep;

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
    unifrom mat4 projection;

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
    
    void main() {
        color = texture(tex, v_tex_coords);
    }"#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_buffer = glium::VertexBuffer::new(&display,
        &[
            Vertex{ position: [0.0, 0.0]},
            Vertex{ position: [0.0, 1.0]},
            Vertex{ position: [1.0, 0.0]},
            Vertex{ position: [1.0, 1.0]},
        ]);

    let mut breakout = Game::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    breakout.init();

    let mut events = Events::new();

    let mut last_frame = Instant::now();
    let mut last_second = Instant::now();
    let mut fps = 0;

    loop {
        if events.now.quit || events.now.key_escape == Some(true) {
            return;
        }
        events.poll(&display);

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

        target.finish().unwrap();
    }
}
