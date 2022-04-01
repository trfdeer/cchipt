use std::time::Instant;

use color_eyre::{eyre::eyre, Result};
use emu::{Emu, KEYS, REFRESH_RATE, WINDOW_HEIGHT, WINDOW_WIDTH};
use gui::Framework;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowBuilderExtWindows,
    window::{Theme, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

mod chip8;
mod emu;
mod gui;

fn main() -> Result<()> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = WindowBuilder::new()
        .with_theme(Some(Theme::Dark))
        .with_title("cchipt")
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
        .with_min_inner_size(LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64))
        .with_maximized(true)
        .build(&event_loop)?;

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WINDOW_WIDTH, WINDOW_HEIGHT, surface_texture)?;
        let framework =
            Framework::new(window_size.width, window_size.height, scale_factor, &pixels);
        (pixels, framework)
    };

    let mut emu = Emu::default();
    emu.load_rom(&std::env::args().nth(1).unwrap())?;

    event_loop.run(move |event, _, control_flow| {
        let frame_start_time = Instant::now();
        if input.update(&event) {
            if input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                framework.resize(size.width, size.height);
            }

            let mut new_keystate = [false; 16];
            for (i, key) in KEYS.iter().enumerate() {
                new_keystate[i] = input.key_pressed(*key);
            }
            emu.update_keystates(new_keystate);

            // if emu.run_steps {
            //     if input.key_pressed(VirtualKeyCode::S) {
            //         emu.progress();
            //     }
            // } else {
            //     for _ in 0..(emu.clock_rate / REFRESH_RATE) {
            //         emu.progress();
            //     }
            // }
        }
        if !emu.run_steps {
            for _ in 0..(emu.clock_rate / REFRESH_RATE) {
                emu.progress();
            }
        }
        window.request_redraw();

        match event {
            Event::WindowEvent { event, .. } => {
                framework.handle_events(&event);
            }
            Event::RedrawRequested(_) => {
                emu.draw(pixels.get_frame());
                framework.prepare(&window, &mut emu);
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    context.scaling_renderer.render(encoder, render_target);
                    framework.render(encoder, render_target, context)?;
                    Ok(())
                });
                if render_result
                    .map_err(|e| eyre!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }

        let elapsed_time = Instant::now().duration_since(frame_start_time).as_millis() as u64;
        let wait_millis = match 1000 / REFRESH_RATE >= elapsed_time {
            true => 1000 / REFRESH_RATE - elapsed_time,
            false => 0,
        };
        let new_inst = frame_start_time + std::time::Duration::from_millis(wait_millis);
        *control_flow = ControlFlow::WaitUntil(new_inst);
    });
}
