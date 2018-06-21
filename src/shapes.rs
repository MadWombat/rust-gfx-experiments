extern crate nalgebra as na;

#[derive(Debug)]
pub struct Vertix {
    coords: [f32; 3],
    color: [f32; 3],
    texcoords: [f32; 2],
}

impl Vertix {
    pub fn as_slice(&self) -> Vec<f32> {
        [[self.coords, self.color].concat(), self.texcoords.to_vec()].concat()
    }
}

#[derive(Debug)]
pub struct Shape {
    vertices: Vec<Vertix>,
    indices: Vec<i32>,
}

impl Shape {
    pub fn rect(width: f32, height: f32) -> Shape {
        /* generate a rectangle with 0.0 as bottom left */
        Shape {
            vertices: vec![
                /* bottom left */
                Vertix { 
                    coords: [0.0, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texcoords: [0.0, 0.0],
                },
                /* top left */
                Vertix { 
                    coords: [0.0, height, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texcoords: [0.0, 1.0],
                },
                /* top right */
                Vertix { 
                    coords: [width, height, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texcoords: [1.0, 1.0],
                },
                /* bottom right */
                Vertix { 
                    coords: [width, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texcoords: [0.0, 1.0],
                },
                ],
            indices: vec![0, 1, 2, 0, 3, 2],
        }
    }

    pub fn indices(&self) -> Vec<i32> {
        self.indices.clone()
    }

    pub fn vertices(&self) -> Vec<f32> {
        self.vertices.iter().map(|v| v.as_slice()).collect::<Vec<Vec<f32>>>().concat()
    }
}


