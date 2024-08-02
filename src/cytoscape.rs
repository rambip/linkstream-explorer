use kurbo::Vec2;
use dioxus::prelude::*;
use serde::Serialize;


#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct CytoscapeNodeData {
    pub id: usize,
    pub label: Option<String>,
}

#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct CytoscapeEdgeData {
    pub id: usize,
    pub source: usize,
    pub target: usize,
}

#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct CytoscapeEdge {
    pub data: CytoscapeEdgeData,
    pub classes: Vec<String>,
}

#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct CytoscapeNode {
    pub data: CytoscapeNodeData,
    pub position: Option<Vec2>,
    pub classes: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq, Default, Clone)]
pub struct CytoscapeData {
    pub nodes: Vec<CytoscapeNode>,
    pub edges: Vec<CytoscapeEdge>,
}

#[derive(PartialEq, Clone, Serialize, Debug)]
pub struct CytoscapeStyleRule {
    pub selector: String,
    pub style: serde_json::Value,
}

#[derive(PartialEq, Clone, Serialize)]
pub struct CytoscapeStyle(pub Vec<CytoscapeStyleRule>);

static CYTOSCAPE_LOADED: GlobalSignal<bool> = Signal::global(|| false);
static CYTOSCAPE_ID: GlobalSignal<usize> = Signal::global(|| 0);

use dioxus_logger::tracing::info;

#[component]
pub fn Cytoscape(data: ReadOnlySignal<CytoscapeData>, style: CytoscapeStyle, pan: Vec2) -> Element {
    
    let id = format!("cy-{}", CYTOSCAPE_ID.peek());
    *CYTOSCAPE_ID.write() += 1;



    let id2 = id.clone();
    let setup_cytoscape = move || {
        let eval = eval(
            r#"
            let id_str = await dioxus.recv();
            let style_str = await dioxus.recv();
            let pan = await dioxus.recv();
            const cy = cytoscape({
                container: document.getElementById(id_str),
                elements: [],
                style: JSON.parse(style_str),
                layout : {name: "preset", uirevision: "static", fit:false},
                pan: JSON.parse(pan),
            })
            console.log(style_str);
            while (true) {
                let elements_str = await dioxus.recv();
                console.log(elements_str);
                cy.add(JSON.parse(elements_str),)
            }
            "#,
        );
        eval.send( serde_json::Value::String(id.clone())).unwrap();
        eval.send( serde_json::Value::String(serde_json::to_string(&style).unwrap())).unwrap();
        eval.send( serde_json::Value::String(serde_json::to_string(&pan).unwrap())).unwrap();

        use_effect(move ||
            eval.send(
                serde_json::Value::String( serde_json::to_string(&*data.read()).unwrap())
            ).unwrap()
        );
    };

    use_effect(move || { if *CYTOSCAPE_LOADED.read() { setup_cytoscape() } });



    rsx!{
        div {
            id: id2,
            class: "graph-container",
            script {
                src: "https://cdn.jsdelivr.net/npm/cytoscape@3.30.0/dist/cytoscape.min.js",
                onload: move |_| *CYTOSCAPE_LOADED.write() = true
            }
        }
    }
}
