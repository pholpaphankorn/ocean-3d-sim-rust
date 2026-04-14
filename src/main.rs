const GRID: usize = 32;
const DT: f32 = 0.1;    // time step
const G: f32  = 9.8;    // gravity
const DX: f32 = 1.0;    // grid cell size

struct SimState {
    h: Vec<f32>,  // height field
    u: Vec<f32>,  // x velocity field
    v: Vec<f32>,  // z velocity field
}

impl SimState {
    fn new() -> Self {
        let size = GRID * GRID;
        let mut h = vec![1.0; size]; // start: flat water at height 1.0

        // drop a "splash" in the center to start things moving
        let cx = GRID / 2;
        let cz = GRID / 2;
        h[cz * GRID + cx] = 2.0; // one cell is higher than the rest

        SimState {
            h,
            u: vec![0.0; size], // everything starts still
            v: vec![0.0; size],
        }
    }
}
// fn step(sim: &mut SimState) {
//     let mut new_h = sim.h.clone();
//     let mut new_u = sim.u.clone();
//     let mut new_v = sim.v.clone();

//     for z in 1..(GRID - 1) {
//         for x in 1..(GRID - 1) {
//             let i  = z * GRID + x;
//             let ix = z * GRID + (x + 1); // neighbor to the right
//             let iz = (z + 1) * GRID + x; // neighbor below

//             // height changes because water flows in/out
//             new_h[i] -= DT * (
//                 (sim.u[ix] - sim.u[i]) / DX +  // flow in x
//                 (sim.v[iz] - sim.v[i]) / DX     // flow in z, we're using square grid so same DX
//             );

//             // velocity changes because of height differences (water flows downhill)
//             new_u[i] -= DT * G * (sim.h[ix] - sim.h[i]) / DX;
//             new_v[i] -= DT * G * (sim.h[iz] - sim.h[i]) / DX;
//         }
//     }

//     sim.h = new_h;
//     sim.u = new_u;
//     sim.v = new_v;
// }
fn step(sim: &mut SimState) {
    let mut new_h = sim.h.clone();
    let mut new_u = sim.u.clone();
    let mut new_v = sim.v.clone();

    for z in 1..(GRID - 1) {
        for x in 1..(GRID - 1) {
            let i    = z * GRID + x;
            let ir   = z * GRID + (x + 1); // right
            let il   = z * GRID + (x - 1); // left  ← was missing!
            let id   = (z + 1) * GRID + x; // down
            let iu   = (z - 1) * GRID + x; // up    ← was missing!

            // centered difference: look BOTH directions on each axis
            new_h[i] -= DT * (
                (sim.u[ir] - sim.u[il]) / (2.0 * DX) +
                (sim.v[id] - sim.v[iu]) / (2.0 * DX)
            );
            new_u[i] -= DT * G * (sim.h[ir] - sim.h[il]) / (2.0 * DX);
            new_v[i] -= DT * G * (sim.h[id] - sim.h[iu]) / (2.0 * DX);
        }
    }

    sim.h = new_h;
    sim.u = new_u;
    sim.v = new_v;
}
fn print_grid(sim: &SimState) {
    for z in 0..GRID {
        for x in 0..GRID {
            let h = sim.h[z * GRID + x];
            let symbol = match h {
                h if h > 1.5 => "▓▓",
                h if h > 1.1 => "▒▒",
                h if h > 0.9 => "░░",
                _             => "  ",
            };
            print!("{}", symbol);
        }
        println!();
    }
}

fn main() {
    let mut sim = SimState::new();

    for frame in 0..20 {
        println!("--- frame {} ---", frame);
        print_grid(&sim);
        println!();
        step(&mut sim);
    }
}