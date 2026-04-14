use wasm_bindgen::prelude::*;

const GRID: usize = 32;
const DT: f32 = 0.02; // ← was 0.1, must satisfy: DT * sqrt(G) / DX < 1
const G: f32 = 9.8;
const DX: f32 = 1.0;
const DAMPING: f32 = 0.995; // ← bleeds a tiny bit of energy each step

// SimState lives on the JS heap, managed via wasm-bindgen
#[wasm_bindgen]
pub struct SimState {
    h: Vec<f32>,
    u: Vec<f32>,
    v: Vec<f32>,
}

#[wasm_bindgen]
impl SimState {
    // JS calls: const sim = SimState.new()
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let size = GRID * GRID;
        let mut h = vec![1.0; size];

        // splash in the center
        let cx = GRID / 2;
        let cz = GRID / 2;
        h[cz * GRID + cx] = 2.0;

        SimState {
            h,
            u: vec![0.0; size],
            v: vec![0.0; size],
        }
    }

    // JS calls: sim.step() each frame
    pub fn step(&mut self) {
        let mut new_h = self.h.clone();
        let mut new_u = self.u.clone();
        let mut new_v = self.v.clone();

        for z in 1..(GRID - 1) {
            for x in 1..(GRID - 1) {
                let i = z * GRID + x;
                let ir = z * GRID + (x + 1);
                let il = z * GRID + (x - 1);
                let id = (z + 1) * GRID + x;
                let iu = (z - 1) * GRID + x;

                new_h[i] -= DT
                    * ((self.u[ir] - self.u[il]) / (2.0 * DX)
                        + (self.v[id] - self.v[iu]) / (2.0 * DX));
                new_u[i] -= DT * G * (self.h[ir] - self.h[il]) / (2.0 * DX);
                new_v[i] -= DT * G * (self.h[id] - self.h[iu]) / (2.0 * DX);
                // ← damping: bleed a tiny bit of velocity each step
                new_u[i] *= DAMPING;
                new_v[i] *= DAMPING;
            }
        }

        self.h = new_h;
        self.u = new_u;
        self.v = new_v;
    }

    // JS calls: sim.get_vertices() to get the 3D mesh
    pub fn get_vertices(&self) -> Vec<f32> {
        let mut vertices = Vec::new();
        let half = GRID as f32 / 2.0;

        for z in 0..GRID {
            for x in 0..GRID {
                let px = x as f32 - half;
                let pz = z as f32 - half;
                let py = self.h[z * GRID + x] - 1.0; // subtract rest height
                vertices.push(px);
                vertices.push(py);
                vertices.push(pz);
            }
        }
        vertices
    }

    // JS calls this once to get the triangle indices (never changes)
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

    // drop a new splash at any grid position
    pub fn splash(&mut self, x: usize, z: usize, amount: f32) {
        if x < GRID && z < GRID {
            self.h[z * GRID + x] += amount;
        }
    }
}
