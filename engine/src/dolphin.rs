pub struct Dolphin {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub length: f32,
    pub segments: usize,
}

impl Dolphin {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 3.0, // Move it above the water surface plane (y = 0) so it's perfectly visible
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0, // Clear velocities
            length: 4.0,
            segments: 8,
        }
    }

    pub fn update(&mut self, dt: f32, water_height: f32) -> Option<(usize, usize, f32)> {
        // // Simple Projectile Motion / Kinematics
        // if self.y > water_height {
        //     self.vy -= 9.8 * dt; // Gravity in air
        // } else {
        //     self.vy += (water_height - self.y) * 2.0 - self.vy * 0.1; // Buoyancy counter-force
        // }

        // let old_y = self.y;
        // self.x += self.vx * dt;
        // self.y += self.vy * dt;
        // self.z += self.vz * dt;

        // // Splash Trigger: Check if crossing the water interface downwards
        // if old_y >= water_height && self.y < water_height {
        //     // Return grid coordinates to trigger a physical splash
        //     let grid_x = (self.x + 64.0).clamp(1.0, 126.0) as usize;
        //     let grid_z = (self.z + 64.0).clamp(1.0, 126.0) as usize;
        //     return Some((grid_x, grid_z, 25.0)); // X, Z, Splash Force
        // }
        None
    }

    /// Generates interleaved [x, y, z, normal/color] vertices along a flexing sine wave spine
    pub fn generate_mesh(&self, time: f32) -> Vec<f32> {
        let mut verts = Vec::new();
        
        // Increase radial segments to 16 for beautiful, high-resolution curves
        let radial_segments = 16; 

        // Helper closure to compute vertex positions and shading for any point on the body
        let get_vertex = |s: usize, r: usize| -> [f32; 4] {
            let t = s as f32 / self.segments as f32;
            let local_z = (t - 0.5) * self.length;

            // Flex body segments organically using a wave formula matching your Gerstner profiles
            let flex = (local_z * 1.5 - time * 5.0).sin() * 0.3 * (1.0 - t);

            // Wrap around radial segments cleanly to seal the tube perfectly
            let r_mod = r % radial_segments;
            let angle = (r_mod as f32 / radial_segments as f32) * std::f32::consts::PI * 2.0;

            // ─── 1. BASE DOLPHIN BODY ENVELOPE ──────────────────────────────────────
            let mut base_radius = (t * std::f32::consts::PI).sin() * 0.8;
            
            // Define a prominent, thin beak/snout at the front of the head (t close to 1.0)
            if t > 0.92 {
                let snout_t = (t - 0.92) / 0.08;
                base_radius = 0.42 * (1.0 - snout_t) + 0.12 * snout_t;
            }

            // Real dolphins are slender side-to-side, but tall up-and-down (ovular)
            let mut rx = base_radius * 0.82;
            let mut ry = base_radius * 1.15;

            // ─── 2. PROCEDURAL FIN DISPLACEMENTS ─────────────────────────────────────
            // A. Dorsal Fin (Top ridge of the back, roughly in the center: t = 0.42 to 0.62)
            if t > 0.42 && t < 0.62 {
                let fin_profile = ((t - 0.42) / 0.20 * std::f32::consts::PI).sin();
                // Isolate the top-center ridge of the back (angle = PI/2)
                let angle_dist = (angle - std::f32::consts::FRAC_PI_2).cos().max(0.0);
                // Exponent 8.0 makes the fin incredibly sharp and narrow
                ry += fin_profile * angle_dist.powf(8.0) * 0.8; 
            }

            // B. Tail Flukes (Horizontal wing at the absolute tip of the tail: t < 0.16)
            if t < 0.16 {
                let fluke_profile = (1.0 - t / 0.16).powf(2.0);
                // Isolate the horizontal sides (angle = 0 or angle = PI)
                let angle_dist = angle.cos().abs();
                rx += fluke_profile * angle_dist.powf(4.0) * 1.2;
                ry *= 1.0 - fluke_profile * 0.65; // Flatten the tail structure vertically
            }

            // C. Pectoral Flippers (Lower sides of the body: t = 0.65 to 0.78)
            if t > 0.65 && t < 0.78 {
                let flip_profile = ((t - 0.65) / 0.13 * std::f32::consts::PI).sin();
                let angle_dist = angle.cos().abs();
                // Push outward and slightly downward on the lower hemisphere
                if angle.sin() < 0.1 {
                    rx += flip_profile * angle_dist.powf(5.0) * 0.5;
                    ry -= flip_profile * angle_dist.powf(5.0) * 0.15;
                }
            }

            // Compute final absolute vertex coordinates
            let px = self.x + angle.cos() * rx + flex;
            let py = self.y + angle.sin() * ry;
            let pz = self.z + local_z;

            // ─── 3. PROCEDURAL SHADING & COUNTERSHADING ──────────────────────────────
            let nx = angle.cos();
            let ny = angle.sin();

            // Calculate directional diffuse lighting from a mock sun at the upper-front-right sky
            let dot = nx * 0.3 + ny * 0.9;
            let ambient = 0.35;
            let diffuse = dot.max(0.0) * 0.55;
            let mut intensity = ambient + diffuse;

            // Biological Countershading: Dark slate grey back, bright white belly
            let counter_shading = if ny > 0.0 {
                1.0 - ny * 0.25 // Darken the top half
            } else {
                1.0 - ny * 0.50 // Lighten the bottom half
            };
            
            intensity = (intensity * counter_shading).clamp(0.2, 1.3);

            [px, py, pz, intensity]
        };

        // ─── 4. MESH GRID TO CONTINUOUS WebGPU TRIANGLE STRIP ───────────────────────
        // Alternating sequence loops to construct a beautifully connected continuous cylinder mesh
        for s in 0..self.segments {
            for r in 0..=radial_segments {
                // Add point from the current ring (s)
                verts.extend_from_slice(&get_vertex(s, r));
                // Add matching point from the next ring forward (s + 1)
                verts.extend_from_slice(&get_vertex(s + 1, r));
            }
        }

        verts
    }
}
