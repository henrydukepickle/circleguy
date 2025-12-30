use crate::PRECISION;
use crate::complex::arc::*;
use crate::complex::c64::C64;
use crate::complex::complex_circle::Circle;
use crate::complex::complex_circle::Contains;
use crate::complex::point::Point;
use crate::puzzle::piece::*;
use crate::puzzle::puzzle::*;
use crate::puzzle::turn::*;
use crate::ui::data_storer::DataStorer;
use crate::ui::data_storer::PuzzleData;
use approx_collections::*;
use egui::FontId;
use egui::RichText;
use egui::{
    Color32, Pos2, Rect, Stroke, Ui, Vec2,
    epaint::{self, PathShape},
    pos2,
};
use std::cmp::*;
use std::f32::consts::PI;

pub struct RenderingCircle {
    pub cent: Pos2,
    pub rad: f32,
}

///the default rendering color
const DETAIL: f64 = 0.5;
///the color of the outlines
const OUTLINE_COLOR: Color32 = Color32::BLACK;

///draws a the circumference of a circle given the coordinates
pub fn draw_circle(real_circle: Circle, ui: &mut Ui, rect: &Rect, scale_factor: f32, offset: Vec2) {
    {
        ui.painter().circle_stroke(
            to_egui_coords(real_circle.center.to_pos2(), &rect, scale_factor, offset),
            real_circle.r() as f32 * scale_factor * (rect.width() / 1920.0),
            (10.0, Color32::WHITE),
        );
    }
}
///the amount more detailed the outlines are than the interiors
const DETAIL_FACTOR: f64 = 3.0;
///take in a triangle and return if its 'almost degenerate' within some leniency (i.e. its points are 'almost colinear')
fn almost_degenerate(triangle: &Vec<Pos2>, leniency: f32) -> bool {
    let angle_1 = (triangle[1] - triangle[0]).angle() - (triangle[1] - triangle[2]).angle(); //get the relevant (smallest/largest) angle of the triangle, by construction
    let close = angle_1.abs().min((PI - angle_1).abs()); //find how close it is to either extreme (0 or PI)
    if close < leniency {
        return true;
    }
    return false;
}

///averages (takes the barycenter) of a vec of points
fn avg_points(points: &Vec<Pos2>) -> Pos2 {
    let n = points.len() as f32;
    let mut pos = pos2(0.0, 0.0);
    for point in points {
        pos.x += point.x / n;
        pos.y += point.y / n;
    }
    return pos;
}

///translates from cga2d coords to egui coords
fn to_egui_coords(pos: Pos2, rect: &Rect, scale_factor: f32, offset: Vec2) -> Pos2 {
    return pos2(
        ((pos.x + offset.x) as f32) * (scale_factor * rect.width() / 1920.0)
            + (rect.width() / 2.0)
            + rect.min.x,
        -1.0 * ((pos.y + offset.y) as f32) * (scale_factor * rect.width() / 1920.0)
            + (rect.height() / 2.0)
            + rect.min.y,
    );
}

///translates from egui coords to cga2d coords
fn from_egui_coords(pos: &Pos2, rect: &Rect, scale_factor: f32, offset: Vec2) -> Pos2 {
    return pos2(
        ((pos.x - (rect.width() / 2.0)) * (1920.0 / (scale_factor * rect.width()))) - offset.x,
        ((pos.y - (rect.height() / 2.0)) * (-1920.0 / (scale_factor * rect.width()))) - offset.y,
    );
}

impl Point {
    ///translates a Point to an egui Pos2
    pub fn to_pos2(&self) -> Pos2 {
        pos2(self.0.re as f32, self.0.im as f32)
    }
}

impl Arc {
    ///draws an arc (the outline) in OUTLINE_COLOR, according to the parameters passed
    fn draw(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        width: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) -> Result<(), String> {
        let size = self.angle_euc().abs() as f32 * self.circle.r() as f32 * DETAIL_FACTOR as f32; //get the absolute size of the arc, to measure how finely we need to render it
        let divisions = (size * detail * DETAIL as f32).max(2.0) as u16; //find the number of divisions we do for the arc
        let mut coords = Vec::new();
        for pos in self.get_polygon(divisions)? {
            //divide up the arc into a polygon and convert to egui
            coords.push(to_egui_coords(pos, rect, scale_factor, offset_pos));
        }
        ui.painter() //paint the path along the polygon
            .add(PathShape::line(coords, Stroke::new(width, OUTLINE_COLOR)));
        Ok(())
    }
    ///gets the polygon representation of an arc for rendering its outline and for triangulation
    fn get_polygon(&self, divisions: u16) -> Result<Vec<Pos2>, String> {
        let mut points: Vec<Pos2> = Vec::new();
        let start_point = self.start;
        let angle = self.angle_euc(); //take the angle of the arc
        let inc_angle = angle / (divisions as f32);
        for i in 0..=divisions {
            //increment the angle and take points
            points.push(
                ((start_point).rotate_about(self.circle.center, (inc_angle as f64) * (i as f64)))
                    .to_pos2(),
            );
        }
        return Ok(points);
    }
    ///triangulate the arc with respect to a given center
    fn triangulate(&self, center: Pos2, detail: f32) -> Result<Vec<Vec<Pos2>>, String> {
        let size = self.angle_euc().abs() as f32 * self.circle.r() as f32;
        let div = (detail * size * DETAIL as f32).max(2.0) as u16; //get the absolute size and use it to determine the level of detail
        let polygon = self.get_polygon(div)?;
        let mut triangles = Vec::new();
        for i in 0..(polygon.len() - 1) {
            //use the polygon to divide into triangles
            triangles.push(vec![center, polygon[i], polygon[i + 1]]);
        }
        Ok(triangles)
    }
    ///get the euclidian angle of the arc. clockwise arcs are negative by convention
    fn angle_euc(&self) -> f32 {
        self.angle as f32
    }
}
///render a piece, with an outline
impl Piece {
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        offset: Option<Turn>,
        detail: f32,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) -> Result<(), String> {
        //get the offset of the piece, base on if its in the animation_offset circle
        let true_offset = if offset.is_none()
            || self.shape.in_circle(offset.unwrap().circle)
                == Some(crate::complex::complex_circle::Contains::Inside)
        {
            offset
        } else {
            None
        };
        let true_piece = if let Some(twist) = true_offset {
            //turn the piece around the offset
            twist.turn_piece(&self).unwrap_or(self.clone())
        } else {
            self.clone()
        };
        let triangulation = true_piece.triangulate(true_piece.barycenter()?, detail)?; //triangulate the piece around its barycenter
        let mut triangle_vertices: Vec<epaint::Vertex> = Vec::new(); //make a new vector of epaint vertices
        for triangle in triangulation {
            //iterate over the triangles
            if !almost_degenerate(&triangle, 0.0) {
                //remove the degenerate ones
                for point in triangle {
                    let vertex = epaint::Vertex {
                        pos: to_egui_coords(point, rect, scale_factor, offset_pos),
                        uv: pos2(0.0, 0.0),
                        color: true_piece.color,
                    };
                    triangle_vertices.push(vertex); //add the nondegenerate triangle vertices
                }
            }
        }
        let mut mesh = epaint::Mesh::default(); //make a new mesh
        mesh.indices = (0..(triangle_vertices.len() as u32)).collect();
        mesh.vertices = triangle_vertices; //add all the vertices
        ui.painter().add(egui::Shape::Mesh(mesh.into())); //paint the triangles
        true_piece.draw_outline(ui, rect, detail, outline_size, scale_factor, offset_pos)?; //draw the outline
        Ok(())
    }
    ///returns a list of triangles for rendering
    fn triangulate(&self, center: Pos2, detail: f32) -> Result<Vec<Vec<Pos2>>, String> {
        let mut triangles = Vec::new();
        for arc in &self.shape.border {
            //triangulate each arc by the center
            triangles.extend(arc.triangulate(center, detail)?);
        }
        return Ok(triangles);
    }
    ///get the barycenter of the piece based on the arcs for triangulation
    fn barycenter(&self) -> Result<Pos2, String> {
        let mut points = Vec::new();
        for arc in &self.shape.border {
            points.push(arc.midpoint().to_pos2())
        }
        if points.is_empty() {
            return Ok(self.shape.border[0].circle.center.to_pos2());
        }
        return Ok(avg_points(&points)); //average the midpoints of the arcs
    }
    ///draw the outline of the piece
    fn draw_outline(
        &self,
        ui: &mut Ui,
        rect: &Rect,
        detail: f32,
        outline_size: f32,
        scale_factor: f32,
        offset_pos: Vec2,
    ) -> Result<(), String> {
        for arc in &self.shape.border {
            //draw the outline of each arc
            arc.draw(ui, rect, detail, outline_size, scale_factor, offset_pos)?;
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
        detail: f32,
        outline_width: f32,
        scale_factor: f32,
        offset: Vec2,
    ) -> Result<(), String> {
        let proper_offset = if let Some(off) = self.animation_offset {
            //get the offset from the animation_offset and anim_left
            Some(off.mult(self.anim_left as f64))
        } else {
            None
        };
        for piece in &self.pieces {
            //render each piece
            piece.render(
                ui,
                rect,
                proper_offset,
                detail,
                outline_width,
                scale_factor,
                offset,
            )?;
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
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset); //the cga2d position of the click
        let mut min_dist: f32 = 10000.0;
        let mut min_rad: f32 = 10000.0;
        let mut correct_id: String = String::from("");
        for turn in &self.base_turns {
            //iterate over the turns to find the closest one
            let (center, radius) = (turn.1.circle.center.to_pos2(), turn.1.circle.r() as f32);
            //compare how close they are
            //ties are broken by the radius, smaller radius gets priority (so that concentric circles work)
            if ((good_pos.distance(center).approx_cmp(&min_dist, PRECISION) == Ordering::Less)
                || ((good_pos.distance(center).approx_eq(&min_dist, PRECISION))
                    && (radius.approx_cmp(&min_rad, PRECISION)) == Ordering::Less))
                && turn.1.circle.contains(Point(C64 {
                    re: good_pos.x as f64,
                    im: good_pos.y as f64,
                })) == Contains::Inside
            {
                min_dist = good_pos.distance(center);
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
        let good_pos = from_egui_coords(&pos, rect, scale_factor, offset); //get the position
        let mut min_dist: f32 = 10000.0;
        let mut min_rad: f32 = 10000.0;
        let mut correct_turn = None;
        for turn in self.base_turns.clone().values() {
            //this algorithm proceeds very similarly to the process_click algorithm above
            let (cent, rad) = (turn.circle.center.to_pos2(), turn.circle.r() as f32);
            if ((good_pos.distance(cent).approx_cmp(&min_dist, PRECISION) == Ordering::Less)
                || ((good_pos.distance(cent).approx_eq(&min_dist, PRECISION))
                    && (rad.approx_cmp(&min_rad, PRECISION)) == Ordering::Less))
                && good_pos.distance(cent) < rad
            {
                min_dist = good_pos.distance(cent);
                min_rad = rad;
                correct_turn = Some(*turn);
            }
        }
        if min_rad == 10000.0 {
            return Ok(None);
        }
        return Ok(Some(
            match correct_turn {
                None => return Ok(None),
                Some(x) => x,
            }
            .circle,
        ));
    }
}

impl DataStorer {
    ///render the data panel on the screen and read input for which button is clicked
    pub fn render_panel(&mut self, ctx: &egui::Context) -> Result<Option<PuzzleData>, ()> {
        let panel = egui::SidePanel::new(egui::panel::Side::Right, "data_panel").resizable(false); //make the new panel
        let mut puzzle_data = None;
        panel.show(ctx, |ui| {
            ui.label(RichText::new("Puzzles").font(FontId::proportional(20.0)));
            //button to reload the puzzles into the data_storer if they were modifed (doing this every frame is too costly)
            if ui.add(egui::Button::new("Reload Puzzle List")).clicked() {
                let _ = self.load_puzzles(
                    "Puzzles/Definitions/",
                    "Configs/Keybinds/Puzzles/",
                    "Configs/Keybinds/groups.kdl",
                );
            }
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for puz in &self.sorted_puzzles {
                    if ui.add(egui::Button::new(puz.preview.clone())).clicked() {
                        //make the buttons for each puzzle
                        puzzle_data = Some(puz.clone());
                    }
                }
            })
        });
        Ok(puzzle_data)
    }
}
