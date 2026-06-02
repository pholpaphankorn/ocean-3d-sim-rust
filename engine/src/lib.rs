use wasm_bindgen::prelude::*;

pub mod dolphin;
pub mod physics;
pub mod waves;

use dolphin::Dolphin;
use physics::{FluidGrid, GRID};
use waves::{gerstner_stack, WAVE_SCALE};

#[wasm_bindgen]
pub struct SimState {
    grid: FluidGrid,
    time: f32,
    dolphin: Dolphin,
}

#[wasm_bindgen]
impl SimState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        SimState {
            grid: FluidGrid::new(),
            time: 0.0,
            dolphin: Dolphin::new(),
        }
    }

    pub fn step(&mut self) {
        // 1. Advance fluid physics
        self.grid.compute_step(self.time);

        // 2. Map dolphin's world position to fluid grid coordinates
        // Your grid is centered, so we offset by half the grid size (64.0)
        let grid_x = (self.dolphin.x + 64.0) as usize;
        let grid_z = (self.dolphin.z + 64.0) as usize;

        // Safety boundary check to prevent out-of-bounds panics on the grid vector
        if grid_x < GRID && grid_z < GRID {
            let current_eta = self.grid.eta[grid_z * GRID + grid_x];

            // 3. Update dolphin kinematics against the localized water height
            // Using `self.dolphin` points cleanly to the value on your struct
            if let Some((sx, sz, force)) = self.dolphin.update(0.1, current_eta) {
                self.grid.add_splash(sx, sz, force);
            }
        }
        self.time += 0.1; // Matches internal DT advancement increments cleanly
    }

    pub fn grid_size(&self) -> usize {
        GRID
    }

    pub fn get_vertices(&self) -> Vec<f32> {
        let mut vertices = Vec::with_capacity(GRID * GRID * 4);
        let half = GRID as f32 / 2.0;
        let mean = self.grid.eta.iter().sum::<f32>() / self.grid.eta.len() as f32;

        for z in 0..GRID {
            for x in 0..GRID {
                let px = x as f32 - half;
                let pz = z as f32 - half;

                // Extraction logic remains compatible with frontend strides
                let swe_y = self.grid.eta[z * GRID + x] - mean;
                let (gdx, gdy, gdz) = gerstner_stack(px * WAVE_SCALE, pz * WAVE_SCALE, self.time);
                let energy = 1.0 + swe_y.abs() * 1.5;

                vertices.push(px + gdx * energy);
                vertices.push(swe_y + gdy * energy);
                vertices.push(pz + gdz * energy);
                vertices.push(swe_y);
            }
        }
        vertices
    }

    pub fn get_indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        let g = GRID as u32;
        for z in 0..(g - 1) {
            for x in 0..(g - 1) {
                let i = z * g + x;
                indices.push(i);
                indices.push(i + g);
                indices.push(i + 1);
                indices.push(i + 1);
                indices.push(i + g);
                indices.push(i + g + 1);
            }
        }
        indices
    }

    pub fn splash(&mut self, x: usize, z: usize, amount: f32) {
        self.grid.add_splash(x, z, amount);
    }

    pub fn get_dolphin_vertices(&self) -> Vec<f32> {
        self.dolphin.generate_mesh(self.time)
    }
}
