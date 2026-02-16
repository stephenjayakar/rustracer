use egui_sdl2_gl::egui::{self, Context, RichText};

/// Available scenes that can be rendered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneType {
    Dragon,
    Teapot,
    Specular,
    Diffuse,
    Triangle,
}

impl SceneType {
    pub fn name(&self) -> &'static str {
        match self {
            SceneType::Dragon => "Dragon",
            SceneType::Teapot => "Teapot",
            SceneType::Specular => "Specular Spheres",
            SceneType::Diffuse => "Diffuse Spheres",
            SceneType::Triangle => "Simple Triangle",
        }
    }

    pub fn all() -> &'static [SceneType] {
        &[
            SceneType::Dragon,
            SceneType::Teapot,
            SceneType::Specular,
            SceneType::Diffuse,
            SceneType::Triangle,
        ]
    }
}

/// Render quality presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderQuality {
    Preview,
    Low,
    Medium,
    High,
    Ultra,
}

impl RenderQuality {
    pub fn name(&self) -> &'static str {
        match self {
            RenderQuality::Preview => "Preview (1 spp)",
            RenderQuality::Low => "Low (4 spp)",
            RenderQuality::Medium => "Medium (16 spp)",
            RenderQuality::High => "High (64 spp)",
            RenderQuality::Ultra => "Ultra (256 spp)",
        }
    }

    pub fn samples_per_pixel(&self) -> u32 {
        match self {
            RenderQuality::Preview => 1,
            RenderQuality::Low => 4,
            RenderQuality::Medium => 16,
            RenderQuality::High => 64,
            RenderQuality::Ultra => 256,
        }
    }

    pub fn light_samples(&self) -> u32 {
        match self {
            RenderQuality::Preview => 1,
            RenderQuality::Low => 4,
            RenderQuality::Medium => 8,
            RenderQuality::High => 16,
            RenderQuality::Ultra => 32,
        }
    }

    pub fn all() -> &'static [RenderQuality] {
        &[
            RenderQuality::Preview,
            RenderQuality::Low,
            RenderQuality::Medium,
            RenderQuality::High,
            RenderQuality::Ultra,
        ]
    }
}

/// Actions that the GUI wants to trigger
#[derive(Debug, Clone, PartialEq)]
pub enum GuiAction {
    None,
    ChangeScene(SceneType),
    StartFullRender,
    CancelRender,
    ToggleDebugMode,
    SaveImage,
    ResetCamera,
    UpdateRenderSettings {
        samples_per_pixel: u32,
        light_samples: u32,
        max_bounces: u32,
    },
}

/// State for the GUI
pub struct GuiState {
    pub selected_scene: SceneType,
    pub render_quality: RenderQuality,
    pub is_rendering: bool,
    pub render_progress: f32,
    pub is_debug_mode: bool,
    pub continuous_rendering: bool,

    // Custom render settings
    pub custom_samples_per_pixel: u32,
    pub custom_light_samples: u32,
    pub custom_max_bounces: u32,
    pub use_custom_settings: bool,

    // Camera info
    pub camera_x: f32,
    pub camera_y: f32,
    pub camera_z: f32,

    // UI state
    pub show_settings_panel: bool,
    pub show_help: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            selected_scene: SceneType::Dragon,
            render_quality: RenderQuality::Medium,
            is_rendering: false,
            render_progress: 0.0,
            is_debug_mode: true,
            continuous_rendering: true,

            custom_samples_per_pixel: 16,
            custom_light_samples: 8,
            custom_max_bounces: 50,
            use_custom_settings: false,

            camera_x: 0.0,
            camera_y: 0.0,
            camera_z: 0.0,

            show_settings_panel: true,
            show_help: false,
        }
    }
}

impl GuiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update camera position display
    pub fn update_camera(&mut self, x: f32, y: f32, z: f32) {
        self.camera_x = x;
        self.camera_y = y;
        self.camera_z = z;
    }

    /// Get the effective samples per pixel based on quality or custom settings
    pub fn effective_samples_per_pixel(&self) -> u32 {
        if self.use_custom_settings {
            self.custom_samples_per_pixel
        } else {
            self.render_quality.samples_per_pixel()
        }
    }

    /// Get the effective light samples based on quality or custom settings
    pub fn effective_light_samples(&self) -> u32 {
        if self.use_custom_settings {
            self.custom_light_samples
        } else {
            self.render_quality.light_samples()
        }
    }

    /// Render the GUI and return any action to perform
    pub fn render(&mut self, ctx: &Context) -> GuiAction {
        let mut action = GuiAction::None;

        // Top panel with main controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save Image").clicked() {
                        action = GuiAction::SaveImage;
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.show_settings_panel, "Settings Panel")
                        .clicked()
                    {
                        ui.close_menu();
                    }
                    if ui.checkbox(&mut self.show_help, "Help").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        // Side panel with controls
        if self.show_settings_panel {
            egui::SidePanel::left("settings_panel")
                .default_width(220.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Rustracer");
                    ui.separator();

                    // Scene selection
                    ui.label(RichText::new("Scene").strong());
                    egui::ComboBox::from_label("")
                        .selected_text(self.selected_scene.name())
                        .show_ui(ui, |ui| {
                            for scene in SceneType::all() {
                                if ui
                                    .selectable_value(
                                        &mut self.selected_scene,
                                        *scene,
                                        scene.name(),
                                    )
                                    .clicked()
                                {
                                    action = GuiAction::ChangeScene(*scene);
                                }
                            }
                        });

                    ui.add_space(10.0);
                    ui.separator();

                    // Render mode
                    ui.label(RichText::new("Render Mode").strong());
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.is_debug_mode, "Fast/Debug")
                            .clicked()
                            && !self.is_debug_mode
                        {
                            self.is_debug_mode = true;
                            action = GuiAction::ToggleDebugMode;
                        }
                        if ui
                            .selectable_label(!self.is_debug_mode, "Full Render")
                            .clicked()
                            && self.is_debug_mode
                        {
                            self.is_debug_mode = false;
                            action = GuiAction::ToggleDebugMode;
                        }
                    });

                    ui.add_space(5.0);
                    ui.checkbox(&mut self.continuous_rendering, "Continuous Update");

                    ui.add_space(10.0);
                    ui.separator();

                    // Quality settings
                    ui.label(RichText::new("Quality").strong());
                    egui::ComboBox::from_id_source("quality_combo")
                        .selected_text(self.render_quality.name())
                        .show_ui(ui, |ui| {
                            for quality in RenderQuality::all() {
                                ui.selectable_value(
                                    &mut self.render_quality,
                                    *quality,
                                    quality.name(),
                                );
                            }
                        });

                    ui.add_space(5.0);
                    ui.checkbox(&mut self.use_custom_settings, "Custom Settings");

                    if self.use_custom_settings {
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("Samples/pixel:");
                            ui.add(
                                egui::DragValue::new(&mut self.custom_samples_per_pixel)
                                    .clamp_range(1..=1024),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Light samples:");
                            ui.add(
                                egui::DragValue::new(&mut self.custom_light_samples)
                                    .clamp_range(1..=128),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Max bounces:");
                            ui.add(
                                egui::DragValue::new(&mut self.custom_max_bounces)
                                    .clamp_range(1..=100),
                            );
                        });
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // Render controls
                    ui.label(RichText::new("Actions").strong());

                    if self.is_rendering {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Rendering...");
                        });
                        ui.add(egui::ProgressBar::new(self.render_progress).show_percentage());
                        if ui.button("Cancel").clicked() {
                            action = GuiAction::CancelRender;
                        }
                    } else {
                        if ui.button("Start Full Render").clicked() {
                            action = GuiAction::StartFullRender;
                        }
                    }

                    ui.add_space(5.0);
                    if ui.button("Save Image").clicked() {
                        action = GuiAction::SaveImage;
                    }

                    ui.add_space(5.0);
                    if ui.button("Reset Camera").clicked() {
                        action = GuiAction::ResetCamera;
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // Camera info
                    ui.label(RichText::new("Camera Position").strong());
                    ui.label(format!("X: {:.2}", self.camera_x));
                    ui.label(format!("Y: {:.2}", self.camera_y));
                    ui.label(format!("Z: {:.2}", self.camera_z));

                    ui.add_space(10.0);
                    ui.separator();

                    // Keyboard shortcuts
                    ui.collapsing("Keyboard Shortcuts", |ui| {
                        ui.label("W/A/S/D - Move camera");
                        ui.label("Q/E - Move up/down");
                        ui.label("R - Toggle render mode");
                        ui.label("C - Toggle continuous");
                        ui.label("F - Start full render");
                        ui.label("Esc - Quit");
                    });
                });
        }

        // Help window
        if self.show_help {
            egui::Window::new("Help")
                .open(&mut self.show_help)
                .show(ctx, |ui| {
                    ui.heading("Rustracer - Path Tracer");
                    ui.add_space(10.0);

                    ui.label("Navigation:");
                    ui.label("  W/A/S/D - Move camera horizontally");
                    ui.label("  Q/E - Move camera up/down");
                    ui.add_space(10.0);

                    ui.label("Rendering:");
                    ui.label("  R - Toggle Debug/Full render mode");
                    ui.label("  F - Start a full quality render");
                    ui.label("  C - Toggle continuous rendering");
                    ui.add_space(10.0);

                    ui.label("Modes:");
                    ui.label("  Debug: Fast preview for navigation");
                    ui.label("  Full: High quality path tracing");
                });
        }

        action
    }
}
