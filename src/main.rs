#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod queue_generator;


use std::{fs, thread, sync::{Arc, Mutex}, time::Duration};

use eframe::{egui, IconData};
use egui::{Vec2, RichText, Color32, Sense, Context};
use queue_generator::ranked_queue_playlist_generator::PlaylistMaker;
use queue_playlist_maker::lock;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2{x: 320.0, y: 400.0}),
        resizable: false,
        centered: true,
        decorated: false,
        transparent: true,
        icon_data: Some(IconData::try_from_png_bytes(include_bytes!("../queue.png")).unwrap()),
        ..Default::default()
    };
    eframe::run_native("Queue Playlist Maker", options, Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::style::Visuals::dark());
            Box::new(PlaylistMakerUI::default(cc.egui_ctx.clone()))
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GenStatus {
    IDLE,
    GENERATING,
    SUCCESS,
    FAILED
}

struct PlaylistMakerUI {
    app_data: Arc<Mutex<AppData>>
}

#[derive(Debug, Clone)]
pub struct AppData {
    title: String,
    author: String,
    image_path: String,
    output_path: String,
    description: String,
    process_amount: f32,
    progress: f32,
    gen_status: GenStatus,
    ctx: Context
}

impl AppData {
    fn default(ctx: Context) -> Self {
        Self {
            title: "".to_owned(),
            author: "".to_owned(),
            image_path: "".to_owned(),
            output_path: "".to_owned(),
            description: "playlist of ranked queue maps".to_owned(),
            process_amount: 0.0,
            progress: 0.0,
            gen_status: GenStatus::IDLE,
            ctx: ctx
        }
    }
}

impl PlaylistMakerUI {
    fn default(ctx: Context) -> Self {
        Self {
            app_data: Arc::new(Mutex::new(AppData::default(ctx)))
        }
    }
}

impl eframe::App for PlaylistMakerUI {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        custom_window_frame(ctx, "Queue Playlist Maker", _frame, |ui| {
            ui.spacing_mut().text_edit_width = 150.0;
            ui.spacing_mut().item_spacing = Vec2{x: 5.0, y: 10.0};
            
            let mut app_data = lock!(self.app_data);

            ui.horizontal(|ui| {
                let name_label = ui.label("Playlist Title: ");
                egui::TextEdit::singleline(&mut app_data.title).hint_text("title").show(ui).response.labelled_by(name_label.id);
            });
            ui.horizontal(|ui| {
                let name_label = ui.label("Playlist Author: ");
                egui::TextEdit::singleline(&mut app_data.author).hint_text("author").show(ui).response.labelled_by(name_label.id);
            });
            ui.horizontal(|ui| {
                let name_label = ui.label("Playlist Image: ");
                egui::TextEdit::singleline(&mut app_data.image_path).hint_text("image path").show(ui).response.labelled_by(name_label.id);
            });
            ui.horizontal(|ui| {
                let name_label = ui.label("Output File Path: ");
                egui::TextEdit::singleline(&mut app_data.output_path).hint_text("./playlist.json").show(ui).response.labelled_by(name_label.id);
            });
            ui.horizontal(|ui| {
                let name_label = ui.label("Playlist Description: ");
                ui.text_edit_multiline(&mut app_data.description).labelled_by(name_label.id);
            });
            ui.vertical_centered(|ui| {
                let sense = if app_data.progress > 0.0 {Sense::focusable_noninteractive()} else {Sense::click()};
                let button = ui.add_sized([120.0, 40.0], egui::Button::new(RichText::new("Make Playlist").size(15.0)).sense(sense));
                if button.clicked() {
                    make_playlist_async(self.app_data.clone());
                }
                let progress = app_data.progress;
                let status = app_data.gen_status;
                let format = get_progression_message(progress as i32, &status);
                let colour = get_progress_colour(progress, &status);
                ui.label(RichText::new(format).color(colour));

                ui.image(egui::include_image!("../queue.png"));
            });
            drop(app_data);
        });
    }
}

fn get_progress_colour(progress: f32, status: &GenStatus) -> Color32 {
    let alpha = if *status == GenStatus::IDLE {0} else {255};
    let amount = match status {
        GenStatus::SUCCESS => 255,
        GenStatus::FAILED | GenStatus::IDLE => 0,
        GenStatus::GENERATING => (progress / 100.0 * 255.0).clamp(0.0, 255.0) as u8
    };
    Color32::from_rgba_unmultiplied(255-amount, amount, 0, alpha)
}

fn make_playlist_async(data_ptr: Arc<Mutex<AppData>>) {
    thread::spawn(move || {
        let playlist = PlaylistMaker::new(data_ptr.clone()).make_playlist().serialise();
        let output_path = get_file_path(lock!(data_ptr).output_path.clone());
        match fs::write(&output_path, playlist) {
            Ok(_) => run_completion_message(data_ptr, GenStatus::SUCCESS),
            Err(_) => run_completion_message(data_ptr, GenStatus::FAILED)
        };
    });
}

fn get_file_path(output_path: String) -> String {
    match output_path.trim() {
        "" => "playlist.json".to_string(),
        path => if !path.ends_with(".json") {format!("{path}.json")} else {path.to_string()}
    }
}

fn get_progression_message(progress: i32, status: &GenStatus) -> String {
    match status {
        GenStatus::FAILED => "Failed to write to file!".to_string(),
        GenStatus::SUCCESS => "Generation Complete!".to_string(),
        _ => format!("{}%", progress)
    }
}

fn run_completion_message(data_ptr: Arc<Mutex<AppData>>, status: GenStatus) {
    lock!(data_ptr).gen_status = status;
    lock!(data_ptr).ctx.request_repaint();
    thread::sleep(Duration::from_millis(700));
    lock!(data_ptr).progress = 0.0;
    lock!(data_ptr).gen_status = GenStatus::IDLE;
    lock!(data_ptr).ctx.request_repaint();
}

fn custom_window_frame(ctx: &egui::Context, title: &str, frame: &mut eframe::Frame, add_contents: impl FnOnce(&mut egui::Ui)) {
    use egui::*;

    let panel_frame = egui::Frame {
        fill: ctx.style().visuals.window_fill(),
        rounding: 10.0.into(),
        stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
        outer_margin: 0.5.into(), // so the stroke is within the bounds
        ..Default::default()
    };

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let title_bar_height = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };
        title_bar_ui(ui, frame, title_bar_rect, title);

        // Add the contents:
        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        }
        .shrink(4.0);
        let mut content_ui = ui.child_ui(content_rect, *ui.layout());
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(ui: &mut egui::Ui, frame: &mut eframe::Frame, title_bar_rect: eframe::epaint::Rect, title: &str) {
    use egui::*;
    let painter = ui.painter();
    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());

    // Paint the title:
    painter.text(title_bar_rect.center(), Align2::CENTER_CENTER, title, FontId::proportional(20.0), ui.style().visuals.text_color());

    // Paint the line under the title:
    painter.line_segment(
        [title_bar_rect.left_bottom() + vec2(1.0, 0.0), title_bar_rect.right_bottom() + vec2(-1.0, 0.0)],
        ui.visuals().widgets.noninteractive.bg_stroke
    );

    if title_bar_response.is_pointer_button_down_on() { frame.drag_window(); }

    ui.allocate_ui_at_rect(title_bar_rect, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_maximize_minimize(ui, frame);
        });
    });
}

/// Show some close/maximize/minimize buttons for the native window.
fn close_maximize_minimize(ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    use egui::Button;

    let button_height = 12.0;
    ui.spacing_mut().item_spacing = Vec2{x: 10.0, y: 0.0};
    let close_response = ui.add(Button::new(RichText::new("❌").size(button_height))).on_hover_text("Close the window");
    if close_response.clicked() { frame.close(); }

    let minimized_response = ui.add(Button::new(RichText::new("🗕").size(button_height))).on_hover_text("Minimize the window");
    if minimized_response.clicked() { frame.set_minimized(true); }
}