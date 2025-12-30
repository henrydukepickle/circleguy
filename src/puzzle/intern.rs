use crate::puzzle::puzzle::Puzzle;

impl Puzzle {
    ///intern all the relevant floats in the puzzle into the float pool
    pub fn intern_all(&mut self) {
        for piece in &mut self.pieces {
            for arc in &mut piece.shape.border {
                self.intern.intern_in_place(&mut arc.circle.center.0.re);
                self.intern.intern_in_place(&mut arc.circle.center.0.im);
                self.intern.intern_in_place(&mut arc.circle.r_sq);
                self.intern.intern_in_place(&mut arc.start.0.re);
                self.intern.intern_in_place(&mut arc.start.0.im);
                self.intern.intern_in_place(&mut arc.angle);
            }
            for circle in &mut piece.shape.bounds {
                self.intern.intern_in_place(&mut circle.circ.center.0.re);
                self.intern.intern_in_place(&mut circle.circ.center.0.im);
                self.intern.intern_in_place(&mut circle.circ.r_sq);
            }
        }
    }
}
