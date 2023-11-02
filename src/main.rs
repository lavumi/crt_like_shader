use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::buffer::TileRenderData;
use crate::renderer::Renderer;

mod renderer;
mod config;
mod buffer;


fn main(){
    pollster::block_on(start());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
async fn start() {
    let title = "wgpu_wasm";
    let width = config::SCREEN_SIZE[0];
    let height = config::SCREEN_SIZE[1];

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        }
    }


    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width,height))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use crate::game_configs;
        window.set_inner_size(PhysicalSize::new(config::SCREEN_SIZE[0], config::SCREEN_SIZE[1]));
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wgpu")?;
                let canvas = web_sys::Element::from(window.canvas());
                canvas.set_id("wasm-canvas");
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    event_loop.set_control_flow(ControlFlow::Poll);


    let mut renderer = Renderer::new(&window).await;

    renderer.set_texture(include_bytes!("../assets/chr.png"));




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
                renderer.render().expect("TODO: panic message");
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


