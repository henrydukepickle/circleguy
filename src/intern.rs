use cga2d::Multivector;

use crate::puzzle::Puzzle;
impl Puzzle {
    pub fn intern_all(&mut self) {
        for piece in &mut self.pieces {
            for arc in &mut piece.shape.border {
                *arc = arc.rescale_oriented();
                self.intern_3.intern_in_place(&mut arc.circle);
                if arc.boundary.is_some() {
                    self.intern_2.intern_in_place(&mut arc.boundary.unwrap());
                }
            }
            for mut circle in &mut piece.shape.bounds {
                *circle = circle.rescale_oriented();
                self.intern_3.intern_in_place(&mut circle);
            }
        }
    }
}
