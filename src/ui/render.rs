use crate::PRECISION;
use crate::complex::c64::C64;
use crate::complex::complex_circle::Circle;
use crate::complex::complex_circle::Contains;
use crate::complex::point::Point;
use crate::hps::data_storer::DataStorer;
use crate::hps::data_storer::PuzzleLoadingData;
use crate::puzzle::color::Color;
use crate::puzzle::puzzle::*;
use crate::puzzle::render_piece::RenderPiece;
use crate::puzzle::render_piece::Triangulation;
use crate::puzzle::turn::*;
use approx_collections::*;
use core::f64;
use egui::FontId;
use egui::RichText;
use egui::{
    Color32, Pos2, Rect, Stroke, Ui, Vec2,
    epaint::{self, PathShape},
    pos2,
};
use std::cmp::*;

pub struct RenderingCircle {
    pub cent: Pos2,
    pub rad: f32,
}

///the default rendering color
///the color of the outlines
const OUTLINE_COLOR: Color32 = Color32::BLACK;

impl Color {
    pub fn to_egui(&self) -> Color32 {
        match self {
            Color::Red => Color32::RED,
            Color::Green => Color32::GREEN,
            Color::Blue => Color32::BLUE,
            Color::Yellow => Color32::YELLOW,
            Color::Purple => Color32::PURPLE,
            Color::Gray => Color32::GRAY,
            Color::Black => Color32::BLACK,
            Color::Brown => Color32::BROWN,
            Color::Cyan => Color32::CYAN,
            Color::White => Color32::WHITE,
            Color::DarkBlue => Color32::DARK_BLUE,
            Color::DarkGreen => Color32::DARK_GREEN,
            Color::DarkGray => Color32::DARK_GRAY,
            Color::DarkRed => Color32::DARK_RED,
            Color::LightBlue => Color32::LIGHT_BLUE,
            Color::LightGray => Color32::LIGHT_GRAY,
            Color::LightGreen => Color32::LIGHT_GREEN,
            Color::LightYellow => Color32::LIGHT_YELLOW,
            Color::LightRed => Color32::LIGHT_RED,
            Color::Khaki => Color32::KHAKI,
            Color::Gold => Color32::GOLD,
            Color::Magenta => Color32::MAGENTA,
            Color::Orange => Color32::ORANGE,
            Color::None => Color32::GRAY,
        }
    }
}

///draws a the circumference of a circle given the coordinates
pub fn draw_circle(real_circle: Circle, ui: &mut Ui, rect: &Rect, scale_factor: f32, offset: Vec2) {
    {
        ui.painter().circle_stroke(
            real_circle.center.to_pos2(rect, scale_factor, offset),
            real_circle.r() as f32 * scale_factor * (rect.width() / 1920.0),
            (10.0, Color32::WHITE),
        );
    }
}

impl Point {
    ///translates from cga2d coords to egui coords
    fn to_pos2(&self, rect: &Rect, scale_factor: f32, offset: Vec2) -> Pos2 {
        pos2(
            (self.0.re as f32 + offset.x) * (scale_factor * rect.width() / 1920.0)
                + (rect.width() / 2.0)
                + rect.min.x,
            -(self.0.im as f32 + offset.y) * (scale_factor * rect.width() / 1920.0)
                + (rect.height() / 2.0)
                + rect.min.y,
        )
    }
    ///translates from egui coords to cga2d coords
    fn from_pos2(pos: &Pos2, rect: &Rect, scale_factor: f32, offset: Vec2) -> Self {
        Self(C64 {
            re: (((pos.x - (rect.width() / 2.0)) * (1920.0 / (scale_factor * rect.width())))
                - offset.x) as f64,
            im: (((pos.y - (rect.height() / 2.0)) * (-1920.0 / (scale_factor * rect.width())))
                - offset.y) as f64,
        })
    }
}

impl Triangulation {
    ///render the triangulation, according to a detail and a color. includes outlines
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        scale_factor: f32,
        offset: Vec2,
        width: f32,
        color: Color,
    ) {
        let mut triangle_vertices: Vec<epaint::Vertex> = Vec::new(); //make a new vector of epaint vertices
        for triangle in &self.inside {
            //iterate over the triangles
            for point in triangle {
                let vertex = epaint::Vertex {
                    pos: point.to_pos2(rect, scale_factor, offset),
                    uv: pos2(0.0, 0.0),
                    color: color.to_egui(),
                };
                triangle_vertices.push(vertex); //add the nondegenerate triangle vertices
            }
        }
        let mut mesh = epaint::Mesh::default(); //make a new mesh
        mesh.indices = (0..(triangle_vertices.len() as u32)).collect();
        mesh.vertices = triangle_vertices; //add all the vertices
        ui.painter().add(egui::Shape::Mesh(mesh.into())); //paint the triangles

        //now we render the outlines
        for arc in &self.border {
            ui.painter().add(PathShape::line(
                arc.iter()
                    .map(|x| x.to_pos2(rect, scale_factor, offset))
                    .collect(),
                Stroke::new(width, OUTLINE_COLOR),
            ));
        }
    }
}

///render a piece, with an outline
impl RenderPiece {
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        offset: Option<Turn>,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) -> Result<(), String> {
        //get the offset of the piece, base on if its in the animation_offset circle
        let true_offset = if offset.is_none()
            || self.piece.shape.in_circle(offset.unwrap().circle)
                == Some(crate::complex::complex_circle::Contains::Inside)
        {
            offset
        } else {
            None
        };
        let true_piece = if let Some(twist) = true_offset {
            //turn the piece around the offset
            twist.turn_render_piece(self).unwrap_or(self.clone())
        } else {
            self.clone()
        };
        for triangle in &true_piece.triangulations {
            //iterate over the triangles
            triangle.render(
                ui,
                rect,
                scale_factor,
                offset_pos,
                outline_size,
                self.piece.color,
            );
        }
        Ok(())
    }
}
impl Puzzle {
    ///render the puzzle, including outlines
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        outline_width: f32,
        scale_factor: f32,
        offset: Vec2,
    ) -> Result<(), String> {
        //get the offset from the animation_offset and anim_left
        let proper_offset = self
            .animation_offset
            .map(|off| off.mult(self.anim_left as f64));
        for piece in &self.pieces {
            //render each piece
            piece.render(ui, rect, proper_offset, outline_width, scale_factor, offset)?;
        }
        Ok(())
    }
    ///processes a click input and does the corresponding turns
    ///Ok(true) means the turn was completed
    ///Ok(false) means that the turn was bandanged, or no turn was found
    ///Err(e) means that an error was encountered
    ///'cut' is whether the turn should cut
    pub fn process_click(
        &mut self,
        rect: &Rect,
        pos: Pos2,
        left: bool,
        scale_factor: f32,
        offset: Vec2,
        cut: bool,
    ) -> Result<bool, String> {
        let good_pos = Point::from_pos2(&pos, rect, scale_factor, offset); //the cga2d position of the click
        let mut min_dist: f64 = 10000.0;
        let mut min_rad: f64 = 10000.0;
        let mut correct_id: String = String::from("");
        for turn in &self.turns {
            //iterate over the turns to find the closest one
            let (center, radius) = (turn.1.turn.circle.center, turn.1.turn.circle.r());
            //compare how close they are
            //ties are broken by the radius, smaller radius gets priority (so that concentric circles work)
            if ((good_pos.dist(center).approx_cmp(&min_dist, PRECISION) == Ordering::Less)
                || ((good_pos.dist(center).approx_eq(&min_dist, PRECISION))
                    && (radius.approx_cmp(&min_rad, PRECISION)) == Ordering::Less))
                && turn.1.turn.circle.contains(Point(C64 {
                    re: good_pos.0.re as f64,
                    im: good_pos.0.im as f64,
                })) == Contains::Inside
            {
                min_dist = good_pos.dist(center);
                min_rad = radius;
                correct_id = turn.0.clone();
            }
        }
        if correct_id.is_empty() {
            //if no circle was found
            return Ok(false);
        }
        if !left {
            //invert based on the type of click
            Ok(self.turn_id(&correct_id, cut, 1)?)
        } else {
            Ok(self.turn_id(&correct_id, cut, -1)?)
        }
    }
    ///get the circle hovered by the mouse
    ///picks amongst the valid turn circles of the puzzle
    pub fn get_hovered(
        &self,
        rect: &Rect,
        pos: Pos2,
        scale_factor: f32,
        offset: Vec2,
    ) -> Result<Option<Circle>, String> {
        let good_pos = Point::from_pos2(&pos, rect, scale_factor, offset); //get the position
        let mut min_dist: f64 = 10000.0;
        let mut min_rad: f64 = 10000.0;
        let mut correct_turn = None;
        for turn in self.turns.clone().values() {
            //this algorithm proceeds very similarly to the process_click algorithm above
            let (cent, rad) = (turn.turn.circle.center, turn.turn.circle.r());
            if ((good_pos.dist(cent).approx_cmp(&min_dist, PRECISION) == Ordering::Less)
                || ((good_pos.dist(cent).approx_eq(&min_dist, PRECISION))
                    && (rad.approx_cmp(&min_rad, PRECISION)) == Ordering::Less))
                && good_pos.dist(cent) < rad
            {
                min_dist = good_pos.dist(cent);
                min_rad = rad;
                correct_turn = Some(*turn);
            }
        }
        if min_rad == 10000.0 {
            return Ok(None);
        }
        Ok(Some(
            match correct_turn {
                None => return Ok(None),
                Some(x) => x,
            }
            .turn
            .circle,
        ))
    }
}

impl DataStorer {
    ///render the data panel on the screen and read input for which button is clicked
    pub fn render_panel(&mut self, ctx: &egui::Context) -> Result<Option<PuzzleLoadingData>, ()> {
        let panel = egui::SidePanel::new(egui::panel::Side::Right, "data_panel").resizable(false); //make the new panel
        let mut puzzle_data = None;
        panel
            .show(ctx, |ui| {
                ui.label(RichText::new("Puzzles").font(FontId::proportional(20.0)));
                //button to reload the puzzles into the data_storer if they were modifed (doing this every frame is too costly)
                if ui.add(egui::Button::new("Reload Puzzle List")).clicked() {
                    if let Err(x) = self.reset(false) {
                        return Err(x);
                    }
                    let _ = self.load_puzzles("Puzzles/Definitions/");
                    let _ = self.load_keybinds("Configs/keybinds.kdl");
                }
                if ui
                    .add(egui::Button::new("Load Experimental Puzzles"))
                    .clicked()
                {
                    if let Err(x) = self.reset(true) {
                        return Err(x);
                    }
                    let _ = self.load_puzzles("Puzzles/Definitions/");
                    let _ = self.load_keybinds("Configs/keybinds.kdl");
                }
                ui.separator();
                Ok(egui::ScrollArea::vertical().show(ui, |ui| {
                    let puzzles_real = self.puzzles.lock().unwrap();
                    let mut names = puzzles_real.keys().collect::<Vec<&String>>();
                    names.sort();
                    for name in names {
                        let puz = if let Some(x) = puzzles_real.get(name) {
                            x
                        } else {
                            return;
                        };
                        if ui.add(egui::Button::new(puz.name.clone())).clicked() {
                            //make the buttons for each puzzle
                            puzzle_data = Some(puz.clone());
                        }
                    }
                }))
            })
            .inner
            .or(Err(()))?;
        Ok(puzzle_data)
    }
}
