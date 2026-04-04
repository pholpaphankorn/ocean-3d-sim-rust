use wasm_bindgen::prelude::*;

fn gerstner_wave(x: f32, z: f32, time: f32, dir_x: f32, dir_z: f32) -> f32 {
    let amplitude = 1.0;
    let frequency = 1.0;
    let speed = 1.0;
    let dot = dir_x * x + dir_z * z;
    amplitude * (frequency * dot + speed * time).sin()
}

fn ocean_height(x: f32, z: f32, time: f32) -> f32 {
    let wave1 = gerstner_wave(x, z, time, 1.0, 0.0);
    let wave2 = gerstner_wave(x, z, time, 0.7, 0.3) * 0.5;
    let wave3 = gerstner_wave(x, z, time, 0.0, 1.0) * 0.3;
    wave1 + wave2 + wave3
}

#[wasm_bindgen]
pub fn get_vertices(grid_size: u32, time: f32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let half = grid_size as f32 / 2.0;

    for row in 0..grid_size {
        for col in 0..grid_size {
            // center the grid around origin
            let x = col as f32 - half;
            let z = row as f32 - half;
            let y = ocean_height(x, z, time);
            vertices.push(x);
            vertices.push(y);
            vertices.push(z);
        }
    }
    vertices
}

// indices tell GPU which 3 vertices form each triangle
#[wasm_bindgen]
pub fn get_indices(grid_size: u32) -> Vec<u32> {
    let mut indices = Vec::new();
    for row in 0..(grid_size - 1) {
        for col in 0..(grid_size - 1) {
            let i = row * grid_size + col;
            // two triangles per grid square
            indices.push(i);
            indices.push(i + grid_size);
            indices.push(i + 1);

            indices.push(i + 1);
            indices.push(i + grid_size);
            indices.push(i + grid_size + 1);
        }
    }
    indices
}