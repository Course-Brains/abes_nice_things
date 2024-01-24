use std::{
    time::Duration,
    thread,
    path::{Path, PathBuf},
    cell::OnceCell,
    fs::metadata, ops::RangeBounds,
    sync::Mutex,
};

use crate::{
    file_ops::{load_bin, save_bin},
    Ranges,
};

use serde::{Deserialize, Serialize};
use rand::{thread_rng, Rng};

pub type Value = f32;

pub struct Trainer<'a> {
    nodes: OnceCell<usize>,
    layers: OnceCell<usize>,
    nets: usize,
    inputs: OnceCell<usize>,
    outputs: OnceCell<usize>,
    delay: Option<Duration>,
    mode: OnceCell<TrainMode>,
    condition: OnceCell<EndCondition<'a>>,
    source: Option<PathBuf>,
    max_change: OnceCell<Ranges<Value>>,
    max_weight: Option<Ranges<Value>>
}
impl<'a> Trainer<'a> {
    pub fn new() -> Self {
        return Default::default()
    }
    pub fn nodes(&mut self, nodes: usize) -> &mut Self {
        self.nodes.set(nodes).unwrap();
        return self
    }
    pub fn layers(&mut self, layers: usize) -> &mut Self {
        self.layers.set(layers).unwrap();
        return self
    }
    pub fn nets(&mut self, nets: usize) -> &mut Self {
        self.nets = nets;
        return self
    }
    pub fn inputs(&mut self, inputs: usize) -> &mut Self {
        self.inputs.set(inputs).unwrap();
        return self
    }
    pub fn outputs(&mut self, outputs: usize) -> &mut Self {
        self.outputs.set(outputs).unwrap();
        return self
    }
    pub fn delay(&mut self, delay: Duration) -> &mut Self {
        self.delay = Some(delay);
        return self
    }
    pub fn sequential(&mut self) -> &mut Self {
        self.mode.set(TrainMode::Sequential).unwrap();
        return self
    }
    pub fn thread(&mut self, threads: usize) -> &mut Self {
        self.mode.set(TrainMode::Threaded(threads)).unwrap();
        return self
    }
    pub fn cond_iter(&mut self, threshold: usize) -> &mut Self {
        self.condition.set(EndCondition::Iteration(threshold)).unwrap();
        return self
    }
    pub fn cond_method(
        &mut self,
        method: &'a (dyn (Fn(&[Net]) -> bool) + std::marker::Send + std::marker::Sync),
    ) -> &mut Self {
        self.condition.set(EndCondition::Method(method)).unwrap();
        return self
    }
    pub fn source<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.source = Some(path.as_ref().to_path_buf());
        return self
    }
    pub fn max_change(&mut self, range: std::ops::Range<Value>) -> &mut Self {
        self.max_change.set(range.into()).unwrap();
        return self
    }
    pub fn max_change_inclusive(&mut self, range: std::ops::RangeInclusive<Value>) -> &mut Self {
        self.max_change.set(range.into()).unwrap();
        return self
    }
    pub fn max_weight(&mut self, range: std::ops::Range<Value>) -> &mut Self {
        self.max_weight = Some(range.into());
        return self
    }
    pub fn max_weight_inclusive(&mut self, range: std::ops::RangeInclusive<Value>) -> &mut Self {
        self.max_weight = Some(range.into());
        return self
    }
    pub fn train(&self, method: impl Fn(&mut [Net], &mut Net) + std::marker::Send + std::marker::Copy) {
        assert_ne!(self.nodes.get().unwrap(), &0, "Number of nodes cannot be 0");
        assert_ne!(self.layers.get().unwrap(), &0, "Number of layers cannot be 0");
        assert_ne!(self.nets, 0, "Number of nets cannot be 0");
        assert_ne!(self.inputs.get().unwrap(), &0, "Number of inputs cannot be 0");
        assert_ne!(self.outputs.get().unwrap(), &0, "Number of outputs cannot be 0");
        let mut best: Option<Net> = None;
        if let Some(path) = &self.source {
            if let Ok(_) = metadata(path) {
                best = Some(load_bin(path));
            }
        }
        let best = best.unwrap_or_else(|| {
            Net::new(
                *self.nodes.get().expect("Number of nodes must be defined"),
                *self.layers.get().expect("Number of layers must be defined"),
                *self.inputs.get().expect("number of inputs must be defined"),
                *self.outputs.get().expect("Number of outputs must be defined"),
            )
        });
        match self.mode.get().unwrap() {
            TrainMode::Sequential => self.sequential_train(best, method),
            TrainMode::Threaded(threads) => self.thread_train(best, method, *threads)
        }
    }
    fn sequential_train(&self, best: Net, method: impl Fn(&mut [Net], &mut Net)) {
        let mut nets: Vec<Net>;
        let mut iter: usize = 0;
        let max_change: &Ranges<Value> = self.max_change.get().unwrap();
        let max_weight: &Option<Ranges<Value>> = &self.max_weight;
        let mut best = best;
        loop {
            nets = Vec::new();
            for _ in 0..self.nets {
                nets.push(best.clone());
            }
            for net in nets.iter_mut() {
                net.randomize_weights(max_change, max_weight);
            }
            method(&mut nets, &mut best);
            for net in nets.iter() {
                if net.score > best.score {
                    best = net.clone();
                }
            }
            if self.condition.get().unwrap().check(&iter, &nets) {
                break
            }
            iter += 1;
            if let Some(delay) = self.delay {
                thread::sleep(delay);
            }
        }
        if let Some(path) = &self.source {
            save_bin(path, &best);
        }
    }
    fn thread_train(
        &self,
        best: Net,
        method: impl Fn(&mut [Net], &mut Net) + std::marker::Send + std::marker::Copy,
        threads: usize
    ) {
        let num_nets = self.nets;
        let best_mutex = &Mutex::new(best);
        let max_change = self.max_change.get().unwrap();
        let max_weight = &self.max_weight;
        let cond = self.condition.get().unwrap();
        thread::scope(|s| {
            for _ in 0..threads {
                let cond = cond.clone();
                s.spawn(move || {
                    let mut nets: Vec<Net> = Vec::new();
                    let mut best = best_mutex.lock().unwrap().to_owned();
                    for _ in 0..num_nets {
                        nets.push(best.to_owned())
                    }
                    let mut iter = 0;
                    loop {
                        nets = Vec::new();
                        for _ in 0..num_nets {
                            let mut current = best.to_owned();
                            current.randomize_weights(max_change, &max_weight);
                            nets.push(current);
                        }
                        method(&mut nets, &mut best);
                        for net in nets.iter() {
                            if net.score > best.score {
                                best = net.to_owned()
                            }
                        }
                        *best_mutex.lock().unwrap() = best.to_owned();
                        if cond.check(&iter, &nets) {
                            break
                        }
                        iter += 1;
                    }
                });
            }
        })
    }
}
impl Default for Trainer<'_> {
    fn default() -> Self {
        return Trainer {
            nodes: OnceCell::new(),
            layers: OnceCell::new(),
            nets: 1,
            inputs: OnceCell::new(),
            outputs: OnceCell::new(),
            delay: None,
            mode: OnceCell::new(),
            condition: OnceCell::new(),
            source: None,
            max_change: OnceCell::new(),
            max_weight: None
        }
    }
}
#[derive(Clone, Deserialize, Serialize, Default, PartialEq, Debug)]
pub struct Net {
    nodes: Vec<Vec<Node>>,
    outputs: Vec<Node>,
    pub inputs: Vec<Value>,
    pub score: Value,
}
impl Net {
    pub fn new(nodes: usize, layers: usize, inputs: usize, outputs: usize) -> Net {
        let mut node_list: Vec<Vec<Node>> = Vec::new();
        let mut temp: Vec<Node> = Vec::new();
        for _ in 0..nodes {
            temp.push(Node::new(inputs))
        }
        node_list.push(temp);
        for _ in 1..layers {
            let mut temp: Vec<Node> = Vec::new();
            for _ in 0..nodes {
                temp.push(Node::new(nodes))
            }
            node_list.push(temp)
        }
        let mut output_list: Vec<Node> = Vec::new();
        for _ in 0..outputs {
            output_list.push(Node::new(nodes))
        }
        return Net {
            nodes: node_list,
            outputs: output_list,
            inputs: Vec::new(),
            score: Default::default()
        }
    }
    pub fn set_inputs(&mut self, value: Vec<Value>) {
        self.inputs = value;
    }
    pub fn get_outputs(&mut self) -> Vec<Value> {
        let mut previous: Vec<Value> = self.inputs.to_owned();
        let mut current: Vec<Value> = Vec::new();
        current.reserve(self.nodes.len().max(self.outputs.len()));
        // nodes
        for node_list in self.nodes.iter() {
            current = Vec::new();
            for node in node_list.iter() {
                current.push(node.gen_value(&previous))
            }
            previous = current;
        }
        debug_assert_eq!(
            previous.len(), self.nodes.len(),
            "Outputs: {}, nodes: {}", previous.len(), self.nodes.len()
        );
        // outputs
        current = Vec::new();
        for output in self.outputs.iter() {
            current.push(output.gen_value(&previous))
        }
        debug_assert_eq!(current.len(), self.outputs.len());
        return current
    }
    fn randomize_weights(&mut self, max_change: &Ranges<Value>, max_weight: &Option<Ranges<Value>>) {
        for output in self.outputs.iter_mut() {
            output.randomize_weights(max_change, max_weight)
        }
        for nodes in self.nodes.iter_mut() {
            for node in nodes.iter_mut() {
                node.randomize_weights(max_change, max_weight)
            }
        }
    }
}
#[derive(Default, Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct Node {
    weights: Vec<Value>,
}
impl Node {
    const DEFAULT_WEIGHT: Value = 1.0;
    fn new(inputs: usize) -> Node {
        let mut weights: Vec<Value> = Vec::new();
        for _ in 0..inputs {
            weights.push(Node::DEFAULT_WEIGHT)
        }
        return Node {
            weights
        }
    }
    pub fn gen_value(&self, inputs: &[Value]) -> Value {
        debug_assert_eq!(self.weights.len(), inputs.len(), "Weights and inputs are not same length");
        let mut value: Value = Default::default();
        let sum = self.weights.iter().sum::<Value>();
        for (index, weight) in self.weights.iter().enumerate() {
            debug_assert_ne!(weight, &Value::INFINITY, "Weight was infinity");
            value += (inputs[index] * weight)/sum;
        }
        debug_assert_ne!(value, Value::INFINITY, "Node generated infinity");
        return value
    }
    fn randomize_weights(&mut self, max_change: &Ranges<Value>, max_weight: &Option<Ranges<Value>>) {
        for weight in self.weights.iter_mut() {
            match max_weight {
                Some(max_weight) => {
                    let mut value = weight.to_owned();
                    loop {
                        let mut rng = thread_rng();
                        value += rng.gen_range(max_change.to_owned());
                        if max_weight.contains(&value) {
                            *weight = value;
                            break
                        }
                    }
                }
                None => {
                    let mut rng = thread_rng();
                    *weight += rng.gen_range(max_change.to_owned());
                }
            }
        }
    }
}

#[derive(Debug)]
enum TrainMode {
    Sequential,
    Threaded(usize),
}
enum EndCondition<'a> {
    Iteration(usize),
    Method(&'a (dyn Fn(&[Net]) -> bool + std::marker::Send + std::marker::Sync)),
}
impl EndCondition<'_> {
    fn check(&self, iter: &usize, nets: &[Net]) -> bool {
        match self {
            EndCondition::Iteration(threshold) => {
                return iter > threshold
            }
            EndCondition::Method(method) => {
                return method(nets)
            }
        }
    }
}
impl std::fmt::Debug for EndCondition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iteration(arg0) => f.debug_tuple("Iteration").field(arg0).finish(),
            Self::Method(_) => f.debug_tuple("Method").finish(),
        }
    }
}