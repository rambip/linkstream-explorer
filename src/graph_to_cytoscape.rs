use super::cytoscape::*;
use super::Graph;
use kurbo::Vec2;
use serde_json::json;


impl Graph {
    pub fn to_cytoscape(&self, positions: &[Vec2]) -> CytoscapeData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        assert_eq!(self.names.len(), self.size);
        assert_eq!(self.node_classes.len(), self.size);

        for i in 0..self.size {
            let mut classes = self.node_classes[i].clone();
            if let Some(sizes) = &self.node_weights {
                classes.push(format!("radius-{}", 5+ (sizes[i] *15.) as u32));
            }

            nodes.push(CytoscapeNode {
                data: CytoscapeNodeData {
                    id: i,
                    label: self.names[i].clone(),
                },
                position: Some(positions[i]),
                classes,
            })
        }

        for (id, (source, target)) in self.edges.iter().enumerate() {
            let mut classes = Vec::new();
            if let Some(sizes) = &self.edge_weights {
                classes.push(format!("opacity-{}", 1+ (sizes[*source][*target] *99.) as u32));
            }

            edges.push(CytoscapeEdge {
                data: CytoscapeEdgeData {
                    id,
                    source: *source,
                    target: *target,
                },
                classes,
            }
            )
        }


        CytoscapeData {
            nodes,
            edges
        }
    }
}

const SHAPES: [&str; 5] = ["diamond", "ellipse", "hexagon", "rectangle", "triangle"];
const COLORS: [&str; 8] = ["royalblue", "seagreen", "crimson", "yellow", "violet", "navy", "darkorange", "black"];


pub fn create_style() -> CytoscapeStyle {
    let mut style_sheet = Vec::new();
    style_sheet.push(CytoscapeStyleRule
        {
            selector: "node".to_string(),
            style: json!{{
                "content": "data(label)",
                "width": "20px",
                "height": "20px",
            }}
    });

    for c in COLORS {
        style_sheet.push(CytoscapeStyleRule {
            selector: format!(".color-{c}"),
            style: json!{{
                "line-color": c,
                "background-color": c
            }}
        })
    }

    for s in SHAPES {
        style_sheet.push(CytoscapeStyleRule {
            selector: format!(".shape-{s}"),
            style: json!{{
                "shape": s,
            }}
        })
    }

    for c in COLORS {
        style_sheet.push(CytoscapeStyleRule {
            selector: format!(".{c}"),
            style: json!{{
                "line-color": c,
                "background-color": c
            }}
        })
    }

    for i in 1..100 {
        let size = i as f32;
        style_sheet.push(CytoscapeStyleRule {
            selector: format!(".radius-{i}"),
            style: json!{{
                "width": format!("{size}px"),
                "height": format!("{size}px"),
            }}
        })
    }

    for i in 1..100 {
        style_sheet.push(CytoscapeStyleRule
            {
                selector: format!(".opacity-{i}"),
                style: json!{{
                    "opacity": format!("{}", i as f64 /100.),
                }}
            }
        );
    };
    CytoscapeStyle(style_sheet)
}

