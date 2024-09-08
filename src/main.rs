#![allow(non_snake_case)]

use crate::linkstream::{LinkStream, LinkStreamData};
use crate::utils::Matrix;
use dioxus::prelude::*;
use kurbo::Vec2;
use std::collections::HashMap;
use std::ops::Range;
use tracing::Level;

mod force_directed_layout;
mod linkstream;
mod render_graph;
mod svg_timeline;
mod utils;
mod time_slider;


use svg_timeline::SvgTimeLine;
use render_graph::MyGraph;
use time_slider::TimeSlider;
use utils::Reset;

#[cfg(debug_assertions)]
const PUBLIC_URL: &str = "http://localhost:8080/linkstream-explorer";

#[cfg(not(debug_assertions))]
const PUBLIC_URL: &str = "https://rambip.github.io/linkstream-explorer";

fn main() {
    #[cfg(debug_assertions)]
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    launch(Home);
}



#[allow(non_snake_case)]
fn ToolBox() -> Element {
    rsx! {}
}

#[component]
fn LoadingGif() -> Element {
    rsx! {
        div { class: "graph-container gif-container",
            img {
                src: "loading_state-optimize.gif",
                alt: "loading_gif",
                height: "250px",
                width: "320px"
            }
        }
    }
}

#[component]
fn Menu(
    current_dataset: ReadOnlySignal<LinkStream>,
    visible_toogle: Signal<bool>,
    time_window: Signal<Range<u64>>,
    time: ReadOnlySignal<u64>,
    r_value: Signal<f64>,
) -> Element {
    let mut zoom = use_signal(|| 0.);

    rsx! {
        div { class: "menu-container",
            div {
                span { id: "current-time", class: "current-time", "current time: {time}" }
            }
            div { class: "right-bar",
                div { id: "graph-info", class: "graph-info",
                    div { class: "rb-area tools",
                        ToolBox {}
                        div { class: "zoom-container",
                            p { class: "zoom-label", "Zoom" }
                            span { "1x" }
                            input {
                                class: "zoom-slider",
                                r#type: "range",
                                value: "{zoom}",
                                oninput: move |e| zoom.set(e.parsed().unwrap()),
                                min: "0",
                                max: "3",
                                step: "any"
                            }
                            span { "1000x" }
                        }
                    }
                    div { class: "rb-area output",
                        h2 { "Graph Stats" }
                        div { class: "data-output", "TODO" }
                    }
                }
            }
            TimeSlider {
                current_dataset,
                time_window,
                zoom,
                time,
                r_value
            }
        }
    }
}

#[component]
fn Popup(children: Vec<VNode>) -> Element {
    rsx! {
        div { class: "dummy-container", {children.into_iter()} }
    }
}

static DATASETS: [(&'static str, &'static str); 3] = [
    ("baboon", ("baboon.json")),
    ("school", ("school.json")),
    ("example", ("example.json")),
];

//#[oneshot]
async fn load_linkstream_and_compute_positions(
    name: String,
    data: LinkStreamData,
) -> (LinkStream, Vec<Vec2>) {
    let link_stream = LinkStream::new(name, data);

    let n = link_stream.node_count();

    let matrix = link_stream.interaction_matrix(link_stream.time_window());
    let m = matrix.matrix_max();
    let normalized_matrix = matrix.matrix_map(|x| x / m);

    let params = force_directed_layout::ForceDirectedLayoutParams {
        dt: 0.05,
        l_0: 0.5 / (n as f64).sqrt(),
        k_r: 0.1,
        k_s: 0.02,
        n_iterations: 500,
        scale: 400.,
    };

    let positions = force_directed_layout::compute(n, &normalized_matrix, params);
    return (link_stream, positions);
}

#[component]
fn GraphView(
    current_dataset: ReadOnlySignal<LinkStream>,
    time_window: ReadOnlySignal<Range<u64>>,
    t: ReadOnlySignal<u64>,
    dt: ReadOnlySignal<u64>,
    mut positions: Signal<Vec<Vec2>>,
) -> Element {
    let n = current_dataset.read().node_count();
    let n_pos = positions.read().len();
    assert_eq!(n, n_pos);

    let (edge_weigths, edges) = {
        let mut edges = Vec::new();
        let matrix = current_dataset
            .read()
            .interaction_matrix(t() - dt() / 2..t() + dt() / 2);
        let m = matrix.matrix_max();

        for n1 in 0..n {
            for n2 in 0..n {
                if matrix[n1][n2] > 0. {
                    edges.push((n1, n2));
                }
            }
        }

        (matrix.matrix_map(|x| x / m), edges)
    };

    let node_weigths = {
        let matrix = current_dataset.read().interaction_matrix(time_window());
        let node_weigths = matrix.sum_one_level();
        let m = node_weigths.matrix_max();
        node_weigths.matrix_map(|x| x / m)
    };

    rsx! {
        MyGraph {
            size: n,
            names: current_dataset.read().node_names().map(|x| Some(x.to_string())).collect(),
            node_classes: vec![vec![]; n],
            node_weights: node_weigths,
            edge_weights: edge_weigths,
            edges,
            positions
        }
    }
}

#[component]
fn InitialView() -> Element {
    rsx! {
        h1 {
            "Select your dataset with the box above"
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct ExplorerProps {
    link_stream: ReadOnlySignal<LinkStream>,
    initial_positions: ReadOnlySignal<Vec<Vec2>>,
    initial_time_window: ReadOnlySignal<Range<u64>>,
}


fn Explorer(props: ExplorerProps) -> Element {
    let visible_toogle = use_signal(|| false);
    let time_window = use_signal(|| props.initial_time_window.cloned());
    let positions = use_signal(|| props.initial_positions.cloned());
    let r_value = use_signal(|| 0.);

    let time = use_memo(move || {
        let Range { start, end } = time_window();
        let t = start as f64 + (end - start) as f64 * r_value();
        t as u64
    });

    let dt = use_memo(move || {
        let Range { start, end } = time_window();
        (end - start) / 100
    });

    rsx! {
        Reset {
            GraphView {
                current_dataset: props.link_stream,
                positions,
                t: time,
                dt,
                time_window
            }
            Menu {
                current_dataset: props.link_stream,
                visible_toogle,
                time_window,
                time,
                r_value
            }
        }
    }
}

#[component]
fn App(dataset_name: ReadOnlySignal<String>, dataset_path: ReadOnlySignal<String>) -> Element {
    tracing::info!("starting app");
    let mut view = use_signal(|| rsx! {});

    let _ = use_resource(move || async move {
        *view.write() = rsx! {
            LoadingGif {}
        };
        let name = dataset_name();
        let path = dataset_path();
        let data_text = reqwest::get(format!("{PUBLIC_URL}/{path}"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let dataset: LinkStreamData = serde_json::from_str(&data_text).unwrap();
        let (stream, positions) = load_linkstream_and_compute_positions(name, dataset).await;
        let time_window = stream.time_window();

        *view.write() = rsx! {
            Reset {
                Explorer { link_stream: stream, initial_positions: positions, initial_time_window: time_window }
            }
        }
    });

    rsx! {
        {view}
    }
}

#[component]
fn Home() -> Element {
    let dataset_paths: Signal<HashMap<String, &str>> = use_signal(|| {
        DATASETS
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
    });

    let mut current_dataset_name: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div { class: "dropdown-dataset-wrapper",
            select {
                id: "dataset-picker",
                class: "dropdown-dataset",
                value: "select your dataset",
                onchange: move |e: Event<FormData>| current_dataset_name.set(Some(e.value())),
                option { value: "", disabled: true, selected: true, "Select your dataset" }
                for (name , _) in DATASETS.iter() {
                    option {
                        value: *name,
                        "{name}"
                    }
                }
            }
        }
        match current_dataset_name() {
            Some(name) => rsx! {App {
                dataset_name: name.clone(),
                dataset_path: dataset_paths.read().get(&name).unwrap().to_string()
            }},
            None => rsx!{InitialView {}}
        }
    }
}
