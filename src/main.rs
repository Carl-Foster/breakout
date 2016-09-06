#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;

#[macro_use]
mod events;

use cgmath::{Vector2, Vector3, Matrix4};
use cgmath::{vec2, vec3, rad};
use glium::texture::Texture2d;
use glium::backend::glutin_backend::GlutinFacade;

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
const PLAYER_SIZE: Vector2<f32> = Vector2{x: 100.0, y: 20.0};
const PLAYER_VELOCIT: f32 = 500.0;

#[derive(Debug)]
enum GameState {
    Active,
    Menu,
    Win,
}



#[derive(Debug)]
struct GameObject {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub color: Vector3<f32>,
    pub rotation: cgmath::Rad<f32>,
    
    pub is_solid: bool,
    pub destroyed: bool,

    texture_path: String,
}

impl GameObject {
    pub fn new(position: Vector2<f32>, size: Vector2<f32>, color: Vector3<f32>, path: &str) -> GameObject {
        use cgmath::{vec2, vec3};
        GameObject {
            position: position,
            size: size,
            velocity: vec2(0.0, 0.0),
            color: color,
            rotation: cgmath::rad(0.0),
            is_solid: false,
            destroyed: false,
            texture_path: path.to_string(),
        }
    }

    fn get_matrix(&self) -> Matrix4<f32> {
        let matrix = transform_model_matrix(self.position, self.size, self.rotation);
        matrix
    }
}

#[derive(Debug)]
struct GameLevel {
    pub bricks: Vec<GameObject>,
}

impl GameLevel {
    pub fn new(path: &str, level_width: u32, level_height: u32) -> Result<GameLevel, std::io::Error> {
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::*;
        let mut tile_data = Vec::new();
        // Load from file 
        let f = try!(File::open(path));
        let reader = BufReader::new(f);

        let lines = reader.lines().map(|l| l.unwrap());

        for line in lines {
            let other_line = line.clone();
            tile_data.push(other_line.split_whitespace().map(|s| String::from(s)).collect());
        }

        Ok(
            GameLevel {
                bricks: GameLevel::init_data(tile_data, level_width, level_height),
            }
        )
    }

    pub fn init_data(data: Vec<Vec<String>>, level_width: u32, level_height: u32) -> Vec<GameObject> {
        use cgmath::{vec2, vec3};
        let height = data.len() as f32;
        let width = data[0].len() as f32;

        let mut bricks: Vec<GameObject> = Vec::new();

        let unit_width: f32 = level_width as f32 / width;
        let unit_height: f32 = level_height as f32 / height;

        let mut y_pos = 0.0;
        for row in data {
            let mut x_pos = 0.0;
            for col in row {
                println!("{:?}", col);
                let number = col.parse::<u8>().unwrap();
                if number == 1 {
                    let position = vec2(unit_width * x_pos, unit_height * y_pos);
                    let size = vec2(unit_width, unit_height);
                    let color = vec3(0.8, 0.8, 0.7);
                    let mut object = GameObject::new(position, size, color, "block_solid");
                    object.is_solid = true;
                    bricks.push(object);
                } else if number > 1 {
                    let color = {
                        match number {
                            2 => vec3(0.2, 0.6, 1.0),
                            3 => vec3(0.0, 0.7, 0.0),
                            4 => vec3(0.8, 0.8, 0.4),
                            5 => vec3(1.0, 0.5, 0.0),
                            _ => vec3(1.0, 1.0, 1.0),
                        }
                    };

                    let position = vec2(unit_width * x_pos, unit_height * y_pos);
                    let size = vec2(unit_width, unit_height);

                    let object = GameObject::new(position, size, color, "block");
                    bricks.push(object);
                }
                x_pos += 1.0;
            }
            y_pos += 1.0;
        }
        bricks
    }

    fn is_completed() -> bool {
        unimplemented!();   
    }
}

#[derive(Debug)]
struct Game {
    state: GameState,
    width: u32,
    height: u32,
    level: usize,
    levels: Vec<GameLevel>,

    player: GameObject,

    resources: ResourceManager,
}

impl Game {
    pub fn new(width: u32, height: u32, display: &GlutinFacade) -> Game {
        let mut resources = ResourceManager::new();
        resources.load_texture("textures/background.jpg", "background", display);
        resources.load_texture("textures/block.png", "block", display);
        resources.load_texture("textures/block_solid.png", "block_solid", display);
        resources.load_texture("textures/paddle.png", "paddle", display);
        // Load levels
        let mut levels = Vec::new();
        levels.push(GameLevel::new("levels/one.lvl", width, height / 2).unwrap());

        let player = GameObject::new(
            vec2(width as f32 / 2.0 - PLAYER_SIZE.x, height as f32 - PLAYER_SIZE.y),
            PLAYER_SIZE,
            vec3(1.0, 1.0, 1.0),
            "paddle");

        Game {
            state: GameState::Menu,
            width: width,
            height: height,
            level: 0,
            levels: levels,

            player: player,

            resources: resources,
        }
    }
}

#[derive(Debug)]
struct ResourceManager {
    pub textures: std::collections::HashMap<String, Texture2d>,
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        use std::collections::HashMap;
        ResourceManager {
            textures: HashMap::new(),
        }
    }

    pub fn load_texture (&mut self, path: &str, key: &str, display: &GlutinFacade) {
        self.textures.insert(key.to_string(), load_texture_from_file(path, display));
    }

    pub fn get_texture(&self, key: &str) -> &glium::texture::Texture2d {
        self.textures.get(key).unwrap()
    }
}

fn load_texture_from_file(path: &str, display: &GlutinFacade) -> glium::Texture2d {
    use std::path::Path;
    let image = image::open(Path::new(path)).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);

    let texture = glium::texture::Texture2d::new(display, image).unwrap();
    texture
}

fn load_sampler_from_texture<'t>(tex: &'t glium::Texture2d) -> glium::uniforms::Sampler<'t, glium::texture::Texture2d> {
    use glium::uniforms::{Sampler, SamplerWrapFunction, MinifySamplerFilter, MagnifySamplerFilter};
    let sampler = Sampler::new(tex)
        .wrap_function(SamplerWrapFunction::Repeat)
        .minify_filter(MinifySamplerFilter::Linear)
        .magnify_filter(MagnifySamplerFilter::Linear);
    sampler
}

fn transform_model_matrix(position: Vector2<f32>, scale: Vector2<f32>, rotation: cgmath::Rad<f32>) -> Matrix4<f32> {
    let translation_matrix: Matrix4<f32> = Matrix4::from_translation(position.extend(0.0));

    let first_trans_matrix: Matrix4<f32> = Matrix4::from_translation(cgmath::vec3(0.5 * scale.x, 0.5 * scale.y, 0.0));
    let rotation_matrix: Matrix4<f32> = Matrix4::from_angle_z(rotation);
    let second_trans_matrix: Matrix4<f32> = Matrix4::from_translation(cgmath::vec3(-0.5 * scale.x, -0.5 * scale.y, 0.0));

    let scale_matrix: Matrix4<f32> = Matrix4::from_nonuniform_scale(scale.x, scale.y, 1.0);
    translation_matrix * first_trans_matrix * rotation_matrix * second_trans_matrix *  scale_matrix
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
        color = texture(tex, v_tex_coords) * vec4(sprite_color, 1.0);
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

    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };

    let breakout = Game::new(SCREEN_WIDTH, SCREEN_HEIGHT, &display);

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
            // println!("FPS: {:?}", fps);
            last_second = Instant::now();
            fps = 0;
        }

        let mut target = display.draw();
        // Clears to black
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        // Draw background
        let model = transform_model_matrix(vec2(0.0, 0.0), vec2(breakout.width as f32, breakout.height as f32), cgmath::rad(0.0));
        let sampler = load_sampler_from_texture(breakout.resources.get_texture("background"));
        let sprite_color = vec3(1.0, 1.0, 1.0);

        let uniforms = uniform! {
                model: Into::<[[f32; 4]; 4]>::into(model),
                projection: Into::<[[f32; 4]; 4]>::into(perspective),
                tex: sampler,
                sprite_color: Into::<[f32; 3]>::into(sprite_color)
            };

        target.draw(&vertex_buffer,
                &indices,
                &program,
                &uniforms,
                &params)
            .unwrap();

        // Draw bricks
        for brick in &breakout.levels[breakout.level].bricks {
            let model: Matrix4<f32> = brick.get_matrix();
            let sampler = load_sampler_from_texture(breakout.resources.get_texture(&brick.texture_path));
            let sprite_color = brick.color; 
            
            let uniforms = uniform! {
                model: Into::<[[f32; 4]; 4]>::into(model),
                projection: Into::<[[f32; 4]; 4]>::into(perspective),
                tex: sampler,
                sprite_color: Into::<[f32; 3]>::into(sprite_color)
            };

            target.draw(&vertex_buffer,
                &indices,
                &program,
                &uniforms,
                &params)
            .unwrap();
        }      

        target.finish().unwrap();

        events.poll(&display);
        if events.now.quit || events.now.key_escape == Some(true) {
            return;
        }
    }
}
