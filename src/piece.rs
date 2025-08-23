use crate::piece_shape::*;
use crate::turn::*;
use cga2d::*;
use egui::Color32;
#[derive(Debug, Clone)]
pub struct Piece {
    pub shape: PieceShape,
    pub color: Color32,
}

impl Piece {
    pub fn turn(&self, turn: Turn) -> Option<Piece> {
        return Some(Piece {
            shape: self.shape.turn(turn)?,
            color: self.color,
        });
    }
    pub fn turn_cut(&self, turn: Turn) -> [Option<Piece>; 2] {
        return self.shape.turn_cut(turn).map(|x| match x {
            None => None,
            Some(x) => Some(Piece {
                shape: x,
                color: self.color,
            }),
        });
    }
    //None: the piece is inside and outside -- blocking
    //return if the shape contains the point properly -- not on the border
    //should return false if the point is properly outside the piece and true if it is properly inside the piece -- behavior on border points is undefined, may panic or return either option.
    //essentially, check how many 'valid' points that are on the border of self and directly left (within leniency) of point, and then take that mod 2 to get the answer
    // fn contains_point(&self, point: Pos2F64) -> bool {
    //     let y = point.y;
    //     let mut intersects = 0;
    //     for i in 0..self.shape.len() {
    //         let arc = self.shape[i];
    //         let prev_arc = self.shape[match i {
    //             0 => self.shape.len() - 1,
    //             _ => i - 1,
    //         }];
    //         let points = arc.arc_points_directly_left(point); //get the points on the current arc directly left of the point
    //         for int_point in points {
    //             if aeq_pos(arc.start, int_point) {
    //                 if ((arc.initially_above_y(y)) != (prev_arc.invert().initially_above_y(y))) //tangent case -- basically we only add in this case if the arcs actually cross the y line
    //                     || aeq(arc.angle, 2.0 * PI as f64)
    //                 {
    //                     intersects += 1;
    //                 }
    //             } else {
    //                 if !aeq((arc.circle.center.y - y).abs(), arc.circle.radius) {
    //                     //throw out the case where the arc is tangent at this point to the y line
    //                     intersects += 1;
    //                 }
    //             }
    //         }
    //     }
    //     //dbg!(intersects);
    //     return (intersects % 2) != 0; //the point is inside the piece if the number of valid intersects is odd, and outside otherwise
    // }
    // fn contains_point_border(&self, point: Pos2F64) -> bool {
    //     for arc in &self.shape {
    //         if arc.contains_properly(point) || aeq_pos(point, arc.start) {
    //             return true;
    //         }
    //     }
    //     return false;
    // }
    //doesnt count arcs
    //fn intersect_arc_start(&self, arc: Arc) -> Option<Vec<Pos2F64>> {}
    //fn intersect_piece(&self, other: Piece) -> Vec<Pos2F64> {}
    // cut a piece by a circle and return the resulting pieces as a Vec
    //the color is maintained
    //[inside, outside]
    pub fn cut_by_circle(&self, circle: Blade3) -> [Option<Piece>; 2] {
        let shapes = self.shape.cut_by_circle(circle);
        shapes.map(|x| match x {
            None => None,
            Some(x) => Some(Piece {
                shape: x,
                color: self.color,
            }),
        })
    }
    //determine which piece occurs earlier on the piece
    //self.shape[0].start is the minimum
    //if the two points are equivalent, returns Less
    // fn order_along_piece(&self, point1: Pos2F64, point2: Pos2F64) -> Ordering {
    //     for arc in &self.shape {
    //         if aeq_pos(point1, arc.start) {
    //             return Ordering::Less;
    //         } else if aeq_pos(point2, arc.start) {
    //             return Ordering::Greater;
    //         } else if arc.contains_properly(point1) {
    //             if arc.contains_properly(point2) {
    //                 if arc.get_angle(point1).abs() <= arc.get_angle(point2).abs() {
    //                     return Ordering::Less;
    //                 }
    //                 return Ordering::Greater;
    //             }
    //             return Ordering::Less;
    //         } else if arc.contains_properly(point2) {
    //             return Ordering::Greater;
    //         }
    //     }
    //     return Ordering::Equal;
    // }
    //sort points based on how they appear along the piece, ccw
    // fn sort_along_piece(&self, points: &Vec<Pos2F64>) -> Vec<Pos2F64> {
    //     let mut sorted = points.clone();
    //     sorted.sort_by(|a, b| self.order_along_piece(*a, *b));
    //     return sorted;
    // }
    //cut the piece into segments (cut at a bunch of points)
    // fn cut_at(&self, points: &Vec<Pos2F64>) -> Vec<Vec<Arc>> {
    //     let mut remaining_shape = Vec::new();
    //     for arc in &self.shape {
    //         remaining_shape.extend(arc.cut_at(points))
    //     }
    //     let sorted_points = self.sort_along_piece(points);
    //     let index = 0;
    //     let mut segments = Vec::new();
    //     let mut curr_segment = Vec::new();
    //     while !remaining_shape.is_empty() {
    //         let arc = remaining_shape[0];
    //         let point = sorted_points[index];
    //         if aeq_pos(point, arc.start) {
    //             segments.push(curr_segment.clone());
    //             curr_segment = Vec::new();
    //         }
    //     }
    //     return segments;
    // }
    // fn cut_by_piece(&self, other: Piece) -> Vec<Piece> {
    //     return Vec::new();
    // }
    // fn get_polygon(&self, detail: u16) -> Vec<Pos2F64> {
    //     let mut points = Vec::new();
    //     for arc in &self.shape {
    //         points.extend(arc.get_polygon(detail));
    //         points.pop();
    //     }
    //     return points;
    // }
}
