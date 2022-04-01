use egui::{Align2, ClippedMesh, Color32, Grid, TexturesDelta};
use egui_wgpu_backend::{BackendError, RenderPass, ScreenDescriptor};
use pixels::wgpu;
use winit::window::Window;

use crate::{chip8::Chip8, emu::Emu};
struct Gui {
    show_run_controls: bool,
    show_cpu_state: bool,
    show_memory: bool,
    show_gfx: bool,
}

impl Gui {
    fn new() -> Self {
        Self {
            show_run_controls: true,
            show_cpu_state: true,
            show_memory: true,
            show_gfx: true,
        }
    }

    fn ui(&mut self, ctx: &egui::Context, emu: &mut Emu) {
        egui::Window::new("Run Controls")
            .open(&mut self.show_run_controls)
            .anchor(Align2::CENTER_TOP, [0.0, 0.0])
            .show(ctx, |ui| {
                Grid::new("info").show(ui, |ui| {
                    ui.label("Status");
                    if emu.run_steps {
                        ui.colored_label(Color32::YELLOW, "PAUSED");
                    } else {
                        ui.colored_label(Color32::GREEN, "RUNNING");
                    }
                    ui.end_row();
                    ui.label("Clock Rate");
                    ui.label(format!("{}", emu.clock_rate));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Run").clicked() {
                        emu.run_steps = false;
                    }
                    if ui.button("Pause").clicked() {
                        emu.run_steps = true;
                    }
                    ui.separator();
                    if ui.button("Step").clicked() {
                        emu.progress();
                    }
                });
            });

        egui::Window::new("CPU State")
            .open(&mut self.show_cpu_state)
            .anchor(Align2::LEFT_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Grid::new("register_grid").show(ui, |ui| {
                    ui.label("Program Counter");
                    ui.label(format!("{:04x}", emu.cpu.pc));

                    ui.end_row();

                    ui.label("Stack Pointer");
                    ui.label(format!("{:04x}", emu.cpu.sp));

                    ui.end_row();

                    ui.label("Index Register");
                    ui.label(format!("{:04x}", emu.cpu.I));

                    ui.end_row();

                    ui.label("Delay Timer");
                    ui.label(format!("{}", emu.cpu.delay_timer));

                    ui.end_row();

                    ui.label("Sound Timer");
                    ui.label(format!("{}", emu.cpu.sound_timer));

                    ui.end_row();
                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    ui.label("Next opcode");
                    ui.label(format!("{:04x}", &emu.cpu.get_opcode()));

                    ui.end_row();

                    ui.label("Next Instruction");
                    ui.label(Chip8::decode_instruction(&emu.cpu.get_opcode()));

                    ui.end_row();
                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    ui.label("V Registers");
                    egui::Grid::new("v_register").striped(true).show(ui, |ui| {
                        for (i, v) in emu.cpu.V.into_iter().enumerate() {
                            ui.label(format!("0x{:01X}", i));
                            ui.label(format!("{:02x}", v));
                            if i % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    });

                    ui.end_row();
                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    ui.label("Stack");
                    egui::Grid::new("v_register").striped(true).show(ui, |ui| {
                        for (i, v) in emu.cpu.stack.into_iter().enumerate() {
                            ui.label(format!("0x{:01X}", i));
                            ui.label(format!("{:04X}", v));
                            if i % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    });
                    ui.end_row();
                });
            });

        egui::Window::new("Memory")
            .anchor(Align2::RIGHT_TOP, [-2.0, 0.0])
            .open(&mut self.show_memory)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("memory_view").striped(true).show(ui, |ui| {
                        for (row, chunk) in emu.cpu.memory.chunks(8).enumerate() {
                            ui.label(format!("{:04X}", row * 8));
                            for byte in chunk {
                                ui.label(format!("{:02x}", byte));
                            }
                            ui.end_row();
                        }
                    });
                });
            });

        egui::Window::new("GFX")
            .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
            .open(&mut self.show_gfx)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for v in emu.cpu.gfx.chunks(64) {
                        let contents = v
                            .iter()
                            .map(|b| if *b { "*" } else { "  " })
                            .collect::<Vec<_>>()
                            .join("");
                        ui.label(contents);
                    }
                });
            });
    }
}

pub(crate) struct Framework {
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedMesh>,
    textures: TexturesDelta,

    gui: Gui,
}

impl Framework {
    pub(crate) fn new(width: u32, height: u32, scale_factor: f32, pixels: &pixels::Pixels) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::from_pixels_per_point(max_texture_size, scale_factor);
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
        let rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);
        let textures = TexturesDelta::default();

        let gui = Gui::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: vec![],
            textures,
            gui,
        }
    }

    pub(crate) fn handle_events(&mut self, event: &winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.physical_width = width;
            self.screen_descriptor.physical_height = height;
        }
    }

    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.scale_factor = scale_factor as f32;
    }

    pub(crate) fn prepare(&mut self, window: &Window, data: &mut Emu) {
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            self.gui.ui(egui_ctx, data);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &pixels::PixelsContext,
    ) -> Result<(), BackendError> {
        self.rpass
            .add_textures(&context.device, &context.queue, &self.textures)?;
        self.rpass.update_buffers(
            &context.device,
            &context.queue,
            &self.paint_jobs,
            &self.screen_descriptor,
        );
        self.rpass.execute(
            encoder,
            render_target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        )?;

        let textures = std::mem::take(&mut self.textures);
        self.rpass.remove_textures(textures)?;
        Ok(())
    }
}
