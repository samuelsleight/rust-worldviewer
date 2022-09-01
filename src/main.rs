use renderer::{InitError, Renderer};
use stateloop::{
    app::{App, Data, Event, Window},
    state::Action,
    states,
};
use std::sync::Arc;
use vulkano::{
    format::Format,
    image::ImageViewAbstract,
    instance::{Instance, InstanceCreateInfo},
    swapchain::Surface,
};
use world::World;

mod renderer;
mod world;

states! {
    State {
        MainHandler Main()
    }
}

struct Storage {
    renderer: Renderer,
    textures: Vec<Arc<dyn ImageViewAbstract>>,
}

type AppData = Data<Storage, Arc<Surface<Window>>>;

impl MainHandler for AppData {
    fn handle_event(&mut self, event: Event) -> Action<State> {
        match event {
            Event::CloseRequested => Action::Quit,
            _ => Action::Continue,
        }
    }

    fn handle_tick(&mut self) {}

    fn handle_render(&self) {
        self.data.renderer.render(self.window(), |frame| {
            frame
                .draw([200, 100].into(), self.data.textures[0].clone())
                .draw([500, 100].into(), self.data.textures[1].clone())
                .draw([500, 400].into(), self.data.textures[2].clone())
                .finish()
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

            let world = World::new();

            let textures = vec![
                renderer.create_texture(
                    world.generate_chunk_texture(-1, 0),
                    512,
                    512,
                    Format::R8G8B8A8_SRGB,
                ),
                renderer.create_texture(
                    world.generate_chunk_texture(0, 0),
                    512,
                    512,
                    Format::R8G8B8A8_SRGB,
                ),
                renderer.create_texture(
                    world.generate_chunk_texture(0, 1),
                    512,
                    512,
                    Format::R8G8B8A8_SRGB,
                ),
            ];
            Ok(Storage { renderer, textures })
        },
    )
    .expect("Unable to initialise application")
    .run(60, State::Main())
}
