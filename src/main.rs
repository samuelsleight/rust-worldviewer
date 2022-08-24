use renderer::Renderer;
use stateloop::{
    app::{App, Data, Event, Window},
    state::Action,
    states,
};
use std::sync::Arc;
use vulkano::{
    instance::{Instance, InstanceCreateInfo},
    swapchain::Surface,
};

mod renderer;

states! {
    State {
        MainHandler Main()
    }
}

struct Storage {
    renderer: Renderer,
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

    fn handle_render(&self) {}
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
        move |surface| Storage {
            renderer: Renderer::init_vulkan(instance, surface),
        },
    )
    .expect("Unable to initialise application")
    .run(60, State::Main())
}
