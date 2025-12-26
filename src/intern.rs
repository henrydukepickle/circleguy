use approx_collections::FloatPool;
use cga2d::{ApproxEq, Blade2, Blade3, Circle, Multivector, Precision};

use crate::puzzle::Puzzle;

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

#[derive(Debug, Clone)]
pub struct Interner {
    pub prec: Precision,
    pub dipoles: Vec<Blade2>,
    pub circles: Vec<Blade3>,
}

impl Interner {
    pub fn intern_2(&mut self, blade: &mut Blade2) {
        for b in &self.dipoles {
            if b.approx_eq(blade, self.prec) {
                *blade = *b;
                return;
            }
        }
        self.dipoles.push(*blade);
    }
    pub fn intern_3(&mut self, blade: &mut Blade3) {
        for b in &self.circles {
            if b.approx_eq(blade, self.prec) {
                *blade = *b;
                return;
            }
        }
        self.circles.push(*blade);
    }
    pub fn new(prec: Precision) -> Self {
        Self {
            prec,
            dipoles: Vec::new(),
            circles: Vec::new(),
        }
    }
}

impl Puzzle {
    ///intern all the relevant floats in the puzzle into the 2 float pools
    pub fn intern_all(&mut self) {
        for piece in &mut self.pieces {
            for arc in &mut piece.shape.border {
                *arc = arc.rescale_oriented(); //for each arc, rescale it and then intern both its circle and its boundary (if it exists)
                self.intern.intern_3(&mut arc.circle);
                if let Some(bound) = &mut arc.boundary {
                    self.intern.intern_2(bound);
                    // if self.intern_2.len() > len {
                    //     //dbg!(&arc.boundary.unwrap());
                    //     //arc.debug();
                    // }
                }
            }
            for circle in &mut piece.shape.bounds {
                //intern each circle
                *circle = circle.rescale_oriented();
                self.intern.intern_3(circle);
            }
        }
    }
}
