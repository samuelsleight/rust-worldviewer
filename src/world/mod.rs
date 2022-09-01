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

        let nm = NoiseMap::new(noise)
            .set(Seed::of("test"))
            .set(Step::of(0.005, 0.005));

        let worldgen = WorldGen::new()
            .set(Size::of(512, 512))
            .add(Tile::new(Colour::new(0, 70, 170)).when(constraint!(Box::new(nm), < -0.1)))
            .add(Tile::new(Colour::new(20, 200, 90)));

        Self { worldgen }
    }

    pub fn generate_chunk_texture(
        &self,
        x: i64,
        y: i64,
    ) -> impl Iterator<Item = u8> + ExactSizeIterator {
        let chunk = self.worldgen.generate(x, y);
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
