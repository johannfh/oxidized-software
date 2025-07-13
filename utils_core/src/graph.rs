use std::{
    cmp::Ordering,
    collections::{HashMap, hash_map},
};

#[derive(Debug, Default)]
pub struct Graph<T> {
    nodes: HashMap<NodeID, Node<T>>,
    connections: Vec<(NodeID, NodeID, f64)>,
    next_id: i32,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            next_id: 0,
        }
    }

    fn get_next_id(&mut self) -> NodeID {
        let id = self.next_id;
        self.next_id += 1;
        id.into()
    }

    pub fn insert(&mut self, value: Node<T>) -> NodeID {
        let id = self.get_next_id();
        self.nodes.insert(id, value);
        return id;
    }

    pub fn connect(&mut self, from_id: NodeID, to_id: NodeID, weight: f64) {
        self.connections.push((from_id, to_id, weight));
    }

    pub fn connect_bidirectionally(&mut self, from_id: NodeID, to_id: NodeID, weight: f64) {
        self.connect(from_id, to_id, weight);
        self.connect(to_id, from_id, weight);
    }

    pub fn shortest_distance(&self, from_id: NodeID, to_id: NodeID) -> f64 {
        let mut adj: HashMap<NodeID, Vec<(NodeID, f64)>> = HashMap::new();

        for (f_id, t_id, weight) in &self.connections {
            adj.entry(*f_id)
                .or_insert_with(Vec::new)
                .push((*t_id, *weight));
        }

        println!("{:#?}", adj);

        todo!()
    }

    pub fn get_node(&self, id: &NodeID) -> Option<&Node<T>> {
        self.nodes.get(&id)
    }

    pub fn get_node_mut(&mut self, id: &NodeID) -> Option<&mut Node<T>> {
        self.nodes.get_mut(&id)
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            inner: self.nodes.iter(),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
struct State {
    cost: f64,
    position: NodeID,
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // smallest first for min-heap
        other.cost.partial_cmp(&self.cost)
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// An iterator of the Nodes of a Graph
pub struct Iter<'a, T> {
    inner: hash_map::Iter<'a, NodeID, Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a NodeID, &'a Node<T>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[derive(Debug)]
pub struct Node<T> {
    /// value of this node
    pub value: T,
}

impl<T> Node<T> {
    pub fn new(value: impl Into<T>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl<T> Default for Node<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            value: Default::default(),
        }
    }
}

impl<T> From<T> for Node<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NodeID(i32);

impl From<i32> for NodeID {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

