use dioxus::prelude::*;
use kurbo::Vec2;
use tracing::info;

#[derive(Props, Clone, PartialEq, Default, Debug)]
pub struct GraphProps {
    pub size: usize,
    pub names: Vec<Option<String>>,
    pub node_weights: Vec<f64>,
    pub node_classes: Vec<Vec<String>>,
    pub edges: Vec<(usize, usize)>,
    pub edge_weights: Vec<Vec<f64>>,
    pub positions: ReadOnlySignal<Vec<Vec2>>,
}

pub fn MyGraph(g: GraphProps) -> Element {
    let n = g.size;
    let node_size = 10.;
    // TODO: essayer un autre methode pour voir si plus efficace
    // (vec de signaux ?)
    let mut positions_edited = use_signal(|| g.positions.cloned());
    let mut selected = use_signal(|| None);

    use_effect(move||
        assert_eq!(
            positions_edited.len(),
            n
        )
    );

    rsx!{
        svg {
            width: "1000px",
            height: "1000px",
            onmouseup: move |_| *selected.write() = None,
            onmousemove: move |e: MouseEvent| {
                if let Some(id) = selected() {
                    let coord = e.coordinates().element();
                    positions_edited.write()[id] = (coord.x, coord.y).into();
                }
            },
            for a in 0..n {
                for b in 0..=a {
                    if g.edge_weights[a][b] > 0.{
                        line {
                            stroke: "rgba(0,0,0,{g.edge_weights[a][b]})",
                            stroke_width: "3px",
                            x1: positions_edited.read()[a].x,
                            y1: positions_edited.read()[a].y,
                            x2: positions_edited.read()[b].x,
                            y2: positions_edited.read()[b].y,
                        }
                    }
                }
            }
            for id in 0..n {
                circle {
                    onmousedown: move |_| *selected.write() = Some(id),
                    r: node_size*g.node_weights[id],
                    cx: positions_edited.read()[id].x,
                    cy: positions_edited.read()[id].y,
                }
            }
        }
    }

}
