use gl;
use std;
use image;
use std::ffi::{CString, CStr};
use resources::{self, Resources};

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len));
    unsafe { CString::from_vec_unchecked(buffer) }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to load resource {}: {}", name, inner)]
    ResourceLoad { name: String, #[cause] inner: resources::Error },
    #[fail(display = "Can not determine shader type for resource {}", name)]
    CanNotDetermineShaderTypeForResource { name: String },
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program {}: {}", name, message)]
    LinkError { name: String, message: String },
    #[fail(display = "Failed to load image {}", name)]
    ImageError { name: String, #[cause] inner: image::ImageError },
}

pub struct Shader {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Shader {
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
        ];
        let shader_kind = POSSIBLE_EXT.iter()
            .find(|&&(file_extension, _)| { name.ends_with(file_extension) })
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::CanNotDetermineShaderTypeForResource { name: name.into() })?;
        let source = res.load_cstring(name)
            .map_err(|e| Error::ResourceLoad { name: name.into(), inner: e })?;
        Shader::from_source(gl, &source, shader_kind).map_err(|message| Error::CompileError { name: name.into(), message })
    }

    pub fn from_source(gl: &gl::Gl, source: &CStr, kind: gl::types::GLuint) -> Result<Shader, String> {
        let mut success: gl::types::GLint = 1;
        let id = unsafe { gl.CreateShader(kind) };

        unsafe {
            gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl.CompileShader(id);
            gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl.GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            }

            return Err(error.to_string_lossy().into_owned());
        }
        Ok(Shader{ gl: gl.clone(), id: id })
    }

    pub fn from_vert_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Program {
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Program, Error> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];
        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| { Shader::from_res(gl, res, &format!("{}{}", name, file_extension)) })
            .collect::<Result<Vec<Shader>, Error>>()?;
        Program::from_shaders(gl, &shaders[..])
            .map_err(|message| Error::LinkError { name: name.into(), message })
    }

    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let id = unsafe { gl.CreateProgram() };
        for shader in shaders {
            unsafe { gl.AttachShader(id, shader.id()); }
        }

        unsafe { gl.LinkProgram(id); }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl.GetProgramInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl.DetachShader(id, shader.id()); }
        }

        Ok(Program { gl: gl.clone(), id: id })
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Texture {
    id: gl::types::GLuint,
}

impl Texture {
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Texture, Error> {
        let mut id: gl::types::GLuint = 0;
        let data: &[u8] = & res.load_vector(name).map_err(|e| Error::ResourceLoad { name: name.into(), inner: e })?;
        let im = image::load_from_memory(data).map_err(|err| Error::ImageError { name: name.into(), inner: err })?;
        let (width, height) = im.to_rgb().dimensions();
        let data: Vec<u8> = im.raw_pixels();
        unsafe {
            gl.GenTextures(1, &mut id);
            gl.BindTexture(gl::TEXTURE_2D, id);

            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            
            gl.TexImage2D(gl::TEXTURE_2D, 
                          0, 
                          gl::RGB as gl::types::GLint,
                          width as gl::types::GLint, 
                          height as gl::types::GLint,
                          0, 
                          gl::RGB, 
                          gl::UNSIGNED_BYTE, 
                          data.as_ptr() as *const gl::types::GLvoid);
            gl.GenerateMipmap(gl::TEXTURE_2D);
        };
        Ok(Texture { id: id })
    }
}
