use rand::{thread_rng, Rng};
use std::{ops::{Range, RangeInclusive}, thread, time::Duration};
use crate::Ranges;

#[derive(Default)]
pub enum TrainMode {
    #[default]
    Sequential,
    // Threaded,
    // GPU
}
#[derive(Default)]
pub struct TrainCondition<'a> {
    pub iteration: Option<usize>,
    pub condition: Option<&'a dyn Fn(&mut Vec<Net>) -> bool>,
    pub iter_init: bool,
    pub cond_init: bool,
}
impl<'a> TrainCondition<'a> {
    pub fn iteration(&mut self, value: usize) -> &mut Self {
        self.iteration = Some(value);
        self.iter_init = true;
        return self
    }
    pub fn condition(&mut self, value: &'a impl Fn(&mut Vec<Net>) -> bool) -> &mut Self {
        self.condition = Some(value);
        self.cond_init = true;
        return self
    }
    fn is_defined(&self) -> bool {
        if let Some(_) = self.iteration {
            return true
        }
        if let Some(_) = self.condition {
            return true
        }
        return false
    }
    pub fn check(&self, iteration: usize, nets: &mut Vec<Net>) -> bool {
        if let Some(threshold) = self.iteration {
            if threshold == iteration {
                return true
            }
        }
        if let Some(method) = self.condition {
            if method(nets) {
                return true
            }
        }
        return false
    }
}
#[derive(Default)]
pub struct Trainer<'a> {
    pub nets: Option<usize>,
    pub nodes: Option<usize>,
    pub layers: Option<usize>,
    pub inputs: Option<usize>,
    pub outputs: Option<usize>,
    pub best_net: Net,
    pub best_score: f32,
    pub delay: Duration,
    pub mode: Option<TrainMode>,
    pub max_change: Option<Ranges<f32>>,
    pub condition: TrainCondition<'a>,
}
impl<'a> Trainer<'a> {
    pub fn new() -> Trainer<'a> {
        return Default::default();
    }
    pub fn sequential(&mut self) -> &mut Self {
        self.mode = Some(TrainMode::Sequential);
        self
    }
    pub fn delay(&mut self, delay: Duration) -> &mut Self {
        self.delay = delay;
        self
    }
    pub fn max_change(&mut self, value: Range<f32>) -> &mut Self {
        self.max_change = Some(From::from(value));
        self
    }
    pub fn max_change_inclusive(&mut self, value: RangeInclusive<f32>) -> &mut Self {
        self.max_change = Some(From::from(value));
        self
    }
    pub fn nodes(&mut self, nodes: usize) -> &mut Self {
        self.nodes = Some(nodes);
        self
    }
    pub fn layers(&mut self, layers: usize) -> &mut Self {
        self.layers = Some(layers);
        self
    }
    pub fn nets(&mut self, nets: usize) -> &mut Self {
        self.nets = Some(nets);
        self
    }
    pub fn inputs(&mut self, inputs: usize) -> &mut Self {
        self.inputs = Some(inputs);
        self
    }
    pub fn outputs(&mut self, outputs: usize) -> &mut Self {
        self.outputs = Some(outputs);
        self
    }
    pub fn end_cond_iter(&mut self, threshold: usize) -> &mut Self {
        self.condition.iteration = Some(threshold);
        self
    }
    pub fn end_cond_method(&mut self, check: &'a impl Fn(&mut Vec<Net>) -> bool) -> &mut Self {
        self.condition.condition = Some(check);
        self
    }

    pub fn train(&mut self, method: impl Fn(&mut Vec<Net>)) {
        assert!(&self.mode.is_some(), "Training mode needs to be defined");
        assert!(&self.max_change.is_some(), "Maximum change needs to be defined");
        assert!(&self.nodes.is_some(), "Number of nodes needs to be defined");
        assert!(&self.layers.is_some(), "Number of layers needs to be defined");
        assert!(&self.nets.is_some(), "Number of nets needs to be defined");
        assert!(&self.inputs.is_some(), "Number of inputs needs to be defined");
        assert!(&self.outputs.is_some(), "Number of outputs needs to be defined");
        assert!(&self.condition.is_defined(), "Stop condition needs to be defnined");
        self.best_net = Net::new(
            self.nodes.unwrap(),
            self.layers.unwrap(),
            self.inputs.unwrap(),
            self.outputs.unwrap(),
        );
        self.best_score = 0.0;
        match self.mode.as_ref().unwrap() {
            TrainMode::Sequential => self.train_sequential(method),
        }
        println!("Best score: {}", self.best_score);
        println!("\n\n\n\nBest net: {:?}", self.best_net);
    }
    pub fn train_sequential(&mut self, method: impl Fn(&mut Vec<Net>)) {
        let mut iteration: usize = 0;
        let mut nets: Vec<Net> = vec![
            Net::new(
                self.nodes.unwrap(),
                self.layers.unwrap(),
                self.inputs.unwrap(),
                self.outputs.unwrap(),
            );
            self.nets.unwrap()
        ];
        loop {
            for net in nets.iter() {
                if net.score > self.best_score {
                    self.best_net = net.clone();
                }
            }
            for net in nets.iter_mut() {
                *net = self.best_net.clone();
                net.randomize_weights(self.max_change.as_ref().unwrap());
            }
            method(&mut nets);
            if self.condition.check(iteration, &mut nets) {
                break
            }
            iteration += 1;
            thread::sleep(self.delay);
        }
    }
}
#[derive(Clone, Default, Debug)]
pub struct Net {
    pub nodes: Vec<Node>,
    pub inputs: Vec<f32>,
    pub score: f32,
}
impl Net {
    pub fn get_outputs(&self) -> Vec<f32> {
        let mut out: Vec<f32> = Vec::new();
        for node in self.nodes.iter() {
            out.push(node.get_output(&self.inputs))
        }
        return out;
    }
    pub fn randomize_weights(&mut self, range: &Ranges<f32>) {
        for node in self.nodes.iter_mut() {
            node.randomize_weights(range)
        }
    }
    pub fn update_weights(&mut self, source: &Net, range: &Ranges<f32>) {
        self.nodes = source.nodes.clone();
        self.randomize_weights(range);
    }
    pub fn new(nodes: usize, layers: usize, inputs: usize, outputs: usize) -> Net {
        let mut out: Net = Default::default();
        for _ in 0..outputs {
            out.nodes.push(Node::new(nodes, layers, inputs))
        }
        for _ in 0..inputs {
            out.inputs.push(0.0)
        }
        return out;
    }
}
#[derive(Clone, Default, Debug)]
pub struct Node {
    pub inputs: Vec<(NodeType, f32)>,
}
impl Node {
    pub fn get_output(&self, inputs: &Vec<f32>) -> f32 {
        let mut out: f32 = 0.0;
        for (input, weight) in self.inputs.iter() {
            match input {
                NodeType::Node(node) => out += node.get_output(inputs) * weight,
                NodeType::Input(index) => out += inputs[*index] * weight,
            }
        }
        return out;
    }
    pub fn randomize_weights(&mut self, range: &Ranges<f32>) {
        for (input, weight) in self.inputs.iter_mut() {
            let mut rng = thread_rng();
            *weight += rng.gen_range(range.clone());
            if let NodeType::Node(node) = input {
                node.randomize_weights(range)
            }
        }
    }
    pub fn new(nodes: usize, layers: usize, inputs: usize) -> Node {
        let mut out: Node = Default::default();
        if layers == 0 {
            for input in 0..inputs {
                out.inputs.push((NodeType::Input(input), 1.0))
            }
            return out
        }
        for _ in 0..nodes {
            out.inputs
                .push((NodeType::Node(Node::new(nodes, layers - 1, inputs)), 1.0))
        }
        return out;
    }
}
#[derive(Clone, Debug)]
pub enum NodeType {
    Node(Node),
    Input(usize),
}
