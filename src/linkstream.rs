use rust_lapper::{Interval, Lapper};
use serde::{Deserialize, Serialize};
use std::ops::Range;

// TODO: utiliser `Interval`
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Link {
    n1: usize,
    n2: usize,
    start: u64,
    end: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LinkStreamData {
    node_count: usize,
    node_names: Vec<String>,
    links: Vec<Link>,
    min_time: u64,
    max_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LinkStream {
    data: LinkStreamData,
    // TODO
    intervals: Lapper<u64, usize>,
    name: String,
}

impl LinkStream {
    pub fn new(name: String, data: LinkStreamData) -> Self {
        let intervals = Lapper::new(
            data.links
                .iter()
                .enumerate()
                .map(|(i, l)| Interval {
                    start: l.start,
                    stop: l.end,
                    val: i,
                })
                .collect(),
        );

        Self {
            data,
            intervals,
            name,
        }
    }

    fn links_during(&self, time_window: Range<u64>) -> impl Iterator<Item = Link> + '_ {
        self.intervals
            .find(time_window.start, time_window.end)
            .map(|it| self.data.links[it.val])
    }

    pub fn interaction_matrix(&self, time_window: Range<u64>) -> Vec<Vec<f64>> {
        let n = self.data.node_count;
        let mut result = vec![vec![0.; n]; n];
        for Link { n1, n2, start, end } in self.links_during(time_window) {
            result[n1][n2] += (end - start) as f64;
            result[n2][n1] += (end - start) as f64;
        }
        result
    }

    pub fn interaction_score_during(&self, time_window: Range<u64>) -> f64 {
        let mut score = 0.;
        for Link {
            n1: _,
            n2: _,
            start,
            end,
        } in self.links_during(time_window)
        {
            score += (end - start) as f64;
        }
        score
    }

    pub fn node_names(&self) -> impl Iterator<Item = &str> {
        self.data.node_names.iter().map(|x| x as _)
    }

    fn data(&self) -> &LinkStreamData {
        &self.data
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn node_count(&self) -> usize {
        self.data.node_count
    }

    pub fn time_window(&self) -> Range<u64> {
        self.data.min_time..self.data.max_time
    }
}
