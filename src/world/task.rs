use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use worldgen::{
    constraint,
    noise::perlin::PerlinNoise,
    noisemap::{NoiseMap, NoiseMapGenerator, Seed, Step},
    world::{
        tile::{Constraint, ConstraintType},
        Size, Tile, World,
    },
};

use super::{Chunk, ChunkKey, Colour};

pub struct Worldgen {
    world: World<Colour>,
}

impl Worldgen {
    pub fn generate_chunk(&self, x: i64, y: i64) -> Vec<Vec<Colour>> {
        self.world.generate(x, y).unwrap()
    }
}

unsafe impl Sync for Worldgen {}

pub fn worldgen_task(rx: Receiver<ChunkKey>, tx: Sender<Chunk>) {
    let noise = PerlinNoise::new();

    let nm1 = NoiseMap::new(noise)
        .set(Seed::of(Instant::now()))
        .set(Step::of(0.005, 0.005));

    let nm2 = NoiseMap::new(noise)
        .set(Seed::of(Instant::now()))
        .set(Step::of(0.02, 0.02));

    let nm = Box::new(nm1 * 4 + nm2);

    let worldgen = Worldgen {
        world: World::new()
            .set(Size::of(512, 512))
            .add(Tile::new(Colour::new(0, 70, 170)).when(constraint!(nm.clone(), < -0.1)))
            .add(Tile::new(Colour::new(190, 180, 130)).when(constraint!(nm.clone(), < -0.05)))
            .add(Tile::new(Colour::new(20, 220, 100)).when(constraint!(nm.clone(), < 0.45)))
            .add(Tile::new(Colour::new(180, 180, 180)).when(constraint!(nm, < 0.85)))
            .add(Tile::new(Colour::new(220, 220, 220))),
    };

    std::thread::scope(|scope| {
        let mut handles = Vec::new();

        for _ in 0..12 {
            let thread_rx = rx.clone();
            let thread_tx = tx.clone();
            let thread_wg = &worldgen;

            handles.push(scope.spawn(move || {
                while let Ok(key) = thread_rx.recv() {
                    let data = thread_wg.generate_chunk(key.x, key.y);
                    thread_tx.send(Chunk::new(key, data)).unwrap()
                }
            }));
        }

        handles.clear()
    })
}
