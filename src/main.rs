//#![windows_subsystem = "windows"]
pub mod app;
pub mod arc;
pub mod circle_utils;
pub mod data_storer;
pub mod intern;
pub mod io;
pub mod keybinds;
pub mod piece;
pub mod piece_shape;
pub mod puzzle;
pub mod puzzle_generation;
pub mod render;
pub mod turn;
use crate::app::*;

use cga2d::*;

///used for general purpose
pub const PRECISION: approx_collections::Precision = Precision::new_simple(20);
///used for purposes that have been tested to need slightly less precision
pub const LOW_PRECISION: approx_collections::Precision = Precision::new_simple(16);
///used for the float pools from approx
pub const POOL_PRECISION: approx_collections::Precision = Precision::new_simple(26);

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    //run the native as defined in app.rs
    eframe::run_native(
        "circleguy",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::puzzle_generation::load_puzzle_and_def_from_file;

    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(
            load_puzzle_and_def_from_file(&"Puzzles/Definitions/666666ring_deep.kdl".to_string())
                .unwrap()
                .pieces
                .len(),
            301
        )
    }
}
