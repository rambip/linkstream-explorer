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
    pub positions: Signal<Vec<Vec2>>,
}

pub fn MyGraph(mut g: GraphProps) -> Element {
    let n = g.size;
    let node_size = 10.;
    // TODO: essayer un autre methode pour voir si plus efficace
    // (vec de signaux ?)
    let mut selected = use_signal(|| None);

    let pos = g.positions.read();

    assert_eq!(n, pos.len());

    rsx!{
        svg {
            width: "1000px",
            height: "1000px",
            onmouseup: move |_| *selected.write() = None,
            onmousemove: move |e: MouseEvent| {
                if let Some(id) = selected() {
                    let coord = e.coordinates().element();
                    g.positions.write()[id] = (coord.x, coord.y).into();
                }
            },
            for a in 0..n {
                for b in 0..=a {
                    if g.edge_weights[a][b] > 0.{
                        line {
                            stroke: "rgba(0,0,0,{g.edge_weights[a][b]})",
                            stroke_width: "3px",
                            x1: pos[a].x,
                            y1: pos[a].y,
                            x2: pos[b].x,
                            y2: pos[b].y,
                        }
                    }
                }
            }
            for id in 0..n {
                // TODO: z-index
                {info!("{}", pos.len())}
                circle {
                    onmousedown: move |_| *selected.write() = Some(id),
                    r: node_size*g.node_weights[id],
                    cx: pos[id].x,
                    cy: pos[id].y,
                }
                if let Some(name) = &g.names[id] {
                    text {
                        x: pos[id].x+20.,
                        y: pos[id].y+5.,
                        "{name}"
                    }
                }
            }
        }
    }

}
