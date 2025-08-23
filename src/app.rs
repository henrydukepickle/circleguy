fn get_first_puzzle() -> String {
    String::from("1010101010geranium.kdl")
}
use crate::data_storer::*;
use crate::io::*;
use crate::puzzle::*;
use crate::puzzle_generation::*;
use crate::render::draw_circle;
use egui::*;

const SCALE_FACTOR: f32 = 200.0;

const ANIMATION_SPEED: f64 = 5.0;

impl DataStorer {
    fn render_panel(&self, ctx: &egui::Context) -> Result<Option<(Puzzle, String)>, ()> {
        let panel = egui::SidePanel::new(egui::panel::Side::Right, "data_panel").resizable(false);
        let mut puzzle = None;
        panel.show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for puz in &self.data {
                    if ui.add(egui::Button::new(&puz.0)).clicked() {
                        puzzle = match parse_kdl(&puz.1) {
                            Some(inside) => Some((inside, puz.1.clone())),
                            None => None,
                        }
                    }
                }
            })
        });
        Ok(puzzle)
    }
}

#[derive(Debug, Clone)]
pub struct App {
    data_storer: DataStorer,
    puzzle: Puzzle,
    def_string: String,
    log_path: String,
    curr_msg: String,
    animation_speed: f64,
    last_frame_time: web_time::Instant,
    outline_width: f32,
    detail: f32,
    scale_factor: f32,
    offset: Vec2,
    cut_on_turn: bool,
    preview: bool,
    rend_correct: bool,
    //debug: usize,
    //debug2: usize,
}
impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut data_storer = DataStorer { data: Vec::new() };
        let _ = data_storer.load_puzzles(&String::from("Puzzles/Definitions/"));
        let p = load_puzzle_and_def_from_file(
            &(String::from("Puzzles/Definitions/") + &get_first_puzzle()),
        )
        .unwrap();
        // for i in 0..20 {
        //     p.0.turn_id("B".to_string(), true);
        //     p.0.turn_id("A".to_string(), true);
        // }
        // let circ = circle(point(0.0, 0.0), 1.0);
        // let t = crate::puzzle_generation::basic_turn(circ, std::f64::consts::PI / 3.5);
        // let mut c2 = 1e-6 * circle(point(0.0, 1.0), 1.0);
        // for i in 0..7000 {
        //     c2 = cga2d::Rotoflector::sandwich(t.rotation, c2);
        // }
        // dbg!(match c2.unpack() {
        //     cga2d::Circle::Circle { cx, cy, r, ori } => (cx, cy, r, ori),
        //     _ => panic!("HI"),
        // });
        // for arc in &rel_piece.shape.border {
        //     dbg!(dbg!(arc.circle).approx_eq(dbg!(&p.0.turns["A"].circle), PRECISION));
        //     dbg!(
        //         arc.circle
        //             .approx_eq(&dbg!(-p.0.turns["A"].circle), PRECISION)
        //     );
        // }
        // p.0.pieces = vec![rel_piece];
        return Self {
            data_storer,
            puzzle: p.0,
            def_string: p.1,
            log_path: String::from("logfile"),
            curr_msg: String::new(),
            animation_speed: ANIMATION_SPEED,
            last_frame_time: web_time::Instant::now(),
            outline_width: 5.0,
            detail: 50.0,
            scale_factor: SCALE_FACTOR,
            offset: vec2(0.0, 0.0),
            cut_on_turn: true,
            preview: false,
            rend_correct: false,
        };
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // dbg!(self.puzzle.pieces.len());
            // dbg!(circle_orienation_euclid(
            //     self.puzzle.pieces[0].shape.border[0].circle
            // ));
            // let a = point(0.0, 0.0);
            // let c1 = inside_circle(a, 1.0);
            // let b = point(1.0, 0.0);
            // let c = point(0.0, 1.0);
            // //dbg!(circle_orienation_euclid(c1));
            // let arc = PieceArc {
            //     circle: c1,
            //     boundary: Some(b ^ c),
            // };

            // let mut sum = 0;
            // for piece in &self.puzzle.pieces {
            //     sum += piece.shape.border.len();
            // }
            //dbg!(sum);
            // dbg!(arc.angle_euc());
            let rect = ui.available_rect_before_wrap();
            // dbg!(sum);
            // arc.draw(
            //     ui,
            //     &rect,
            //     good_detail,
            //     None,
            //     self.outline_width,
            //     self.scale_factor,
            //     self.offset,
            // );
            // dbg!(self.puzzle.pieces[self.debug].shape.border.len());
            // for arc in &self.puzzle.pieces[self.debug].shape.border {
            //     dbg!(arc.angle_euc());
            // }
            if !self.preview {
                if let Err(x) = self.puzzle.render(
                    ui,
                    &rect,
                    self.detail,
                    self.outline_width,
                    self.scale_factor,
                    self.offset,
                    self.rend_correct,
                ) {
                    self.curr_msg = x;
                }
                // let a = PieceArc {
                //     boundary: Some(Blade2 {
                //         mp: -0.0293736,
                //         mx: -0.03444056,
                //         px: 0.024306666,
                //         my: -0.01304419,
                //         py: 0.03105767,
                //         xy: 0.02562106,
                //     }),
                //     circle: Blade3 {
                //         mpx: 0.00000011,
                //         mpy: 0.499999863,
                //         mxy: 0.58625006,
                //         pxy: -0.41374993,
                //     },
                // };
                // let c = Blade3 {
                //     mpx: 0.0,
                //     mpy: -0.5,
                //     mxy: 0.695000,
                //     pxy: -0.304999999,
                // };
                // let ca = PieceArc {
                //     boundary: None,
                //     circle: c,
                // };
                // dbg!(a.contains(a.intersect_circle(c)[1].unwrap()));
                // if let Dipole::Real(real) = a.boundary.unwrap().unpack() {
                //     dbg!(real[1].approx_eq(
                //         &a.intersect_circle(c)[1].unwrap().unpack().unwrap(),
                //         PRECISION
                //     ));
                // }
                // dbg!(a.in_circle(c));
                // //dbg!(a.in_circle(c));
                // for p in [a, ca] {
                //     p.draw(
                //         ui,
                //         &rect,
                //         self.detail,
                //         self.outline_width,
                //         self.scale_factor,
                //         self.offset,
                //     );
                // }
                // self.puzzle.pieces[self.debug].render(
                //     ui,
                //     &rect,
                //     None,
                //     self.detail,
                //     self.outline_width,
                //     self.scale_factor,
                //     self.offset,
                // );
                // for circ in &self.puzzle.pieces[self.debug].shape.bounds {
                //     draw_circle(*circ, ui, &rect, self.scale_factor, self.offset);
                // }
                // self.puzzle.pieces[15].render(
                //     ui,
                //     &rect,
                //     None,
                //     self.detail,
                //     self.outline_width,
                //     self.scale_factor,
                //     self.offset,
                // );
                // self.curr_msg = self.puzzle.pieces[0].shape.border.len().to_string();
                // self.puzzle.pieces[0].shape.border[self.debug].draw(
                //     ui,
                //     &rect,
                //     self.detail,
                //     self.outline_width,
                //     self.scale_factor,
                //     self.offset,
                // );
            } else {
                for piece in &self.puzzle.solved_state {
                    if let Err(x) = piece.render(
                        ui,
                        &rect,
                        None,
                        self.detail,
                        self.outline_width,
                        self.scale_factor,
                        self.offset,
                        self.rend_correct,
                    ) {
                        self.curr_msg = x;
                    }
                }
            }
            // let arc = self.puzzle.pieces[1].shape.border[1];
            // let arc2 = self.puzzle.pieces[1].shape.border[0];
            // dbg!(circle_orientation_euclid(arc.circle));
            // dbg!(circle_orientation_euclid(arc2.circle));
            // dbg!(self.puzzle.pieces[1].in_circle(self.puzzle.turns["A"].circle));
            // arc.draw(
            //     ui,
            //     &rect,
            //     good_detail,
            //     None,
            //     self.outline_width,
            //     self.scale_factor,
            //     self.offset,
            // );
            // arc2.draw(
            //     ui,
            //     &rect,
            //     good_detail,
            //     None,
            //     self.outline_width,
            //     self.scale_factor,
            //     self.offset,
            // );

            match self.data_storer.render_panel(ctx) {
                Err(()) => {
                    self.curr_msg =
                        String::from("Failed to render side panel or failed to create puzzle!")
                }
                Ok(Some(puz)) => {
                    (self.puzzle, self.def_string) = puz;
                }
                _ => {}
            }
            let delta_time = self.last_frame_time.elapsed();
            self.last_frame_time = web_time::Instant::now();
            if self.puzzle.anim_left >= 0.0 {
                self.puzzle.anim_left = f32::max(
                    self.puzzle.anim_left
                        - (delta_time.as_secs_f32() * self.animation_speed as f32),
                    0.0,
                );
            }
            if 24.9 < self.animation_speed {
                self.puzzle.animation_offset = None;
            }
            if (ui.add(egui::Button::new("UNDO")).clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Z)))
                && !self.preview
            {
                let _ = self.puzzle.undo();
            }
            if ui.add(egui::Button::new("SCRAMBLE")).clicked() && !self.preview {
                let _ = self.puzzle.scramble(self.cut_on_turn);
            }
            // if ui
            //     .add(egui::Button::new("INCREMENT DEBUG COUNTER"))
            //     .clicked()
            // {
            //     self.debug += 1;
            // }
            if ui.add(egui::Button::new("RESET")).clicked() && !self.preview {
                self.puzzle.reset();
            }
            ui.add(
                egui::Slider::new(&mut self.outline_width, (0.0)..=(10.0)).text("Outline Width"),
            );
            ui.add(egui::Slider::new(&mut self.detail, (1.0)..=(100.0)).text("Detail"));
            ui.add(
                egui::Slider::new(&mut self.animation_speed, (1.0)..=(25.0))
                    .text("Animation Speed"),
            );
            ui.add(
                egui::Slider::new(&mut self.scale_factor, (10.0)..=(5000.0)).text("Rendering Size"),
            );
            // ui.add(egui::Slider::new(&mut def.r_left, (0.01)..=(2.0)).text("Left Radius"));
            // ui.add(egui::Slider::new(&mut def.n_left, 2..=50).text("Left Number"));
            // ui.add(egui::Slider::new(&mut def.r_right, (0.01)..=(2.0)).text("Right Radius"));
            // ui.add(egui::Slider::new(&mut def.n_right, 2..=50).text("Right Number"));
            ui.add(egui::Slider::new(&mut self.offset.x, (-2.0)..=(2.0)).text("Move X"));
            ui.add(egui::Slider::new(&mut self.offset.y, (-2.0)..=(2.0)).text("Move Y"));
            // ui.add(egui::Slider::new(&mut def.depth, 0..=5000).text("Scramble Depth"));
            if ui.add(egui::Button::new("RESET VIEW")).clicked() {
                (self.scale_factor, self.offset) = (SCALE_FACTOR, vec2(0.0, 0.0))
            }
            ui.label("Log File Path");
            ui.add(egui::TextEdit::singleline(&mut self.log_path));
            #[cfg(not(target_arch = "wasm32"))]
            if ui.add(egui::Button::new("SAVE")).clicked() {
                self.curr_msg = match write_to_file(
                    &self.def_string,
                    &self.puzzle.stack,
                    &(String::from("Puzzles/Logs/") + &self.log_path + ".txt"),
                ) {
                    Ok(()) => String::new(),
                    Err(err) => err.to_string(),
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            if ui.add(egui::Button::new("LOAD LOG")).clicked() {
                (self.puzzle, self.def_string) = load_puzzle_and_def_from_file(
                    &(String::from("Puzzles/Logs/") + &self.log_path + ".txt"),
                )
                .unwrap_or((self.puzzle.clone(), self.def_string.clone()));
            }
            if ui.add(egui::Button::new("RELOAD PUZZLES")).clicked() {
                let _ = self.data_storer.load_puzzles("Puzzles/Definitions/");
            }
            // if ui.add(egui::Button::new("GENERATE")).clicked()
            //     && alneq(1.0, def.r_left + def.r_right)
            // {
            //     puzzle = load(def.clone(), &mut def);
            // }
            // let new_p = data.show_puzzles(ui, &rect);
            // if new_p.is_some() {
            //     puzzle = load(new_p.unwrap(), &mut def);
            // }
            ui.checkbox(&mut self.cut_on_turn, "Cut on turn?");
            ui.checkbox(&mut self.preview, "Preview solved state?");
            ui.checkbox(&mut self.rend_correct, "Render in Fine Mode?");
            //ui.label("Fine mode fixes some rendering errors regarding disconnected pieces, but is significantly less performant.");
            ui.label(String::from("Name: ") + &self.puzzle.name.clone());
            ui.label(String::from("Authors: ") + &self.puzzle.authors.join(","));
            ui.label(self.puzzle.pieces.len().to_string() + " pieces");
            if !self.curr_msg.is_empty() {
                ui.label(&self.curr_msg);
            }
            if self.puzzle.solved {
                ui.label("Solved!");
            }
            let cor_rect = Rect {
                min: pos2(180.0, 0.0),
                max: pos2(rect.width() - 180.0, rect.height()),
            };
            // dbg!((puzzle.turns[1].circle.center).to_pos2());
            if self.puzzle.anim_left != 0.0 {
                ui.ctx().request_repaint();
            }
            let r = ui.interact(cor_rect, egui::Id::new(19), egui::Sense::all());
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
            if r.clicked()
                && !self.preview
                && let Some(pointer) = r.interact_pointer_pos()
            {
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
            if r.hover_pos().is_some()
                && !self.preview
                && let Some(pointer) = r.hover_pos()
            {
                let hovered_circle =
                    self.puzzle
                        .get_hovered(&rect, pointer, self.scale_factor, self.offset);
                if let Err(x) = &hovered_circle {
                    self.curr_msg = x.clone();
                }
                if let Ok(Some(real_circle)) = hovered_circle {
                    draw_circle(real_circle, ui, &rect, self.scale_factor, self.offset);
                }
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
            if r.dragged_by(egui::PointerButton::Middle) {
                let delta = r.drag_delta();
                let good_delta = vec2(
                    delta.x / self.scale_factor,
                    -1.0 * (delta.y / self.scale_factor),
                );
                self.offset += good_delta;
            }
            if ui.input(|i| i.modifiers.command_only()) && scroll != 0 {
                self.scale_factor += 10.0 * scroll as f32;
                // if r.hover_pos().is_some() {
                //     let pos = from_egui_coords(
                //         &r.hover_pos().unwrap(),
                //         &rect,
                //         self.scale_factor,
                //         self.offset,
                //     );
                //     let curr_center =
                //         from_egui_coords(&rect.center(), &rect, self.scale_factor, self.offset)
                //             + self.offset;
                //     self.offset = self.offset + (curr_center - pos);
                //}
            }
        });
    }
}
