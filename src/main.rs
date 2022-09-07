use enumset::{EnumSet, EnumSetType};
use renderer::{
    material::{Material, TextureMaterial},
    InitError, Renderer,
};
use stateloop::{
    app::{App, Data, Event, Window},
    state::Action,
    states,
    winit::event::{ElementState, VirtualKeyCode},
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

#[derive(Debug, EnumSetType)]
pub enum InputState {
    Up,
    Down,
    Right,
    Left,
}

states! {
    State {
        MainHandler Main(input: EnumSet<InputState>)
    }
}

enum TextureEntry {
    Requested,
    Valid(Arc<dyn ImageViewAbstract>),
}

struct Storage {
    renderer: Renderer,
    world: World,
    material: Material<TextureMaterial>,
    bounds: (u32, u32),
    offset: (i64, i64),
    chunk_offset: (i64, i64),
    textures: HashMap<ChunkKey, TextureEntry>,
}

type AppData = Data<Storage, Arc<Surface<Window>>>;

impl Storage {
    fn new(surface: &Arc<Surface<Window>>, renderer: Renderer) -> Self {
        let material = renderer.create_material(TextureMaterial).unwrap();

        let mut storage = Self {
            renderer,
            world: World::new(),
            material,
            bounds: (0, 0),
            offset: (0, 0),
            chunk_offset: (0, 0),
            textures: HashMap::new(),
        };

        storage.update_bounds(surface);
        storage
    }

    fn update_bounds(&mut self, surface: &Arc<Surface<Window>>) {
        let window_bounds = surface
            .window()
            .inner_size()
            .to_logical::<u32>(surface.window().scale_factor());
        let chunk_size = 300u32;
        self.bounds = (
            window_bounds.width / chunk_size,
            window_bounds.height / chunk_size,
        );
    }
}

impl MainHandler for AppData {
    fn handle_event(
        &mut self,
        event: Event,
        mut input_state: EnumSet<InputState>,
    ) -> Action<State> {
        match event {
            Event::KeyboardInput { ref input, .. } => {
                let input_kind = match input.virtual_keycode {
                    Some(VirtualKeyCode::Up | VirtualKeyCode::W) => InputState::Up,
                    Some(VirtualKeyCode::Down | VirtualKeyCode::S) => InputState::Down,
                    Some(VirtualKeyCode::Right | VirtualKeyCode::D) => InputState::Right,
                    Some(VirtualKeyCode::Left | VirtualKeyCode::A) => InputState::Left,
                    _ => return Action::Continue,
                };

                if input.state == ElementState::Pressed {
                    input_state.insert(input_kind);
                } else {
                    input_state.remove(input_kind);
                }

                Action::Done(State::Main(input_state))
            }
            Event::Resized(..) => {
                let window = self.window().clone();
                self.data.update_bounds(&window);
                Action::Continue
            }

            Event::CloseRequested => Action::Quit,
            _ => Action::Continue,
        }
    }

    fn handle_tick(&mut self, input_state: EnumSet<InputState>) {
        let speed = 5;

        if input_state.contains(InputState::Up) {
            self.data.offset.1 += speed;
        }

        if input_state.contains(InputState::Down) {
            self.data.offset.1 -= speed;
        }

        if input_state.contains(InputState::Right) {
            self.data.offset.0 -= speed;
        }

        if input_state.contains(InputState::Left) {
            self.data.offset.0 += speed;
        }

        self.data.chunk_offset = (
            (self.data.offset.0 + 150) / 300 as i64,
            (self.data.offset.1 + 150) / 300 as i64,
        );

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

        for x in (0..=self.data.bounds.0 + 2).map(|x| x as i64 - 1) {
            for y in (0..=self.data.bounds.1 + 2).map(|y| y as i64 - 1) {
                let key = ChunkKey::new(
                    x as i64 - self.data.chunk_offset.0,
                    y as i64 - self.data.chunk_offset.1,
                );

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

    fn handle_render(&self, _: EnumSet<InputState>) {
        self.data.renderer.render(self.window(), |frame| {
            let mut frame = frame.material(&self.data.material);

            for x in (0..=self.data.bounds.0 + 2).map(|x| x as i64 - 1) {
                for y in (0..=self.data.bounds.1 + 2).map(|y| y as i64 - 1) {
                    let key = ChunkKey::new(
                        x as i64 - self.data.chunk_offset.0,
                        y as i64 - self.data.chunk_offset.1,
                    );
                    if let Some(&TextureEntry::Valid(ref texture)) = self.data.textures.get(&key) {
                        frame = frame.draw(
                            [
                                (self.data.offset.0 + key.x * 300) as f32,
                                (self.data.offset.1 + key.y * 300) as f32,
                            ]
                            .into(),
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
    .run(60, State::Main(EnumSet::empty()))
}
