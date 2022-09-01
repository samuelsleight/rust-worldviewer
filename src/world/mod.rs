use std::time::Instant;

use worldgen::{
    constraint,
    noise::perlin::PerlinNoise,
    noisemap::{NoiseMap, NoiseMapGenerator, Seed, Step},
    world::{
        tile::{Constraint, ConstraintType},
        Size, Tile, World as WorldGen,
    },
};

pub use self::colour::Colour;

mod colour;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct ChunkKey {
    pub x: i64,
    pub y: i64,
}

pub struct World {
    worldgen: WorldGen<Colour>,
}

struct SizedIteratorWrapper<T, I: Iterator<Item = T>> {
    inner: I,
    size: usize,
}

impl<T, I: Iterator<Item = T>> SizedIteratorWrapper<T, I> {
    fn new(inner: I, size: usize) -> Self {
        Self { inner, size }
    }
}

impl<T, I: Iterator<Item = T>> Iterator for SizedIteratorWrapper<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<T, I: Iterator<Item = T>> ExactSizeIterator for SizedIteratorWrapper<T, I> {}

impl World {
    pub fn new() -> Self {
        let noise = PerlinNoise::new();

        let nm1 = NoiseMap::new(noise)
            .set(Seed::of(Instant::now()))
            .set(Step::of(0.005, 0.005));

        let nm2 = NoiseMap::new(noise)
            .set(Seed::of(Instant::now()))
            .set(Step::of(0.02, 0.02));

        let nm = Box::new(nm1 * 4 + nm2);

        let worldgen = WorldGen::new()
            .set(Size::of(512, 512))
            .add(Tile::new(Colour::new(0, 70, 170)).when(constraint!(nm.clone(), < -0.1)))
            .add(Tile::new(Colour::new(190, 180, 130)).when(constraint!(nm.clone(), < -0.05)))
            .add(Tile::new(Colour::new(20, 220, 100)).when(constraint!(nm.clone(), < 0.45)))
            .add(Tile::new(Colour::new(180, 180, 180)).when(constraint!(nm, < 0.85)))
            .add(Tile::new(Colour::new(220, 220, 220)));

        Self { worldgen }
    }

    pub fn generate_chunk_texture(
        &self,
        key: ChunkKey,
    ) -> impl Iterator<Item = u8> + ExactSizeIterator {
        let chunk = self.worldgen.generate(key.x, key.y);
        SizedIteratorWrapper::new(
            chunk
                .unwrap()
                .into_iter()
                .flatten()
                .flat_map(Colour::as_array),
            512 * 512 * 4,
        )
    }
}

impl ChunkKey {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}
