#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;

#[macro_use]
mod events;

use cgmath::{Vector2};
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
}
implement_vertex!(Vertex, position);

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const PLAYER_SIZE: Vector2<f32> = Vector2{x: 100.0, y: 20.0};
const PLAYER_VELOCITY: f32 = 500.0;
const BALL_SIZE: Vector2<f32> = Vector2 { x: 10.0, y: 10.0};

fn load_texture_from_file(path: &str, display: &GlutinFacade) -> glium::Texture2d {
    use std::path::Path;
    let image = image::open(Path::new(path)).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);

    let texture = glium::texture::Texture2d::new(display, image).unwrap();
    texture
}

fn main() {
    use glium::{DisplayBuild, Surface};
    use std::time::Instant;

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
        .with_title("Breakout!")
        .build_glium()
        .unwrap();

    // Load textures into array
    use std::collections::HashMap;
    let texture_dict: HashMap<&str, u32> = {
        let mut map: HashMap<&str, u32> = HashMap::new();
        let paths = vec!["block.png", "block_solid.png"];
        for (num, path) in (0u32..).zip(paths.into_iter()) {
            map.insert(path, num);
        }
        map
    };
    let textures = {
        let images = texture_dict.iter().map(|(path, _)| {
            use std::path::Path;
            let folder = "textures/".to_string();
            let path = folder + &path;
            let image = image::open(Path::new(&path)).unwrap().to_rgba();
            let image_dimensions = image.dimensions();
            glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions)
        }).collect::<Vec<_>>();

        glium::texture::Texture2dArray::new(&display, images).unwrap()
    };

    // Load up level data from file into Vertex and Index buffers
    let level_data: Vec<Vec<String>> = {
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::*;
        // Load from file 
        let f = File::open("levels/one.lvl").unwrap();
        let reader = BufReader::new(f);
        reader.lines()
            .map(|l| l.unwrap())
            .map(|line| 
                line.clone()
                    .split_whitespace()
                    .map(|s| String::from(s))
                    .collect())
            .collect()
        };

    let (level_rows, level_columns) = (level_data.len() as f32, level_data[0].len() as f32);

    let (block_vertices, block_indexes) = {
        #[derive(Copy, Clone)]
        struct BlockVertex {
            position: [f32; 2],
            tex_id: u32,
            color: [f32; 3],
        }

        implement_vertex!(BlockVertex, position, tex_id, color);

        let block_count = (level_rows * level_columns) as usize;

        let mut vb_data: Vec<BlockVertex> = Vec::with_capacity(block_count * 4);
        let mut ib_data = Vec::with_capacity(block_count * 6);

        let (unit_width, unit_height) = (SCREEN_WIDTH as f32 / level_columns, (SCREEN_HEIGHT as f32/ 2.0) / level_rows);
        let mut block_counter = 0;
        for (y_pos, line) in level_data.into_iter().enumerate() {
            for (x_pos, value) in line.into_iter().enumerate() {
                let value = value.parse::<u8>().unwrap();
                if value == 0 { continue; }
                

                let left = unit_width * x_pos as f32;
                let right = left + unit_width;
                let top = unit_height * y_pos as f32;
                let bottom = top + unit_height;
                let tex_id = match value {
                    1 => texture_dict.get("block_solid.png").unwrap(),
                    2 ... 5 => texture_dict.get("block.png").unwrap(),
                    _ => texture_dict.get("block_solid.png").unwrap(),
                };
                let color = {
                        match value {
                            1 => [0.8, 0.8, 0.7],
                            2 => [0.2, 0.6, 1.0],
                            3 => [0.0, 0.7, 0.0],
                            4 => [0.8, 0.8, 0.4],
                            5 => [1.0, 0.5, 0.0],
                            _ => [1.0, 1.0, 1.0],
                        }
                    };
                
                vb_data.push( BlockVertex { position: [left, top], tex_id: tex_id.clone(), color: color});
                vb_data.push( BlockVertex { position: [right, top], tex_id: tex_id.clone(), color: color});
                vb_data.push( BlockVertex { position: [left, bottom], tex_id: tex_id.clone(), color: color});
                vb_data.push( BlockVertex { position: [right, bottom], tex_id: tex_id.clone(), color: color});

                let num = block_counter as u16;
                ib_data.push(num * 4);
                ib_data.push(num * 4 + 1);
                ib_data.push(num * 4 + 2);
                ib_data.push(num * 4 + 1);
                ib_data.push(num * 4 + 3);
                ib_data.push(num * 4 + 2);

                block_counter += 1;
            }
        }

        let vb = glium::VertexBuffer::new(&display, &vb_data).unwrap();
        let ib = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &ib_data).unwrap();

        (vb, ib)
    };

    let vertex_shader_src = r#"
    #version 140

    in vec2 position;
    in uint tex_id;
    in vec3 color;

    out vec2 v_tex_coords;
    out vec3 v_color;
    flat out uint v_tex_id;

    uniform mat4 projection;

    void main() {
        v_color = color;
        v_tex_id = tex_id;
        if (gl_VertexID % 4 == 0) {
            v_tex_coords = vec2(0.0, 1.0);
        } else if (gl_VertexID % 4 == 1) {
            v_tex_coords = vec2(1.0, 1.0);
        } else if (gl_VertexID % 4 == 2) {
            v_tex_coords = vec2(0.0, 0.0);
        } else {
            v_tex_coords = vec2(1.0, 0.0);
        }
        
        gl_Position = projection * vec4(position, 0.0, 1.0);
    }
    "#;

    let fragment_shader_src = r#"
    #version 140
    
    in vec2 v_tex_coords;
    flat in uint v_tex_id;
    in vec3 v_color;
    out vec4 color;
    
    uniform sampler2DArray tex;
    uniform vec3 sprite_color;
    
    void main() {
        color = texture(tex, vec3(v_tex_coords, float(v_tex_id))) * vec4(v_color, 1.0);
    }"#;

    let block_program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    /* Add in vertex buffer for general objects
     * Could go dynamic or go the old way with the object model
     * being uploaded to uniform each draw call
     * If it were to be dynamic, would only draw certain slices of an index_buffer
     * at each draw call. This would make the vertex storage faster, 
     * with the vertex positions being modified through memory instead of through
     * the CPU bound matrix calculations. The matrix calculation is definitely easier (was implemented)
     * but I think it's a better idea to actually go the way of a dynamic vertex buffer
     * This seems dumb in retrospect since only need to draw Background, Ball and Paddle.
     */

    let (background_vertices, background_indices) = {
        /* Need to store Background, Paddle and Ball */
        

        let mut vb_data = Vec::with_capacity(4);
        let mut ib_data: Vec<u16> = Vec::with_capacity(6);

        /* Add Background */
        vb_data.push( Vertex { position: [0.0, 0.0]});
        vb_data.push( Vertex { position: [SCREEN_WIDTH as f32, 0.0]});
        vb_data.push( Vertex { position: [0.0, SCREEN_HEIGHT as f32]});
        vb_data.push( Vertex { position: [SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32]});

        ib_data.push(0);
        ib_data.push(1);
        ib_data.push(2);
        ib_data.push(1);
        ib_data.push(3);
        ib_data.push(2);

        let vb = glium::VertexBuffer::new(&display, &vb_data).unwrap();
        let ib = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &ib_data).unwrap();

        (vb, ib)
    };

    

    let (paddle_vertices, paddle_indices) = {
        let mut ib_data: Vec<u16> = Vec::with_capacity(6);

        ib_data.push(0);
        ib_data.push(1);
        ib_data.push(2);
        ib_data.push(1);
        ib_data.push(3);
        ib_data.push(2);

        let vb = glium::VertexBuffer::empty_dynamic(&display, 4).unwrap();
        let ib = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &ib_data).unwrap();

        (vb, ib)
    };

    let (ball_vertices, ball_indices) = {
        let mut ib_data: Vec<u16> = Vec::with_capacity(6);

        ib_data.push(0);
        ib_data.push(1);
        ib_data.push(2);
        ib_data.push(1);
        ib_data.push(3);
        ib_data.push(2);

        let vb: glium::VertexBuffer<Vertex> = glium::VertexBuffer::empty_dynamic(&display, 4).unwrap();
        let ib = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &ib_data).unwrap();

        (vb, ib)
    };

    let background_texture = load_texture_from_file("textures/background.jpg", &display);
    let paddle_texture = load_texture_from_file("textures/paddle.png", &display);
    let ball_texture = load_texture_from_file("images/ball.png", &display);

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;

        uniform mat4 projection;

        out vec2 v_tex_coords;

        void main() {

            if (gl_VertexID % 4 == 0) {
                v_tex_coords = vec2(0.0, 1.0);
            } else if (gl_VertexID % 4 == 1) {
                v_tex_coords = vec2(1.0, 1.0);
            } else if (gl_VertexID % 4 == 2) {
                v_tex_coords = vec2(0.0, 0.0);
            } else {
                v_tex_coords = vec2(1.0, 0.0);
            }
            gl_Position = projection * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;
        
        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let default_program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let perspective: [[f32; 4]; 4] = {
        let persp: cgmath::Matrix4<f32> = cgmath::ortho(0.0, SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32, 0.0, -1.0, 1.0);
        Into::<[[f32; 4]; 4]>::into(persp)
    };

    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };

    let mut events = Events::new();

    let mut last_frame = Instant::now();
    let mut last_second = Instant::now();
    let mut fps = 0;
    let mut elapsed;

    let mut paddle_position: Vector2<f32> = Vector2 {
        x: (SCREEN_WIDTH / 2) as f32,
        y: (SCREEN_HEIGHT as f32 - PLAYER_SIZE.y / 2.0) as f32,
    };

    let mut ball_position: Vector2<f32> = Vector2 {
        x: paddle_position.x,
        y: paddle_position.y - (PLAYER_SIZE.y / 2.0) - BALL_SIZE.y,
    };

    loop {

        // Handle FPS
        {
            let dt = last_frame.elapsed().subsec_nanos() as f32 / 1.0e6; // ns -> ms
            elapsed = dt / 1.0e3; // ms -> s
            last_frame = Instant::now();
            fps += 1;
            if last_frame.duration_since(last_second).as_secs() >= 1 {
                println!("FPS: {:?}", fps);
                last_second = Instant::now();
                fps = 0;
            }
        }

        // Ball movement and collision checking

        // Handle events
        {
            let velocity = PLAYER_VELOCITY * elapsed;

            if events.key_left {
                if paddle_position.x - PLAYER_SIZE.x / 2.0 >= 0.0 {
                    paddle_position.x -= velocity;
                    if paddle_position.x - PLAYER_SIZE.x / 2.0 < 0.0 {
                        paddle_position.x = PLAYER_SIZE.x / 2.0;
                    }
                }
            }
            if events.key_right {
                if paddle_position.x + PLAYER_SIZE.x / 2.0 <= SCREEN_WIDTH as f32 {
                    paddle_position.x += velocity;
                    if paddle_position.x + PLAYER_SIZE.x / 2.0 > SCREEN_WIDTH as f32 {
                        paddle_position.x = SCREEN_WIDTH as f32 - PLAYER_SIZE.x / 2.0;
                    }
                }
            }

            ball_position.x = paddle_position.x;
        }

        // Paddle screen position
        {
            let left = paddle_position.x - PLAYER_SIZE.x / 2.0;
            let right = paddle_position.x + PLAYER_SIZE.x / 2.0;
            let bottom = paddle_position.y + PLAYER_SIZE.y / 2.0;
            let top = paddle_position.y - PLAYER_SIZE.y / 2.0;

            let mut vb_data: Vec<Vertex> = Vec::with_capacity(4);
            vb_data.push( Vertex { position: [left, top]});
            vb_data.push( Vertex { position: [right, top]});
            vb_data.push( Vertex { position: [left, bottom]});
            vb_data.push( Vertex { position: [right, bottom]});

            paddle_vertices.write(&vb_data);
        }

        {
            let left = ball_position.x - BALL_SIZE.x;
            let right = ball_position.x + BALL_SIZE.x;
            let bottom = ball_position.y + BALL_SIZE.y;
            let top = ball_position.y - BALL_SIZE.y;

            let mut vb_data: Vec<Vertex> = Vec::with_capacity(4);
            vb_data.push( Vertex { position: [left, top]});
            vb_data.push( Vertex { position: [right, top]});
            vb_data.push( Vertex { position: [left, bottom]});
            vb_data.push( Vertex { position: [right, bottom]});

            ball_vertices.write(&vb_data);
        }

        // Draw graphics
        {
            let mut target = display.draw();
            // Clears to black
            target.clear_color(0.0, 0.0, 0.0, 1.0);

            // Draw background, paddle and ball
            {
                let uniforms = uniform! {
                    projection: perspective,
                    tex: &background_texture
                };

                target.draw(&background_vertices,
                        &background_indices,
                        &default_program,
                        &uniforms,
                        &params)
                    .unwrap();
            }

            {
                let uniforms = uniform! {
                    projection: perspective,
                    tex: &paddle_texture
                };

                target.draw(&paddle_vertices,
                        &paddle_indices,
                        &default_program,
                        &uniforms,
                        &params)
                    .unwrap();
            }
            
            // Draw ball
            {
                let uniforms = uniform! {
                    tex: &ball_texture,
                    projection: perspective,
                };

                target.draw(
                    &ball_vertices,
                    &ball_indices,
                    &default_program,
                    &uniforms,
                    &params
                ).unwrap();
            }

            // Draw blocks
            {
                let uniforms = uniform! {
                    tex: &textures,
                    projection: perspective,
                };

                target.draw(&block_vertices,
                        &block_indexes,
                        &block_program,
                        &uniforms,
                        &params)
                    .unwrap();  
            }

            
            
            target.finish().unwrap();
        }
        

        events.poll(&display);
        if events.now.quit || events.now.key_escape == Some(true) {
            return;
        }
    }
}
