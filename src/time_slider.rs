use dioxus::prelude::*;
use std::ops::Range;
use crate::LinkStream;
use crate::utils::Matrix;
use crate::SvgTimeLine;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Direction {
    Right,
    Left,
}

#[component]
pub fn Arrow(direction: Direction, onclick: EventHandler<MouseEvent>) -> Element {
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
pub fn TimeSlider(
    time_window: Signal<Range<u64>>,
    current_dataset: ReadOnlySignal<LinkStream>,
    zoom: ReadOnlySignal<f64>,
    r_value: Signal<f64>,
    time: ReadOnlySignal<u64>,
) -> Element {
    // ne change que quand le zoom a lieu.
    use_effect(move || {
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
