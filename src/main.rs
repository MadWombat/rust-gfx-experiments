extern crate gl;
extern crate sdl2;
extern crate image;
extern crate nalgebra as na;
#[macro_use] extern crate failure;

pub mod render;
pub mod resources;

use failure::err_msg;
use resources::Resources;

use std::f32::consts::PI;
use std::ffi::{CString};
use na::{Vector3, Matrix4};

fn main() {
    if let Err(e) = run() {
        println!("{}", e);
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_exe_path()?;

    let sdl = sdl2::init().map_err(err_msg)?;
    let video = sdl.video().map_err(err_msg)?;

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video
        .window("Playground", 1024, 768)
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let program = render::Program::from_res(&gl, &res, "shaders/triangle")?;

    let tex = render::Texture::from_res(&gl, &res, "textures/wall.jpg")?;

    let vertices: Vec<f32> = vec![
         0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, // top left
         0.5,  0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, // top right
        -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bot left
        -0.5,  0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, // bot right
    ];

    let indices: Vec<u32> = vec![
        0, 1, 3,
        0, 2, 3,
    ];

    let mut vbo: gl::types::GLuint = 0;
    let mut vao: gl::types::GLuint = 0;
    let mut ebo: gl::types::GLuint = 0;

    /* setup all the buffers */
    unsafe {
        gl.GenVertexArrays(1, &mut vao);
        gl.BindVertexArray(vao);

        gl.GenBuffers(1, &mut vbo);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl.BufferData(gl::ARRAY_BUFFER,
                      (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                      vertices.as_ptr() as *const gl::types::GLvoid,
                      gl::STATIC_DRAW);

        gl.GenBuffers(1, &mut ebo);
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

        gl.BufferData(gl::ELEMENT_ARRAY_BUFFER,
                      (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                      indices.as_ptr() as *const gl::types::GLvoid,
                      gl::STATIC_DRAW);

        /* vertex coordinate */
        gl.VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE,
                               (8 * std::mem::size_of::<f32>()) as gl::types::GLint,
                               std::ptr::null());
        gl.EnableVertexAttribArray(0);
        /* color */
        gl.VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE,
                               (8 * std::mem::size_of::<f32>()) as gl::types::GLint,
                               (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        gl.EnableVertexAttribArray(1);
        /* texture coordinate */
        gl.VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE,
                               (8 * std::mem::size_of::<f32>()) as gl::types::GLint,
                               (6 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        gl.EnableVertexAttribArray(2);

        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);
    }

    /* enable wireframe mode */
    // unsafe {
    //     gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE)
    // }

    /* setup viewport and clear the screen */
    unsafe {
        gl.Viewport(0, 0, 1024, 768);
        gl.ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    program.set_used();

    let mut iter = (0..200).map(|x| { x as f32 * 0.01 * PI }).cycle();

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {}
            }

        }

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        program.set_used();

        let transform = Matrix4::new_rotation(&Vector3::x() * iter.next().unwrap());
        unsafe {
            let uni_loc = gl.GetUniformLocation(program.id(), CString::new("transform")?.as_ptr());
            gl.UniformMatrix4fv(uni_loc, 1, gl::FALSE, transform.as_slice().as_ptr());
        }

        unsafe {
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, tex.id());

            gl.BindVertexArray(vao);
            gl.DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            gl.BindVertexArray(0);
        }

        window.gl_swap_window();
    }

    Ok(())
}
