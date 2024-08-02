use dioxus::prelude::*;
use kurbo::Vec2;

#[derive(Props, Clone, PartialEq, Default, Debug)]
pub struct GraphProps {
    pub size: usize,
    pub names: Vec<Option<String>>,
    pub node_weights: Vec<f64>,
    pub node_classes: Vec<Vec<String>>,
    pub edges: Vec<(usize, usize)>,
    pub edge_weights: Vec<Vec<f64>>,
    pub positions: Vec<Vec2>,
}

pub fn MyGraph(g: GraphProps) -> Element {
    let n = g.size;
    let node_size = 10.;
    // TODO: essayer un autre methode pour voir si plus efficace
    // (vec de signaux ?)
    let mut positions = use_signal(|| g.positions);
    let mut selected = use_signal(|| None);

    rsx!{
        svg {
            width: "1000px",
            height: "1000px",
            onmouseup: move |_| *selected.write() = None,
            onmousemove: move |e: MouseEvent| {
                if let Some(id) = selected() {
                    let coord = e.coordinates().element();
                    positions.write()[id] = (coord.x, coord.y).into();
                }
            },
            for a in 0..n {
                for b in 0..=a {
                    if g.edge_weights[a][b] > 0.{
                        line {
                            stroke: "rgba(0,0,0,{g.edge_weights[a][b]})",
                            stroke_width: "3px",
                            x1: positions.read()[a].x,
                            y1: positions.read()[a].y,
                            x2: positions.read()[b].x,
                            y2: positions.read()[b].y,
                        }
                    }
                }
            }
            for id in 0..n {
                circle {
                    onmousedown: move |_| *selected.write() = Some(id),
                    r: node_size*g.node_weights[id],
                    cx: positions.read()[id].x,
                    cy: positions.read()[id].y,
                }
            }
        }
    }

}
