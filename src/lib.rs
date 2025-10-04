
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


use std::sync::{Arc, Mutex};
use winit::dpi::*;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::config::GameConfig;
use crate::renderer::Renderer;

mod renderer;
mod config;
mod buffer;
mod resources;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {

    let config = GameConfig::new().await;


    let width = config.options.screen_size[0];
    let height = config.options.screen_size[1];

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        }
    }


    let event_loop = EventLoop::new().unwrap();


    let builder = WindowBuilder::new()
        .with_title("rust window")
        .with_inner_size(LogicalSize::new(width,height));

    #[cfg(target_arch = "wasm32")]
        let builder = {
        use winit::platform::web::WindowBuilderExtWebSys;
        builder.with_append(true)
            .with_inner_size(PhysicalSize::new(width,height))
    };
    let window = Arc::new(builder.build(&event_loop).unwrap());


    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        let canvas = window.canvas().unwrap();
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wgpu")?;
                let canvas = web_sys::Element::from(canvas);
                canvas.set_id("wasm-canvas");
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    event_loop.set_control_flow(ControlFlow::Poll);




    let renderer = Arc::new(Mutex::new(Renderer::new(window.clone(), &config).await));

    let res = resources::load_binary("../res/chr.png").await.unwrap();

    {
        let mut renderer = renderer.lock().unwrap();
        renderer.set_texture(&res);
        renderer.init_instances();
    }



    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                elwt.exit();
            },
            Event::AboutToWait => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.
                renderer.lock().unwrap().render().expect("TODO: panic message");
                window.request_redraw();
            },
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.
            },
            _ => ()
        }
    }).expect("TODO: panic message");
}


