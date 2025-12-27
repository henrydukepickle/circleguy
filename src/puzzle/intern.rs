use approx_collections::FloatPool;

use crate::puzzle::puzzle::Puzzle;

// pub struct DipoleInterner {
//     pub mp: FloatPool,
//     pub mx: FloatPool,
//     pub px: FloatPool,
//     pub my: FloatPool,
//     pub py: FloatPool,
//     pub xy: FloatPool,
// }

// impl DipoleInterner {
//     pub fn new(prec: Precision) -> Self {
//         Self {
//             mp: FloatPool::new(prec),
//             mx: FloatPool::new(prec),
//             my: FloatPool::new(prec),
//             px: FloatPool::new(prec),
//             py: FloatPool::new(prec),
//             xy: FloatPool::new(prec),
//         }
//     }
//     fn get_mut_pools(&mut self) -> [&mut FloatPool; 6] {
//         return [
//             &mut self.mp,
//             &mut self.mx,
//             &mut self.px,
//             &mut self.my,
//             &mut self.py,
//             &mut self.xy,
//         ];
//     }
//     fn get_pools(&self) -> [&FloatPool; 6] {
//         return [&self.mp, &self.mx, &self.px, &self.my, &self.py, &self.xy];
//     }
//     fn intern_dipole(&mut self, dip: &mut Blade2) {}
//     fn check_dipole(&self, dip: &Blade2) -> bool {
//         let axes = [&dip.mp, &dip.mx, &dip.px, &dip.my, &dip.py, &dip.xy];
//         let pools = self.get_pools();
//         for i in 0..6 {
//             if pools[i].intern(value)
//         }
//     }
// }

impl Puzzle {
    ///intern all the relevant floats in the puzzle into the 2 float pools
    pub fn intern_all(&mut self) {
        for piece in &mut self.pieces {
            for arc in &mut piece.shape.border {
                self.intern.intern_in_place(&mut arc.circle.center.re);
                self.intern.intern_in_place(&mut arc.circle.center.im);
                self.intern.intern_in_place(&mut arc.circle.r_sq);
                self.intern.intern_in_place(&mut arc.start.re);
                self.intern.intern_in_place(&mut arc.start.im);
                self.intern.intern_in_place(&mut arc.angle);
            }
            for circle in &mut piece.shape.bounds {
                self.intern.intern_in_place(&mut circle.circ.center.re);
                self.intern.intern_in_place(&mut circle.circ.center.im);
                self.intern.intern_in_place(&mut circle.circ.r_sq);
            }
        }
    }
}
