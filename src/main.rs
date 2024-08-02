#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;
use async_std::task;
use std::time::Duration;
use cytoscape::Cytoscape;
use serde::{Serialize, Deserialize};
use kurbo::Vec2;
use std::ops::Range;
use tracing::info;
use std::collections::HashMap;
//use gloo_worker::oneshot::oneshot;
//use gloo_worker::Spawnable;
use rust_lapper::{Interval};

mod cytoscape;
mod force_directed_layout;
mod graph_to_cytoscape;

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(Home);
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Graph {
    pub size: usize,
    pub names: Vec<Option<String>>,
    pub node_weights: Option<Vec<f64>>,
    pub node_classes: Vec<Vec<String>>,
    pub edges: Vec<(usize, usize)>,
    pub edge_weights: Option<Vec<Vec<f64>>>,
}



#[allow(non_snake_case)]
fn ToolBox() -> Element {
    rsx!{}
}

#[allow(non_snake_case)]
fn TimeSlider() -> Element {
    rsx!{}
}

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

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
struct LinkStream {
    data: LinkStreamData,
    // TODO
    // intervals: interval_tree::IntervalTree,
}

impl LinkStream {
    fn interaction_matrix_naive(&self, time_window: Range<u64>) -> Vec<Vec<f64>> {
        let n = self.data.node_count;
        let mut result = vec![vec![0.; n]; n];
        for &Link {n1, n2, start, end} in &self.data.links {
            if !(time_window.start > end || start  > time_window.end) {
                result[n1][n2] += (end - start) as f64;
                result[n2][n1] += (end - start) as f64;
            }
        }
        result
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



fn generate_graph(link_stream: &LinkStream, time_window: Range<u64>) -> Graph {
    let mut edges = Vec::new();
    let n = link_stream.node_count();

    let matrix = link_stream.interaction_matrix_naive(time_window);
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


    Graph {
        size: n,
        names: link_stream.data.node_names.iter().map(|x| Some(x.clone())).collect(),
        node_classes: vec![vec![]; n],
        node_weights: Some(node_weigths_normalized),
        edge_weights: Some(normalized_matrix),
        edges
    }
}

#[component]
fn Menu(
    visible_toogle: Signal<bool>,
    current_dataset: LinkStream,
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
fn Popup(children: Vec<Element>) -> Element {
    rsx!{
        div {
            class: "dummy-container",
            {children.into_iter()}
        }
    }
}

#[derive(Debug)]
enum State {
    Initial,
    Exploring(String)
}

static DATASETS: [(&'static str, &'static str); 2] = [
    ("baboon", include_str!("../baboon.json")),
    ("school", include_str!("../school.json")),
];

static STATE: GlobalSignal<State> = Signal::global(
    || State::Initial,
);


//#[oneshot]
async fn LoadLinkstreamAndComputePositionsBg(data: LinkStreamData) -> (LinkStream, Vec<Vec2>) {
    let n = data.node_count;
    let LinkStreamData { min_time, max_time, ..} = data;
    let link_stream = LinkStream {data};

    let matrix = link_stream.interaction_matrix_naive(min_time..max_time);
    let m = matrix.matrix_max();
    let normalized_matrix = matrix.matrix_map(|x| x/m);

    let params = force_directed_layout::ForceDirectedLayoutParams {
        dt: 0.05,
        l_0: 0.03,
        k_r: 0.1,
        k_s: 0.02,
        n_iterations: 500,
        scale: 1000.,
    };

    let positions = force_directed_layout::compute(n, &normalized_matrix, params);
    return ( link_stream, positions )
}

async fn load_linkstream_and_compute_positions(data: LinkStreamData) -> (LinkStream, Vec<Vec2>) {
    //tokio::spawn(
        LoadLinkstreamAndComputePositionsBg(data).await
    //).await.unwrap()
}

#[component]
fn GraphView(current_dataset: ReadOnlySignal<LinkStream>, time_window: ReadOnlySignal<Range<u64>>, positions: ReadOnlySignal<Vec<Vec2>>) -> Element {
    let graph = use_memo(
        move || generate_graph(&current_dataset.read(), time_window.read().clone()).to_cytoscape(&positions.read())
    );

    rsx!{
        Cytoscape {
            data: graph,
            style: graph_to_cytoscape::create_style(),
            pan: Vec2::new(0., 0.)
        }
    }
}

#[component]
fn InitialView() -> Element {
    rsx!{
    }
}

#[component]
fn Explorer(dataset: LinkStreamData) -> Element {
    let visible_toogle = use_signal(|| true);

    let time_window = use_signal(|| dataset.min_time..dataset.max_time);

    let linkstream_resource = use_resource(
        move || load_linkstream_and_compute_positions(
            dataset.clone(),
        )
    );

    rsx! {
        match linkstream_resource.read().clone() {
            Some((g, p)) => rsx!{
                GraphView {
                    current_dataset: g.clone(),
                    time_window: time_window,
                    positions: p,
                }
                Menu {
                    current_dataset: g,
                    visible_toogle
                }
            },
            None => rsx!{
                LoadingGif {}
            }
        }
    }
}

#[component]
fn Home() -> Element {
    let datasets: Signal<HashMap<String, LinkStreamData>> = use_signal(
        || DATASETS.into_iter()
        .map(|(k, v)| (k.to_string(), serde_json::from_str(v).unwrap()))
        .collect()
    );

    use_effect(|| {
        info!("{:?}", STATE.read())
    });


    let options = DATASETS.iter()
        .map(|(name, _)| name.to_owned())
        .map(|s| rsx!{
            option{
                value: s, 
                onclick: move |_| *STATE.write() = State::Exploring(s.to_string()),
                "{s}",
            }
        });

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
                {options}
            }
        }

        match &*STATE.read() {
            State::Initial => rsx!{
                InitialView {}
            },
            State::Exploring(x) => rsx!{
                Explorer {
                    dataset: datasets.read().get(x).unwrap().clone(),
                }
            },
        }
        Popup {
        }
    }
}
