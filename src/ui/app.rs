use std::collections::HashMap;

use crate::puzzle::puzzle::*;
use crate::ui::data_storer::*;
use crate::ui::keybinds::Keybinds;
use crate::ui::keybinds::load_keybinds;
use crate::ui::puzzle_generation::*;
use crate::ui::render::draw_circle;
use egui::*;

///default scale factor
const SCALE_FACTOR: f32 = 200.0;
///default animation speed
const ANIMATION_SPEED: f64 = 5.0;
///credits string
const CREDITS: &str = "Created by Henry Pickle,
with major help from:
Luna Harran (sonicpineapple)
Andrew Farkas (HactarCE)
";
///default puzzle loaded when the program is opened
const DEFAULT_PUZZLE: &str = "2222flowers_2_color.kdl";

#[derive(Debug, Clone)]
///used for running the app. contains all puzzle and view data at runtime
pub struct App {
    data_storer: DataStorer, //stores the data for the puzzles (on the right panel)
    puzzle: Puzzle,          //stores the puzzle
    log_path: String,        //stores the path log files are loaded from/saved to
    curr_msg: String,        //current message (usually for errors)
    animation_speed: f64,    //speed at which animations happen
    last_frame_time: web_time::Instant, //the absolute time at which the last frame happened
    outline_width: f32,      //the width of the outlines
    detail: f32,             //the detail of rendering
    scale_factor: f32,       //the scale factor (zoom)
    offset: Vec2,            //the offset of the puzzle from the center of the screen (pan)
    cut_on_turn: bool,       //whether or not turns should cut the puzzle
    preview: bool,           //whether the solved state is being previewed
    keybinds: Option<Keybinds>,
    debug: bool,
    debug2: usize,
}
impl App {
    ///initialize a new app, using some default settings (from the constants)
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut data_storer = DataStorer {
            puzzles: HashMap::new(),
            sorted_puzzles: Vec::new(),
            prev_data: Vec::new(),
            top: Vec::new(),
        }; //initialize a new data storer
        let _ = data_storer.load_puzzles(
            "Puzzles/Definitions/",
            "Configs/Keybinds/Puzzles/",
            "Configs/Keybinds/groups.kdl",
        );
        let p_data = &data_storer.puzzles.get(DEFAULT_PUZZLE).unwrap().clone();
        Self {
            //return the default app
            data_storer,
            puzzle: parse_kdl(&p_data.data).unwrap(),
            log_path: String::from("logfile"),
            curr_msg: String::new(),
            animation_speed: ANIMATION_SPEED,
            last_frame_time: web_time::Instant::now(),
            outline_width: 5.0,
            detail: 50.0,
            scale_factor: SCALE_FACTOR,
            offset: vec2(0.0, 0.0),
            cut_on_turn: false,
            preview: false,
            debug: false,
            keybinds: if let Some(kb) = &p_data.keybinds
                && let Some(gr) = &p_data.keybind_groups
                && let Some(keybinds) = load_keybinds(&kb, &gr)
            {
                Some(keybinds.clone())
            } else {
                None
            },
            debug2: 0,
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
                self.puzzle.render(
                    ui,
                    &rect,
                    self.detail,
                    self.outline_width,
                    self.scale_factor,
                    self.offset,
                );
                ui.label(
                    self.puzzle.pieces[self.debug2]
                        .shape
                        .border
                        .len()
                        .to_string(),
                );
                if ui.button("NEXT").clicked() {
                    self.debug2 += 1;
                }
                if ui.button("PREV").clicked() {
                    self.debug2 -= 1;
                }
                // } else {
                //     self.puzzle.render(
                //         ui,
                //         &rect,
                //         None,
                //         self.detail,
                //         self.outline_width,
                //         self.scale_factor,
                //         self.offset,
                //         false,
                //     );
                // match &mut self.puzzle.solved_state {
                //     Some(p) => {
                //         for piece in p {
                //             if let Err(x) = piece.render(
                //                 ui,
                //                 &rect,
                //                 None,
                //                 self.detail,
                //                 self.outline_width,
                //                 self.scale_factor,
                //                 self.offset,
                //                 self.rend_correct,
                //             ) {
                //                 self.curr_msg = x;
                //             }
                //         }
                //     }
                //     None => {
                //         self.curr_msg =
                //             String::from("Error in App.update: could not generate puzzle preview!")
                //     }
                // }
                //if the puzzle is in preview mode, render all of the pieces of the solved state
            }
            if self.debug {
                self.puzzle.turn_id("A", false, 1);
                self.puzzle.turn_id("B", false, 2);
            }
            ui.checkbox(&mut self.debug, "LOL");
            // ui.label(self.puzzle.intern.dipoles.len().to_string());
            // if ui.button("PIECES").clicked() {
            //     for piece in &self.puzzle.pieces {
            //         for arc in &piece.shape.border {
            //             dbg!(arc.boundary);
            //         }
            //     }
            // }
            //render the data storer panel -- this stores all of the puzzles that you can load
            match self.data_storer.render_panel(ctx) {
                Err(()) => {
                    self.curr_msg =
                        String::from("Failed to render side panel or failed to create puzzle!")
                }
                Ok(Some(puzzle_data)) => {
                    //if a puzzle is returned (a button is clicked), load it
                    if let Some(puz) = parse_kdl(&puzzle_data.data) {
                        self.puzzle = puz;
                        if let Some(kb) = puzzle_data.keybinds
                            && let Some(gr) = puzzle_data.keybind_groups
                            && let Some(keybinds) = load_keybinds(&kb, &gr)
                        {
                            self.keybinds = Some(keybinds);
                        }
                    }
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
            //add the undo button. undo can also be performed using the z key
            if (ui.add(egui::Button::new("UNDO")).clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Z)))
                && !self.preview
            {
                let _ = self.puzzle.undo();
            }
            //add the scramble button
            if ui.add(egui::Button::new("SCRAMBLE")).clicked() && !self.preview {
                let _ = self.puzzle.scramble(self.cut_on_turn);
            }
            //add the reset button
            if ui.add(egui::Button::new("RESET")).clicked() && !self.preview {
                if self.puzzle.reset().is_err() {
                    self.curr_msg = String::from("Reset failed!")
                };
            }
            //outline width scale
            ui.add(
                egui::Slider::new(&mut self.outline_width, (0.0)..=(10.0)).text("Outline Width"),
            );
            //detail scale
            ui.add(egui::Slider::new(&mut self.detail, (1.0)..=(100.0)).text("Detail"));
            //animation speed scale
            ui.add(
                egui::Slider::new(&mut self.animation_speed, (1.0)..=(25.0))
                    .text("Animation Speed"),
            );
            //rendering size (zoom) scale
            ui.add(
                egui::Slider::new(&mut self.scale_factor, (10.0)..=(5000.0)).text("Rendering Size"),
            );
            //scales for panning (more efficiently done with mouse3, but possible via these)
            ui.add(egui::Slider::new(&mut self.offset.x, (-2.0)..=(2.0)).text("Move X"));
            ui.add(egui::Slider::new(&mut self.offset.y, (-2.0)..=(2.0)).text("Move Y"));
            //resets the view to default
            if ui.add(egui::Button::new("RESET VIEW")).clicked() {
                (self.scale_factor, self.offset) = (SCALE_FACTOR, vec2(0.0, 0.0))
            }
            //input box for editing the path log files save to
            ui.label("Log File Path");
            ui.add(egui::TextEdit::singleline(&mut self.log_path));
            //save functionality, only working when not on web
            #[cfg(not(target_arch = "wasm32"))]
            if ui.add(egui::Button::new("SAVE")).clicked() {
                self.curr_msg = match self
                    .puzzle
                    .write_to_file(&(String::from("Puzzles/Logs/") + &self.log_path + ".kdl"))
                {
                    Ok(()) => String::from("Saved successfully!"),
                    Err(err) => err.to_string(),
                }
            }
            //load functionality, also only working not on web
            #[cfg(not(target_arch = "wasm32"))]
            if ui.add(egui::Button::new("LOAD LOG")).clicked() {
                self.puzzle = load_puzzle_and_def_from_file(
                    &(String::from("Puzzles/Logs/") + &self.log_path + ".kdl"),
                )
                .unwrap_or(self.puzzle.clone());
            }
            //reload the puzzles into the data_storer if they were modifed (doing this every frame is too costly)
            if ui.add(egui::Button::new("RELOAD PUZZLES")).clicked() {
                let _ = self.data_storer.load_puzzles(
                    "Puzzles/Definitions/",
                    "Configs/Keybinds/Puzzles/",
                    "Configs/Keybinds/groups.kdl",
                );
            }
            //whether turns should cut
            ui.checkbox(&mut self.cut_on_turn, "Cut on turn?");
            //whether the solve state is being previewed
            ui.checkbox(&mut self.preview, "Preview solved state?");
            //display puzzle info
            ui.label(String::from("Name: ") + &self.puzzle.name.clone());
            ui.label(String::from("Authors: ") + &self.puzzle.authors.join(","));
            ui.label(self.puzzle.pieces.len().to_string() + " pieces");
            ui.label(self.puzzle.stack.len().to_string() + " turns (QTM uncollapsed)");
            //display the current message if it isn't empty
            if !self.curr_msg.is_empty() {
                ui.label(&self.curr_msg);
            }
            //if the puzzle is solved, display as much (this is currently not working)
            if self.puzzle.solved {
                ui.label("Solved!");
            }
            //display the credits
            ui.label(CREDITS);
            //gets the rect for interaction with the puzzle (so that ui elements like buttons dont conflict with puzzle input)
            let cor_rect = Rect {
                min: pos2(180.0, 0.0),
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
            {
                if let Err(x) = self.puzzle.process_click(
                    &rect,
                    pointer,
                    false,
                    self.scale_factor,
                    self.offset,
                    self.cut_on_turn,
                ) {
                    self.curr_msg = x;
                }
            }
            //keybinds
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
                        && let Some(k) = &self.keybinds
                        && let Some((t, m)) = k.get(&b)
                    {
                        if let Err(x) = self.puzzle.turn_id(&t, self.cut_on_turn, *m) {
                            self.curr_msg = x;
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
                {
                    if let Err(x) = self.puzzle.process_click(
                        &rect,
                        pointer,
                        scroll > 0,
                        self.scale_factor,
                        self.offset,
                        self.cut_on_turn,
                    ) {
                        self.curr_msg = x;
                    }
                }
            }
            //if the middle mouse button is being pressed, pan the camera
            if r.dragged_by(egui::PointerButton::Middle) {
                let delta = r.drag_delta();
                let good_delta = vec2(
                    delta.x / self.scale_factor,
                    -1.0 * (delta.y / self.scale_factor),
                );
                self.offset += good_delta;
            }
            //if ctrl scrolling, zoom
            if ui.input(|i| i.modifiers.command_only()) && scroll != 0 {
                self.scale_factor += 10.0 * scroll as f32;
            }
        });
    }
}
