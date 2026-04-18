use wasm_bindgen::prelude::*;

const GRID: usize = 32;
const DT: f32 = 0.1;
const G: f32 = 9.8;
const DX: f32 = 1.0;
const H: f32 = 1.0;          // ← fixed mean depth (linearization point)
const DAMPING: f32 = 0.995;

// We simulate η (deviation from mean), not total height
// This keeps wave speed = sqrt(G*H) = constant → always stable

#[wasm_bindgen]
pub struct SimState {
    eta: Vec<f32>, // surface deviation from mean (+ = crest, - = trough)
    u:   Vec<f32>, // x-velocity on x-faces
    v:   Vec<f32>, // z-velocity on z-faces
}

#[wasm_bindgen]
impl SimState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let size = GRID * GRID;
        let mut eta = vec![0.0_f32; size];

        // Gaussian splash — smooth initial condition, no sharp spikes
        let cx = (GRID / 2) as f32;
        let cz = (GRID / 2) as f32;
        for z in 0..GRID {
            for x in 0..GRID {
                let dx = x as f32 - cx;
                let dz = z as f32 - cz;
                let r2 = dx*dx + dz*dz;
                eta[z * GRID + x] = 1.5 * (-r2 / 4.0).exp();
            }
        }

        SimState { eta, u: vec![0.0; size], v: vec![0.0; size] }
    }

    pub fn step(&mut self) {
        // LEAPFROG — update η first, then u/v using new η
        // This is symplectic (energy-conserving) by construction

        // Step 1: update η from velocity divergence
        let mut new_eta = self.eta.clone();
        for z in 0..GRID {
            for x in 0..GRID {
                let i = z * GRID + x;

                let u_r = if x + 1 < GRID { self.u[z * GRID + (x+1)] } else { 0.0 };
                let u_l = self.u[i]; // boundary: u[0,z] = 0 always
                let v_b = if z + 1 < GRID { self.v[(z+1) * GRID + x] } else { 0.0 };
                let v_t = self.v[i]; // boundary: v[x,0] = 0 always

                new_eta[i] -= DT * H / DX * ((u_r - u_l) + (v_b - v_t));
            }
        }

        // Step 2: update u/v from gradient of NEW η
        let mut new_u = self.u.clone();
        let mut new_v = self.v.clone();

        for z in 0..GRID {
            for x in 1..GRID { // x=0 face stays 0 (wall boundary)
                let i  = z * GRID + x;
                let il = z * GRID + (x - 1);
                new_u[i] -= DT * G / DX * (new_eta[i] - new_eta[il]);
                new_u[i] *= DAMPING;
            }
        }

        for z in 1..GRID { // z=0 face stays 0 (wall boundary)
            for x in 0..GRID {
                let i  = z * GRID + x;
                let iu = (z - 1) * GRID + x;
                new_v[i] -= DT * G / DX * (new_eta[i] - new_eta[iu]);
                new_v[i] *= DAMPING;
            }
        }

        self.eta = new_eta;
        self.u   = new_u;
        self.v   = new_v;
    }

    pub fn get_vertices(&self) -> Vec<f32> {
        let mut vertices = Vec::new();
        let half = GRID as f32 / 2.0;
        for z in 0..GRID {
            for x in 0..GRID {
                vertices.push(x as f32 - half);
                vertices.push(self.eta[z * GRID + x]);
                vertices.push(z as f32 - half);
            }
        }
        vertices
    }

    pub fn get_indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        let g = GRID as u32;
        for z in 0..(g-1) {
            for x in 0..(g-1) {
                let i = z * g + x;
                indices.push(i);     indices.push(i+g); indices.push(i+1);
                indices.push(i+1);   indices.push(i+g); indices.push(i+g+1);
            }
        }
        indices
    }

    pub fn splash(&mut self, x: usize, z: usize, amount: f32) {
        // Gaussian splash instead of single spike
        let cx = x as f32;
        let cz = z as f32;
        for dz in 0..GRID {
            for dx in 0..GRID {
                let ddx = dx as f32 - cx;
                let ddz = dz as f32 - cz;
                let r2 = ddx*ddx + ddz*ddz;
                self.eta[dz * GRID + dx] += amount * (-r2 / 3.0).exp();
            }
        }
    }
}