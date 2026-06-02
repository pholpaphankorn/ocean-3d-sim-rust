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
            y: -5.0,
            z: 0.0,
            vx: 0.5,
            vy: 4.5,
            vz: 0.5, // Initial upward breaching velocity
            length: 4.0,
            segments: 8,
        }
    }

    pub fn update(&mut self, dt: f32, water_height: f32) -> Option<(usize, usize, f32)> {
        // Simple Projectile Motion / Kinematics
        if self.y > water_height {
            self.vy -= 9.8 * dt; // Gravity in air
        } else {
            self.vy += (water_height - self.y) * 2.0 - self.vy * 0.1; // Buoyancy counter-force
        }

        let old_y = self.y;
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.z += self.vz * dt;

        // Splash Trigger: Check if crossing the water interface downwards
        if old_y >= water_height && self.y < water_height {
            // Return grid coordinates to trigger a physical splash
            let grid_x = (self.x + 64.0).clamp(1.0, 126.0) as usize;
            let grid_z = (self.z + 64.0).clamp(1.0, 126.0) as usize;
            return Some((grid_x, grid_z, 25.0)); // X, Z, Splash Force
        }
        None
    }

    /// Generates interleaved [x, y, z, normal/color] vertices along a flexing sine wave spine
    pub fn generate_mesh(&self, time: f32) -> Vec<f32> {
        let mut verts = Vec::new();
        let radial_segments = 6;

        for s in 0..=self.segments {
            let t = s as f32 / self.segments as f32;
            let local_z = (t - 0.5) * self.length;

            // Flex body segments organically using a wave formula matching your Gerstner profiles
            let flex = (local_z * 1.5 - time * 5.0).sin() * 0.3 * (1.0 - t);

            // Compute thickness profile of the dolphin body (tapered capsule)
            let radius = (t * std::f32::consts::PI).sin() * 0.8;

            for r in 0..radial_segments {
                let angle = (r as f32 / radial_segments as f32) * std::f32::consts::PI * 2.0;

                let px = self.x + angle.cos() * radius + flex;
                let py = self.y + angle.sin() * radius;
                let pz = self.z + local_z;

                verts.push(px);
                verts.push(py);
                verts.push(pz);
                verts.push(0.7); // Simple attribute flag for shading
            }
        }
        verts
    }
}
