#[test]
fn test_circle() {
    use crate::{
        complex::{
            arc::Arc,
            c64::C64,
            complex_circle::{ComplexCircle, Contains, Orientation, OrientedCircle},
        },
        puzzle::piece_shape::PieceShape,
    };

    let piece1 = PieceShape {
        border: vec![
            Arc::from_endpoints(
                ComplexCircle {
                    center: C64 { re: 0.0, im: 0.0 },
                    r_sq: 1.0,
                },
                C64 { re: 1.0, im: 0.0 },
                C64 { re: 0.0, im: 1.0 },
                Orientation::CCW,
            ),
            Arc::from_endpoints(
                ComplexCircle {
                    center: C64 { re: 1.0, im: 1.0 },
                    r_sq: 1.0,
                },
                C64 { re: 0.0, im: 1.0 },
                C64 { re: 1.0, im: 0.0 },
                Orientation::CCW,
            ),
        ],
        bounds: vec![
            OrientedCircle {
                circ: ComplexCircle {
                    center: C64 { re: 0.0, im: 0.0 },
                    r_sq: 1.0,
                },
                ori: Contains::Inside,
            },
            OrientedCircle {
                circ: ComplexCircle {
                    center: C64 { re: 1.0, im: 1.0 },
                    r_sq: 1.0,
                },
                ori: Contains::Inside,
            },
        ],
    };
    let c1 = ComplexCircle {
        center: C64 { re: -1.0, im: -1.0 },
        r_sq: 1.0,
    };
    assert_eq!(piece1.cut_by_circle(c1), None);
}
