use renderer::{InitError, Renderer};
use stateloop::{
    app::{App, Data, Event, Window},
    state::Action,
    states,
};
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    sync::Arc,
};
use vulkano::{
    format::Format,
    image::ImageViewAbstract,
    instance::{Instance, InstanceCreateInfo},
    swapchain::Surface,
};
use world::{ChunkKey, World};

mod renderer;
mod world;

states! {
    State {
        MainHandler Main()
    }
}

enum TextureEntry {
    Requested,
    Valid(Arc<dyn ImageViewAbstract>),
}

struct Storage {
    renderer: Renderer,
    world: World,
    bounds: (u32, u32),
    textures: HashMap<ChunkKey, TextureEntry>,
}

type AppData = Data<Storage, Arc<Surface<Window>>>;

impl Storage {
    fn new(surface: &Arc<Surface<Window>>, renderer: Renderer) -> Self {
        let mut storage = Self {
            renderer,
            world: World::new(),
            bounds: (0, 0),
            textures: HashMap::new()
        };

        storage.update_bounds(surface);
        storage
    }

    fn update_bounds(&mut self, surface: &Arc<Surface<Window>>) {
        let window_bounds = surface.window().inner_size().to_logical::<u32>(surface.window().scale_factor());
        let chunk_size = 300u32;
        self.bounds = (window_bounds.width / chunk_size, window_bounds.height / chunk_size);
    }
}

impl MainHandler for AppData {
    fn handle_event(&mut self, event: Event) -> Action<State> {
        match event {
            Event::Resized(..) => {
                let window = self.window().clone();
                self.data.update_bounds(&window);
                Action::Continue
            },
            Event::CloseRequested => Action::Quit,
            _ => Action::Continue,
        }
    }

    fn handle_tick(&mut self) {
        while let Some(chunk) = self.data.world.get_chunk_result() {
            self.data.textures.insert(
                chunk.key,
                TextureEntry::Valid(self.data.renderer.create_texture(
                    chunk.texture(),
                    512,
                    512,
                    Format::R8G8B8A8_SRGB,
                )),
            );
        }

        for x in 0..=self.data.bounds.0 {
            for y in 0..=self.data.bounds.1 {
                let key = ChunkKey::new(x as i64, y as i64);

                match self.data.textures.entry(key) {
                    Occupied(_) => continue,
                    Vacant(entry) => {
                        entry.insert(TextureEntry::Requested);
                        self.data.world.request_chunk(key);
                    }
                }
            }
        }
    }

    fn handle_render(&self) {
        self.data.renderer.render(self.window(), |mut frame| {
            for x in 0..=self.data.bounds.0 {
                for y in 0..=self.data.bounds.1 {
                let key = ChunkKey::new(x as i64, y as i64);
                    if let Some(&TextureEntry::Valid(ref texture)) = self.data.textures.get(&key) {
                        frame = frame.draw(
                            [(key.x * 300) as f32, (key.y * 300) as f32].into(),
                            texture.clone(),
                        );
                    }
                }
            }

            frame.finish()
        });
    }
}

fn main() {
    let instance = {
        let required_extensions = vulkano_win::required_extensions();

        Instance::new(InstanceCreateInfo {
            enabled_extensions: required_extensions,
            enumerate_portability: true,
            ..Default::default()
        })
        .expect("Unable to initialise vulkan")
    };

    let constructor_instance = instance.clone();

    App::new(
        move |event_loop| Renderer::construct_window(event_loop, constructor_instance),
        move |surface| -> Result<_, InitError> {
            let renderer = Renderer::init_vulkan(&instance, surface)?;
            Ok(Storage::new(surface, renderer))
        },
    )
    .expect("Unable to initialise application")
    .run(60, State::Main())
}
