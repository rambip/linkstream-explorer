#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;
use async_std::task;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use kurbo::Vec2;
use std::ops::Range;
use tracing::info;
use std::collections::HashMap;
//use gloo_worker::oneshot::oneshot;
//use gloo_worker::Spawnable;
use rust_lapper::{Lapper, Interval};

mod force_directed_layout;
mod render_graph;

use render_graph::MyGraph;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(Home);
}




#[allow(non_snake_case)]
fn ToolBox() -> Element {
    rsx!{}
}

#[allow(non_snake_case)]
fn TimeSlider() -> Element {
    rsx!{}
}

#[component]
fn LoadingGif() -> Element {
    rsx!{
        div {
            class: "graph-container gif-container",
            img {
                src:"loading_state-optimize.gif",
                alt:"loading_gif",
                height:"250px",
                width:"320px"
            }
        }
    }
}

// TODO: utiliser `Interval`
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Link {
    n1: usize,
    n2: usize,
    start: u64,
    end: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
struct LinkStreamData {
    node_count: usize,
    node_names: Vec<String>,
    links: Vec<Link>,
    min_time: u64,
    max_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LinkStream {
    data: LinkStreamData,
    // TODO
    intervals: Lapper<u64, usize>,
    name: String,
}

impl LinkStream {
    fn interaction_matrix(&self, time_window: Range<u64>) -> Vec<Vec<f64>> {
        let n = self.data.node_count;
        let mut result = vec![vec![0.; n]; n];
        for it in self.intervals.find(time_window.start, time_window.end) {
            let i_link = it.val;
            let Link {n1, n2, start, end} = self.data.links[i_link];
            result[n1][n2] += (end - start) as f64;
            result[n2][n1] += (end - start) as f64;
        }
        result
    }

    fn data(&self) -> &LinkStreamData {
        &self.data
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn node_count(&self) -> usize {
        self.data.node_count
    }

    fn time_window(&self) -> Range<u64> {
        self.data.min_time..self.data.max_time
    }
}

trait Coeff : Copy + std::cmp::PartialOrd + core::iter::Sum {}

impl Coeff for u32 {}
impl Coeff for f64 {}
impl Coeff for f32 {}

trait Matrix : Clone {
    type Item: Coeff;
    type Row;
    fn matrix_map(&self, f: impl Fn(Self::Item) -> Self::Item + Clone) -> Self;
    fn matrix_max(&self) -> Self::Item;
    fn sum_one_level(&self) -> Self::Row;
}
 

impl<T> Matrix for Vec<T> 
where T: Coeff {
    type Item = T;
    type Row = T;
    fn matrix_map(&self, f: impl Fn(T) -> T + Clone) -> Self {
        self.into_iter().map(|x| f(*x)).collect()
    }
    fn matrix_max(&self) -> T {
        *self.into_iter().max_by(|a, b| T::partial_cmp(a, b).unwrap()).unwrap()
    }
    fn sum_one_level(&self) -> T {
        self.into_iter().map(|x| *x).sum()
    }
}

impl<T> Matrix for Vec<Vec<T>> 
where T: Coeff {
    type Item = T;
    type Row = Vec<T>;
    fn matrix_map(&self, f: impl Fn(T) -> T + Clone) -> Self {
        self.into_iter().map(|x| x.matrix_map(f.clone())).collect()
    }
    fn matrix_max(&self) -> Self::Item {
        self.into_iter()
            .map(|x| x.matrix_max())
            .max_by(|a, b| Self::Item::partial_cmp(a, b).unwrap()).unwrap()
    }
    fn sum_one_level(&self) -> Self::Row {
        self.into_iter().map(|x| x.sum_one_level()).collect()
    }
}



#[component]
fn Menu(
    current_dataset: ReadOnlySignal<LinkStream>,
    visible_toogle: Signal<bool>,
    ) -> Element {
    rsx!{
        div {
        class: "menu-container",
            div {
                span {
                    id: "current-time",
                    class: "current-time",
                }
            }
            div {
                class: "right-bar",
                ToolBox {}
                div {
                    id: "graph-info",
                    class: "graph-info",
                    div {
                        class: "rb-area tools",
                        h2 {
                            "Tools"
                        }
                        div {
                            class: "toolbox",
                            "icons" 
                        }
                        div {
                            class: "zoom-container",
                            "ZOOM stuff"
                        }
                    }
                    div {
                        class: "rb-area output",
                        h2 {
                            "Graph Stats"
                        }
                        div {
                            class: "data-output",
                            "TODO"
                        }
                    }
                }
            }
            TimeSlider {}
        }
    }
}

#[component]
fn Popup(children: Result<VNode, RenderError>) -> Element {
    rsx!{
        div {
            class: "dummy-container",
            {children}
        }
    }
}

static DATASETS: [(&'static str, &'static str); 3] = [
    ("baboon", include_str!("../baboon.json")),
    ("school", include_str!("../school.json")),
    ("example", include_str!("../example.json")),
];


//#[oneshot]
async fn LoadLinkstreamAndComputePositionsBg(name:String, data: &LinkStreamData) -> (LinkStream, Vec<Vec2>) {
    let n = data.node_count;
    let &LinkStreamData { min_time, max_time, ..} = data;
    let intervals = Lapper::new(
        data.links
        .iter()
        .enumerate()
        .map(|(i, l)| Interval {start: l.start, stop: l.end, val: i})
        .collect()
    );
    let link_stream = LinkStream {
        data: data.clone(),
        intervals,
        name
    };

    let matrix = link_stream.interaction_matrix(min_time..max_time);
    let m = matrix.matrix_max();
    let normalized_matrix = matrix.matrix_map(|x| x/m);

    let params = force_directed_layout::ForceDirectedLayoutParams {
        dt: 0.05,
        l_0: 0.03,
        k_r: 0.1,
        k_s: 0.02,
        n_iterations: 5,
        scale: 500.,
    };

    let positions = force_directed_layout::compute(n, &normalized_matrix, params);
    return ( link_stream, positions )
}

async fn load_linkstream_and_compute_positions(name: String, data: LinkStreamData) -> (LinkStream, Vec<Vec2>) {
    task::sleep(Duration::from_millis(1000)).await;
    LoadLinkstreamAndComputePositionsBg(name, &data).await
}

// FIXME: LinkStream n'a pas PartialEq et je ne sais pas comment faire
#[component]
fn GraphView(current_dataset: ReadOnlySignal<LinkStream>, time_window: ReadOnlySignal<Range<u64>>, positions: Vec<Vec2>) -> Element {
    let mut edges = Vec::new();
    let n = current_dataset.read().node_count();

    let matrix = current_dataset.read().interaction_matrix(time_window());
    let m = matrix.matrix_max();
    let normalized_matrix = matrix.matrix_map(|x| x/m);

    let node_weigths = matrix.sum_one_level();
    let m = node_weigths.matrix_max();
    let node_weigths_normalized = node_weigths.matrix_map(|x| x/m);

    for n1 in 0..n {
        for n2 in 0..n {
            if matrix[n1][n2] > 0. {
                edges.push((n1, n2));
            }
        }
    }

    rsx!{
        MyGraph {
            size: n,
            names: current_dataset.read().data.node_names.iter().map(|x| Some(x.clone())).collect(),
            node_classes: vec![vec![]; n],
            node_weights: node_weigths_normalized,
            edge_weights: normalized_matrix,
            edges,
            positions: positions,
        }
    }
}

#[component]
fn InitialView() -> Element {
    rsx!{
    }
}

#[component]
fn Explorer(name: ReadOnlySignal<String>, dataset: ReadOnlySignal<LinkStreamData>) -> Element {
    info!("render Explorer");
    let visible_toogle = use_signal(|| false);

    let mut time_window = use_signal(|| 0..0);
    let mut view = use_signal(|| rsx!{});


    // TODO: ne pas cloner le stream ?
    let _ = use_resource(move || async move {
        *view.write() = rsx!{ LoadingGif {} };
        let (stream, positions) = load_linkstream_and_compute_positions(name(), dataset()).await;
        *time_window.write() = stream.time_window();
        *view.write() = rsx!{
            GraphView {
                current_dataset: stream.clone(),
                positions: positions,
                time_window: time_window,
            }
            Menu {
                current_dataset: stream,
                visible_toogle: visible_toogle
            }
        }
    });

    rsx!{
        {view}
    }
}

#[component]
fn Home() -> Element {
    let datasets: Signal<HashMap<String, LinkStreamData>> = use_signal(
        || DATASETS.into_iter()
        .map(|(k, v)| (k.to_string(), serde_json::from_str(v).unwrap()))
        .collect()
    );

    let mut current_dataset_name = use_signal(|| None);

    rsx!{

        link { rel: "stylesheet", href: "home.css" }
        link { rel: "stylesheet", href: "menu.css" }
        link { rel: "stylesheet", href: "popup.css" }
        link { rel: "stylesheet", href: "variables.css" }


        div {
            class: "dropdown-dataset-wrapper",
            select {
                id: "dataset-picker",
                class: "dropdown-dataset",
                value: "select your dataset",
                for (name, _) in DATASETS.iter() {
                    option{
                        value: *name, 
                        onclick: move |_| *current_dataset_name.write() = Some(name),
                        "{name}"
                    }
                }
            }
        }
        match current_dataset_name() {
            Some(name) => rsx! {Explorer {
                name: *name,
                dataset: datasets.read().get(*name).unwrap().clone()
            }},
            None => rsx!{InitialView {}}
        }
        Popup {
        }
    }
}
