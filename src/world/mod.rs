use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::JoinHandle,
};

pub use self::colour::Colour;

mod colour;
mod task;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct ChunkKey {
    pub x: i64,
    pub y: i64,
}

pub struct Chunk {
    pub key: ChunkKey,
    pub data: Vec<Vec<Colour>>,
}

pub struct World {
    tx: Sender<ChunkKey>,
    rx: Receiver<Chunk>,
    _thread: JoinHandle<()>,
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
        let (request_tx, request_rx) = channel();
        let (result_tx, result_rx) = channel();

        Self {
            _thread: std::thread::Builder::new()
                .name("World Viewer Generation Thread".into())
                .spawn(move || task::worldgen_task(request_rx, result_tx))
                .unwrap(),
            tx: request_tx,
            rx: result_rx,
        }
    }

    pub fn request_chunk(&self, key: ChunkKey) {
        self.tx.send(key).unwrap();
    }

    pub fn get_chunk_result(&self) -> Option<Chunk> {
        self.rx.try_recv().ok()
    }
}

impl Chunk {
    pub fn new(key: ChunkKey, data: Vec<Vec<Colour>>) -> Self {
        Self { key, data }
    }

    pub fn texture(self) -> impl Iterator<Item = u8> + ExactSizeIterator {
        SizedIteratorWrapper::new(
            self.data.into_iter().flatten().flat_map(Colour::as_array),
            512 * 512 * 4,
        )
    }
}

impl ChunkKey {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}
