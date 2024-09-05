#![allow(non_snake_case)]

use crate::linkstream::{LinkStream, LinkStreamData};
use crate::utils::Matrix;
use dioxus::prelude::*;
use kurbo::Vec2;
use std::collections::HashMap;
use std::ops::Range;
use tracing::info;
use tracing::Level;
use uuid::Uuid;

mod force_directed_layout;
mod linkstream;
mod render_graph;
mod svg_timeline;
mod utils;

use svg_timeline::SvgTimeLine;

use render_graph::MyGraph;

// TODO: use `Dioxus.toml` to get this path.
// or read the `reqwest` docs for wasm ?
//const PUBLIC_URL : &str = "https://rambip.github.io/linkstream-explorer";
#[cfg(debug_assertions)]
const PUBLIC_URL: &str = "http://localhost:8080/linkstream-explorer";

#[cfg(not(debug_assertions))]
const PUBLIC_URL: &str = "https://rambip.github.io/linkstream-explorer";

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(Home);
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Direction {
    Right,
    Left,
}

#[component]
fn Arrow(direction: Direction, onclick: EventHandler<MouseEvent>) -> Element {
    let class_name = match direction {
        Direction::Right => "arrow-right",
        Direction::Left => "arrow-left",
    };

    let rotation_deg = match direction {
        Direction::Right => -90,
        Direction::Left => 90,
    };

    rsx! {
        svg {
            width: "24",
            height: "24",
            view_box: "0 0 512 512",
            fill: "none",
            class: class_name,
            onclick: move |e| onclick.call(e),
            path {
                d: "M256 294.1L82.8 121.6C72.5 111.6 56.7 111.6 46.4 121.6C36.1 131.6 36.1 147.4 46.4 157.4L243.2 352.4C253.5 362.4 269.3 362.4 279.6 352.4L476.4 157.4C486.7 147.4 486.7 131.6 476.4 121.6C466.1 111.6 450.3 111.6 440 121.6L256 294.1Z",
                fill: "currentColor",
                transform: "rotate({rotation_deg} 256 256)"
            }
        }
    }
}

#[component]
fn TimeSlider(
    time_window: Signal<Range<u64>>,
    current_dataset: ReadOnlySignal<LinkStream>,
    zoom: ReadOnlySignal<f64>,
    r_value: Signal<f64>,
    time: ReadOnlySignal<u64>,
) -> Element {
    // ne change que quand le zoom a lieu.
    use_effect(move || {
        info!("update");
        let dataset_window = current_dataset.read().time_window();
        let dataset_w = (dataset_window.end - dataset_window.start) as f64;
        let new_w = dataset_w * (10f64).powf(-zoom());

        let t = *time.peek() as f64;
        let mut new_start: f64 = t - new_w / 2.;
        let mut new_end: f64 = t + new_w / 2.;

        if new_start < (dataset_window.start as f64) {
            new_start = dataset_window.start as f64;
            new_end = dataset_window.start as f64 + new_w;
        }

        if new_end > (dataset_window.end as f64) {
            new_end = dataset_window.end as f64;
            new_start = dataset_window.end as f64 - new_w;
        }

        time_window.set(new_start as u64..new_end as u64);
        let new_v = (t - new_start) / new_w;
        r_value.set(new_v);
    });

    let mut intensities = Vec::new();

    let Range { start, end } = time_window();
    let dt = (end - start) / 100;
    for i in 0..100 {
        let time_point = start + i * dt;
        let intensity = current_dataset
            .read()
            .interaction_score_during(time_point as u64..time_point as u64 + dt);
        intensities.push(intensity)
    }

    let m = intensities.matrix_max();
    let intensities = intensities.matrix_map(|x| x / m);
    let empty = m == 0.;

    let mut translate_window = move |p: f64| {
        let dataset_window = current_dataset.read().time_window();
        let Range { start, end } = time_window();
        let w = end - start;
        let dt = (p * w as f64) as i64;

        let mut new_start = (start as i64 + dt) as u64;
        let mut new_end = (end as i64 + dt) as u64;

        if new_start < (dataset_window.start) {
            new_start = dataset_window.start;
            new_end = dataset_window.start + w;
        }

        if new_end > (dataset_window.end) {
            new_end = dataset_window.end;
            new_start = dataset_window.end - w;
        }

        time_window.set(new_start..new_end)
    };

    rsx! {
        div { class: "time-line-container",
            Arrow { onclick: move |_| translate_window(-0.1), direction: Direction::Left }
            input {
                r#type: "range",
                value: "{r_value}",
                min: "0",
                max: "1",
                step: "any",
                oninput: move |e| r_value.set(e.value().parse().unwrap()),
                class: "time-slider"
            }

            div { class: "svg-container",
                SvgTimeLine { n_bar: 100, intensities, empty }
            }

            Arrow { onclick: move |_| translate_window(0.1), direction: Direction::Right }
        }
    }
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

// FIXME: LinkStream n'a pas PartialEq et je ne sais pas comment faire
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
    rsx! {}
}

#[derive(PartialEq, Clone, Props)]
struct ExplorerProps {
    link_stream: ReadOnlySignal<LinkStream>,
    initial_positions: ReadOnlySignal<Vec<Vec2>>,
    initial_time_window: ReadOnlySignal<Range<u64>>,
}

#[component]
fn Reset(children: Element) -> Element {
    rsx!{{std::iter::once(
        rsx! {
            div {
                key: "{Uuid::new_v4()}",
                {children}
            }
        }
    )
    }}
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
