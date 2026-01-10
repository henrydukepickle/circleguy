use crate::DEFAULT_PUZZLE;
use crate::hps::data_storer::data_storer::DataStorer;
use crate::puzzle::puzzle::*;
use crate::ui::render::draw_circle;
use egui::*;

///default scale factor
const SCALE_FACTOR: f32 = 500.0;
///default animation speed
const ANIMATION_SPEED: f64 = 5.0;
///credits string
const CREDITS: &str = "Created by Henry Pickle,
with major help from:
Luna Harran (sonicpineapple)
Andrew Farkas (HactarCE)
cryofractal";

#[derive(Debug)]
///used for running the app. contains all puzzle and view data at runtime
pub struct App {
    data_storer: DataStorer, //stores the data for the puzzles (on the right panel)
    puzzle: Puzzle,          //stores the puzzle
    log_path: String,        //stores the path log files are loaded from/saved to
    curr_msg: String,        //current message (usually for errors)
    animation_speed: f64,    //speed at which animations happen
    last_frame_time: web_time::Instant, //the absolute time at which the last frame happened
    outline_width: f32,      //the width of the outlines
    scale_factor: f32,       //the scale factor (zoom)
    offset: Vec2,            //the offset of the puzzle from the center of the screen (pan)
    cut_on_turn: bool,       //whether or not turns should cut the puzzle
    preview: bool,           //whether the solved state is being previewed
}
impl App {
    ///initialize a new app, using some default settings (from the constants)
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut data_storer = DataStorer::new(false).unwrap(); //initialize a new data storer
        data_storer
            .load_puzzles(
                "Puzzles/Definitions/",
                //"Configs/Keybinds/Puzzles/",
                //"Configs/Keybinds/groups.kdl",
            )
            .unwrap();
        let _ = data_storer.load_keybinds("Configs/keybinds.kdl");
        let p_data = &data_storer
            .puzzles
            .lock()
            .unwrap()
            .get(DEFAULT_PUZZLE)
            .unwrap()
            .clone();
        let p = p_data
            .load(
                &mut data_storer.rt,
                data_storer.keybinds.get_keybinds_for_puzzle(&p_data.name),
            )
            .unwrap();
        Self {
            //return the default app
            data_storer,
            puzzle: Puzzle::new(p),
            log_path: String::from("logfile"),
            curr_msg: String::new(),
            animation_speed: ANIMATION_SPEED,
            last_frame_time: web_time::Instant::now(),
            outline_width: 5.0,
            scale_factor: SCALE_FACTOR,
            offset: vec2(0.0, 0.0),
            cut_on_turn: false,
            preview: false,
            // keybinds: if let Some(kb) = &p_data.keybinds
            //     && let Some(gr) = &p_data.keybind_groups
            //     && let Some(keybinds) = load_keybinds(&kb, &gr)
            // {
            //     Some(keybinds.clone())
            // } else {
            //     None
            // },
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //run the ui of the program on a central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap(); //the space the program has to work with
            if !self.preview {
                //if the puzzle isnt being previewed, render it
                if let Err(x) = self.puzzle.render(
                    ui,
                    &rect,
                    self.outline_width,
                    self.scale_factor,
                    self.offset,
                ) {
                    self.curr_msg = x;
                };
                //if the puzzle is in preview mode, render all of the pieces of the solved state
            } else {
                for piece in &self.puzzle.solved_state {
                    if let Err(x) = piece.render(
                        ui,
                        &rect,
                        None,
                        self.outline_width,
                        self.scale_factor,
                        self.offset,
                    ) {
                        self.curr_msg = x;
                    }
                }
            }
            //render the data storer panel -- this stores all of the puzzles that you can load
            match self.data_storer.render_panel(ctx) {
                Err(()) => {
                    self.curr_msg =
                        String::from("Failed to render side panel or failed to create puzzle!")
                }
                Ok(Some(puzzle_data)) => {
                    //if a puzzle is returned (a button is clicked), load it
                    match puzzle_data.load(
                        &mut self.data_storer.rt,
                        self.data_storer
                            .keybinds
                            .get_keybinds_for_puzzle(&puzzle_data.name),
                    ) {
                        Ok(puz_data) => self.puzzle = Puzzle::new(puz_data),
                        Err(diag) => self.curr_msg = diag.msg.to_string(),
                    }
                    // if let Some(kb) = puzzle_data.keybinds
                    //     && let Some(gr) = puzzle_data.keybind_groups
                    //     && let Some(keybinds) = load_keybinds(&kb, &gr)
                    // {
                    //     self.keybinds = Some(keybinds);
                    // } else {
                    //     self.keybinds = None;
                    // }
                }
                _ => {}
            }
            let delta_time = self.last_frame_time.elapsed(); //the time since the last frame
            self.last_frame_time = web_time::Instant::now(); //reset the time tracker
            if self.puzzle.anim_left >= 0.0 {
                //if the animation is still running, advance it according to delta_time and the animation speed
                self.puzzle.anim_left = f32::max(
                    self.puzzle.anim_left
                        - (delta_time.as_secs_f32() * self.animation_speed as f32),
                    0.0,
                );
            }
            if 24.9 < self.animation_speed {
                //if the animation speed is fast enough, remove animations entirely
                self.puzzle.animation_offset = None;
            }
            //UI Section: menu bar
            egui::MenuBar::new().ui(ui, |ui| {
                //file menu controls save/loading logs
                let file_button = default_menu_button("File");
                file_button.ui(ui, |ui| {
                    //field for adding log path
                    ui.label("Log File Path");
                    ui.add(egui::TextEdit::singleline(&mut self.log_path));
                    //saving, does not work on web
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.add(egui::Button::new("SAVE")).clicked() {
                        self.curr_msg = match self.data_storer.save(&self.log_path, &self.puzzle) {
                            Ok(()) => String::from("Saved successfully!"),
                            Err(err) => err.to_string(),
                        }
                    }
                    // //loading, does not work on web
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.add(egui::Button::new("LOAD LOG")).clicked() {
                        self.puzzle = self
                            .data_storer
                            .load_save(&self.log_path)
                            .unwrap_or(self.puzzle.clone());
                    }
                });
                //view menu controls view graphics
                let view_button = default_menu_button("View");
                view_button.ui(ui, |ui| {
                    //outline width slider
                    ui.add(
                        egui::Slider::new(&mut self.outline_width, (0.0)..=10.0)
                            .text("Outline Width"),
                    );
                    //animation speed slider
                    ui.add(
                        egui::Slider::new(&mut self.animation_speed, (1.0)..=25.0)
                            .text("Animation Speed"),
                    );
                    //rending size (zoom) slider
                    ui.add(
                        egui::Slider::new(&mut self.scale_factor, (10.0)..=5000.0)
                            .text("Rendering Size"),
                    );
                    //panning sliders
                    ui.add(egui::Slider::new(&mut self.offset.y, (-2.0)..=2.0).text("Move Y"));
                    ui.add(egui::Slider::new(&mut self.offset.x, (-2.0)..=2.0).text("Move X"));
                    //preview solved state toggle
                    ui.checkbox(&mut self.preview, "Preview solved state?");
                    //cut on turn toggle
                    //reset view button
                    if ui.add(egui::Button::new("Reset View")).clicked() {
                        (self.scale_factor, self.offset) = (SCALE_FACTOR, vec2(0.0, 0.0))
                    }
                });
                //scramble menu controls scrambling
                let scramble_button = default_menu_button("Scramble");
                scramble_button.ui(ui, |ui| {
                    //scramble button
                    if ui.add(egui::Button::new("Scramble")).clicked() && !self.preview {
                        let _ = self.puzzle.scramble(self.cut_on_turn);
                    }
                    //reset button
                    if ui.add(egui::Button::new("Reset")).clicked()
                        && !self.preview
                        && self.puzzle.reset().is_err()
                    {
                        self.curr_msg = String::from("Reset failed!")
                    };
                });
                //puzzle menu controls puzzle operations
                let puzzle_button = default_menu_button("Puzzle");
                puzzle_button.ui(ui, |ui| {
                    //undo button, also performed using the z key
                    if (ui.add(egui::Button::new("Undo Move")).clicked()) && !self.preview {
                        let _ = self.puzzle.undo();
                    }
                    ui.checkbox(&mut self.cut_on_turn, "Cut on turn?");
                    if ui.add(egui::Button::new("Check Solved")).clicked() {
                        self.puzzle.check();
                    }
                });
                //credits menu displays credits (bugged?)
                let credits_button = default_menu_button("Credits");
                credits_button.ui(ui, |ui| {
                    //display the credits
                    ui.label(CREDITS);
                    ui.separator();
                    //display the puzzle contributors
                    // ui.label(RichText::new("Top puzzle contributors:").color(egui::Color32::WHITE));
                    // match self
                    //     .data_storer
                    //     .get_top_authors::<{ crate::ui::data_storer::TOP }>()
                    // {
                    //     //add the labels for the top 5 puzzle contributors
                    //     Ok(top) => {
                    //         for t in top {
                    //             ui.label(format!("{}: {}", t.0, t.1));
                    //         }
                    //     }
                    //     Err(_) => {}
                    // }
                });
            });
            //UI Section: display puzzle info
            Window::new("Puzzle Info")
                .default_pos((10.0, 40.0))
                .auto_sized()
                .show(ctx, |ui| {
                    ui.label(String::from("Name: ") + &self.puzzle.name);
                    ui.label(String::from("Authors: ") + &self.puzzle.authors.join(", "));
                    ui.label(self.puzzle.pieces.len().to_string() + " pieces");
                });
            //UI Section: Bottom left area
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::Frame::popup(ui.style())
                    .stroke(Stroke::NONE)
                    .shadow(Shadow::NONE)
                    .show(ui, |ui| {
                        ui.set_max_width(200.0);
                        ui.separator();
                        //displays move count
                        ui.label(self.puzzle.stack.len().to_string() + " ETM");
                        //if the puzzle is solved, display as much (this is currently not working)
                        if self.puzzle.solved {
                            ui.label("Solved!");
                        }
                        //display the current message if it isn't empty
                        if !self.curr_msg.is_empty() {
                            ui.label(&self.curr_msg);
                        }
                    });
            });
            //gets the rect for interaction with the puzzle (so that ui elements like buttons dont conflict with puzzle input)
            let cor_rect = Rect {
                min: pos2(180.0, 30.0),
                max: pos2(rect.width() - 180.0, rect.height()),
            };
            //if the puzzle is currently turning, request a repaint so the animation runs
            if self.puzzle.anim_left != 0.0 {
                ui.ctx().request_repaint();
            }
            //get the interactor
            let r = ui.interact(cor_rect, egui::Id::new(19), egui::Sense::all());
            //read scroll input and parse the sign
            let scroll = ui.input(|input| {
                input
                    .raw
                    .events
                    .iter()
                    .filter_map(|ev| match ev {
                        Event::MouseWheel {
                            unit: MouseWheelUnit::Line | MouseWheelUnit::Page,
                            delta,
                            modifiers: _,
                        } => Some((delta.x + delta.y).signum() as i32),
                        _ => None,
                    })
                    .sum::<i32>()
            });
            //if the puzzle is clicked and not in preview mode
            if r.clicked()
                && !self.preview
                && let Some(pointer) = r.interact_pointer_pos()
            {
                //process the input
                if let Err(x) = self.puzzle.process_click(
                    &rect,
                    pointer,
                    true,
                    self.scale_factor,
                    self.offset,
                    self.cut_on_turn,
                ) {
                    self.curr_msg = x;
                }
            }
            //the same input parsing but for the right click
            if r.clicked_by(egui::PointerButton::Secondary)
                && !self.preview
                && let Some(pointer) = r.interact_pointer_pos()
                && let Err(x) = self.puzzle.process_click(
                    &rect,
                    pointer,
                    false,
                    self.scale_factor,
                    self.offset,
                    self.cut_on_turn,
                )
            {
                self.curr_msg = x;
            }
            if ui.input(|i: &InputState| i.key_pressed(egui::Key::Z)) {
                let _ = self.puzzle.undo();
            }
            //keybinds
            if ui.ctx().memory(|x| x.focused().is_none()) {
                let ev = ctx.input(|i| i.events.clone());
                for event in ev {
                    if let Event::Key {
                        key,
                        physical_key,
                        pressed,
                        repeat: _,
                        modifiers: _,
                    } = event
                    {
                        let b = if let Some(p) = physical_key { p } else { key };
                        if pressed
                            && let Some((t, m)) = self.puzzle.keybinds.get(&b).cloned()
                            && self.puzzle.turns.contains_key(&t)
                        {
                            if let Err(x) = self.puzzle.turn_id(&t, self.cut_on_turn, m) {
                                self.curr_msg = x;
                            }
                        }
                    }
                }
            }
            //parse hovering. theres some casework here
            if r.hover_pos().is_some()
                && !self.preview
                && let Some(pointer) = r.hover_pos()
            {
                let hovered_circle =
                    self.puzzle
                        .get_hovered(&rect, pointer, self.scale_factor, self.offset);
                //get the hovered circle (turn circle)
                if let Err(x) = &hovered_circle {
                    self.curr_msg = x.clone();
                }
                if let Ok(Some(real_circle)) = hovered_circle {
                    draw_circle(real_circle, ui, &rect, self.scale_factor, self.offset);
                } //if a circle is hovered, highlight its border
                //if a circle is being hovered and the scroll wheel is being used, parse the scroll like a click
                //if the middle mouse button is pressed, or the control button is pressed, dont parse this input as these are camera commands
                if scroll != 0
                    && !r.dragged_by(egui::PointerButton::Middle)
                    && !ui.input(|i| i.modifiers.command_only())
                    && !self.preview
                    && let Some(pointer) = r.hover_pos()
                    && let Err(x) = self.puzzle.process_click(
                        &rect,
                        pointer,
                        scroll > 0,
                        self.scale_factor,
                        self.offset,
                        self.cut_on_turn,
                    )
                {
                    self.curr_msg = x;
                }
            }
            //if the middle mouse button is being pressed, pan the camera
            if r.dragged_by(egui::PointerButton::Middle) {
                let delta = r.drag_delta();
                let good_delta = vec2(delta.x / self.scale_factor, -(delta.y / self.scale_factor));
                self.offset += good_delta;
            }
            //if ctrl scrolling, zoom
            if ui.input(|i| i.modifiers.command_only()) && scroll != 0 {
                self.scale_factor += 10.0 * scroll as f32;
            }
        });
    }
}

//make a menu button that isn't trash
fn default_menu_button<'a>(text: &'a str) -> egui::containers::menu::MenuButton<'a> {
    let button = egui::containers::menu::MenuButton::new(text);
    let config = egui::containers::menu::MenuConfig::new();
    button.config(config.close_behavior(PopupCloseBehavior::CloseOnClickOutside))
}
