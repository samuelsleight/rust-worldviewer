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

struct Storage {
    renderer: Renderer,
    world: World,
    textures: HashMap<ChunkKey, Arc<dyn ImageViewAbstract>>,
}

type AppData = Data<Storage, Arc<Surface<Window>>>;

impl MainHandler for AppData {
    fn handle_event(&mut self, event: Event) -> Action<State> {
        match event {
            Event::CloseRequested => Action::Quit,
            _ => Action::Continue,
        }
    }

    fn handle_tick(&mut self) {
        for x in 0..5 {
            for y in 0..3 {
                let key = ChunkKey::new(x, y);

                match self.data.textures.entry(key) {
                    Occupied(_) => continue,
                    Vacant(entry) => {
                        entry.insert(self.data.renderer.create_texture(
                            self.data.world.generate_chunk_texture(key),
                            512,
                            512,
                            Format::R8G8B8A8_SRGB,
                        ));

                        return;
                    }
                }
            }
        }
    }

    fn handle_render(&self) {
        self.data.renderer.render(self.window(), |mut frame| {
            for x in 0..5 {
                for y in 0..3 {
                    let key = ChunkKey::new(x, y);
                    if let Some(texture) = self.data.textures.get(&key) {
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
            Ok(Storage {
                renderer,
                world: World::new(),
                textures: HashMap::new(),
            })
        },
    )
    .expect("Unable to initialise application")
    .run(60, State::Main())
}
