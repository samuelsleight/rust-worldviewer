use std::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

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

pub fn worldgen_task(rx: Receiver<ChunkKey>, tx: Sender<Chunk>) {
    let noise = PerlinNoise::new();

    let nm1 = NoiseMap::new(noise)
        .set(Seed::of(Instant::now()))
        .set(Step::of(0.005, 0.005));

    let nm2 = NoiseMap::new(noise)
        .set(Seed::of(Instant::now()))
        .set(Step::of(0.02, 0.02));

    let nm = Box::new(nm1 * 4 + nm2);

    let worldgen = World::new()
        .set(Size::of(512, 512))
        .add(Tile::new(Colour::new(0, 70, 170)).when(constraint!(nm.clone(), < -0.1)))
        .add(Tile::new(Colour::new(190, 180, 130)).when(constraint!(nm.clone(), < -0.05)))
        .add(Tile::new(Colour::new(20, 220, 100)).when(constraint!(nm.clone(), < 0.45)))
        .add(Tile::new(Colour::new(180, 180, 180)).when(constraint!(nm, < 0.85)))
        .add(Tile::new(Colour::new(220, 220, 220)));

    while let Ok(key) = rx.recv() {
        let data = worldgen.generate(key.x, key.y).unwrap();
        tx.send(Chunk::new(key, data)).unwrap()
    }
}
