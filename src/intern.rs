use crate::puzzle::Puzzle;
impl Puzzle {
    pub fn intern_all(&mut self) {
        for piece in &mut self.pieces {
            for arc in &mut piece.shape.border {
                self.intern.intern_in_place(&mut arc.circle);
                if arc.boundary.is_some() {
                    self.intern.intern_in_place(&mut arc.boundary.unwrap());
                }
            }
            for mut circle in &mut piece.shape.bounds {
                self.intern.intern_in_place(&mut circle);
            }
        }
    }
}
