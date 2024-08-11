use dioxus::prelude::*;

#[component]
fn Bar(
    size: f64,
    index: usize,
    intensity: f64,
    ) -> Element {
    let (r, g, b) = (
        256.+(28.-256.)*intensity, 
        256.+(110.-256.)*intensity, 
        256.+(140.-256.)*intensity, 
    );
    rsx!{
        rect {
            x: (index as f64) * size,
            y: 0,
            width: size,
            height: 10,
            fill: "rgb({r}, {g}, {b})"
        }
    }
}

#[component]
pub fn SvgTimeLine(
    n_bar: usize,
    intensities: Vec<f64>,
    empty:  bool,
) -> Element {
    rsx!{
        svg {
            preserve_aspect_ratio: "none",
            view_box: "0 0 100 4.2",
            height: "100%",
            width: "100%",
            if empty {
                Bar {
                    size: 100.,
                    index: 0,
                    intensity: 0.
                }
            }
            else {
                for i in 0..n_bar {
                    Bar  {
                        size: 100. / n_bar as f64,
                        index: i,
                        intensity: intensities[i]
                    }
                }
            }
        }
    }
}
