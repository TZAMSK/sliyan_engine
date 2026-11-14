mod app;
pub mod editor;
mod physics;
mod renderer;
mod scene;
pub mod selection;
mod ui;

use winit::event_loop::EventLoop;

use crate::app::Application;

fn main() {
    //println!("Hello, world!");
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = Application::new().await;
    event_loop.run_app(&mut app).unwrap();
}
