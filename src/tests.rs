use std::collections::HashMap;

use crate::{
    MulticriteriaDijkstra, MulticriteriaDijkstraContext,
    node::{MulticriteriaDijkstraCost, MulticriteriaDijkstraIdentifier, MulticriteriaDijkstraNode},
    prune_list::{
        IndexedNode, MulticriteriaDijkstraPruneList, NodeWithIsTarget, NodeWithOptionalPredecessor,
        NodeWithOptionalPredecessorAndIsTarget,
    },
};

#[derive(Debug, Default)]
struct PruneList {
    active_nodes: HashMap<Identifier, Vec<usize>>,
    nodes: Vec<PruneListNode>,
    target_nodes: Vec<usize>,
}

#[derive(Debug)]
struct PruneListNode {
    node: Node,
    predecessor_index: Option<usize>,
    open: bool,
    pruned: bool,
}

struct Context {
    sequence_a: Vec<u8>,
    sequence_b: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Node {
    identifier: Identifier,
    cost: Cost,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Identifier {
    position_a: usize,
    position_b: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Cost {
    hops: usize,
    cost: usize,
}

impl MulticriteriaDijkstraPruneList for PruneList {
    type Node = Node;

    type Index = usize;

    fn insert(
        &mut self,
        NodeWithOptionalPredecessorAndIsTarget {
            node,
            predecessor_index,
            is_target,
        }: NodeWithOptionalPredecessorAndIsTarget<Self::Node, Self::Index>,
    ) -> Option<Self::Index> {
        // Check if node is dominated.
        for &index in self
            .active_nodes
            .get(&node.identifier)
            .unwrap_or(&Vec::new())
            .iter()
            .chain(self.target_nodes.iter())
        {
            if self.nodes[index].node.dominates(&node) {
                return None;
            }
        }

        // Prune what it dominates.
        if is_target {
            // Prune all open nodes that are dominated by this node.
            for vec in self.active_nodes.values_mut() {
                vec.retain(|&index| {
                    let result = !node.dominates(&self.nodes[index].node);
                    self.nodes[index].pruned |= !result;
                    result
                });
            }
        } else {
            // Prune only open nodes with the same identifier that are dominated by this node.
            if let Some(vec) = self.active_nodes.get_mut(&node.identifier) {
                vec.retain(|&index| {
                    let result = !node.dominates(&self.nodes[index].node);
                    self.nodes[index].pruned |= !result;
                    result
                });
            }
        }

        // Update target nodes.
        self.target_nodes.retain(|&index| !self.nodes[index].pruned);

        // Insert node.
        let index = self.nodes.len();
        let identifier = node.identifier.clone();
        self.nodes.push(PruneListNode {
            node,
            predecessor_index,
            open: true,
            pruned: false,
        });
        self.active_nodes.entry(identifier).or_default().push(index);
        if is_target {
            self.target_nodes.push(index);
        }

        Some(index)
    }

    fn next(&mut self) -> Option<IndexedNode<&Self::Node, Self::Index>> {
        for (index, node) in self.nodes.iter_mut().enumerate() {
            if node.open && !node.pruned {
                node.open = false;
                return Some(IndexedNode {
                    node: &node.node,
                    index,
                });
            }
        }
        None
    }

    fn get(
        &self,
        index: Self::Index,
    ) -> Option<NodeWithOptionalPredecessor<&Self::Node, Self::Index>> {
        self.nodes
            .get(index)
            //.filter(|node| !node.pruned)
            .map(|node| NodeWithOptionalPredecessor {
                node: &node.node,
                predecessor_index: node.predecessor_index,
            })
    }

    fn iter_nodes_with_predecessor_at_identifier<'this, 'identifier>(
        &'this self,
        identifier: &'identifier Identifier,
    ) -> impl use<'this> + Iterator<Item = NodeWithOptionalPredecessor<&'this Self::Node, Self::Index>>
    where
        Self::Node: 'this,
    {
        self.active_nodes
            .get(identifier)
            .map(|vec| {
                vec.iter().filter_map(move |&index| {
                    self.nodes
                        .get(index)
                        .filter(|node| !node.pruned)
                        .map(|node| NodeWithOptionalPredecessor {
                            node: &node.node,
                            predecessor_index: node.predecessor_index,
                        })
                })
            })
            .into_iter()
            .flatten()
    }

    fn clear(&mut self) {
        *self = Default::default();
    }

    fn supports_backtracking(&self) -> bool {
        true
    }
}

impl
    Extend<
        NodeWithOptionalPredecessorAndIsTarget<
            <Self as MulticriteriaDijkstraPruneList>::Node,
            <Self as MulticriteriaDijkstraPruneList>::Index,
        >,
    > for PruneList
{
    fn extend<
        T: IntoIterator<
            Item = NodeWithOptionalPredecessorAndIsTarget<
                <Self as MulticriteriaDijkstraPruneList>::Node,
                <Self as MulticriteriaDijkstraPruneList>::Index,
            >,
        >,
    >(
        &mut self,
        iter: T,
    ) {
        for node in iter {
            self.insert(node);
        }
    }
}

impl Extend<NodeWithIsTarget<<Self as MulticriteriaDijkstraPruneList>::Node>> for PruneList {
    fn extend<
        T: IntoIterator<Item = NodeWithIsTarget<<Self as MulticriteriaDijkstraPruneList>::Node>>,
    >(
        &mut self,
        iter: T,
    ) {
        for node in iter {
            self.insert(NodeWithOptionalPredecessorAndIsTarget {
                node: node.node,
                predecessor_index: None,
                is_target: node.is_target,
            });
        }
    }
}

impl MulticriteriaDijkstraContext for Context {
    type Node = Node;

    fn generate_root_nodes(&self, output: &mut impl Extend<NodeWithIsTarget<Self::Node>>) {
        output.extend(Some(NodeWithIsTarget {
            node: Node {
                identifier: Identifier {
                    position_a: 0,
                    position_b: 0,
                },
                cost: Cost { hops: 0, cost: 0 },
            },
            is_target: self.sequence_a.is_empty() && self.sequence_b.is_empty(),
        }));
    }

    fn generate_successors<Index: Copy>(
        &self,
        IndexedNode { node, index }: IndexedNode<Self::Node, Index>,
        output: &mut impl Extend<NodeWithOptionalPredecessorAndIsTarget<Self::Node, Index>>,
    ) {
        let na = node.identifier.position_a;
        let nb = node.identifier.position_b;

        if na < self.sequence_a.len() && nb < self.sequence_b.len() {
            // Match or mismatch.
            let cost = if self.sequence_a[na] == self.sequence_b[nb] {
                0
            } else {
                3
            };
            output.extend(Some(NodeWithOptionalPredecessorAndIsTarget {
                node: Node {
                    identifier: Identifier {
                        position_a: na + 1,
                        position_b: nb + 1,
                    },
                    cost: Cost {
                        hops: node.cost.hops + 1,
                        cost: node.cost.cost + cost,
                    },
                },
                predecessor_index: Some(index),
                is_target: na + 1 == self.sequence_a.len() && nb + 1 == self.sequence_b.len(),
            }));
        }

        if na < self.sequence_a.len() {
            // Deletion.
            output.extend(Some(NodeWithOptionalPredecessorAndIsTarget {
                node: Node {
                    identifier: Identifier {
                        position_a: na + 1,
                        position_b: nb,
                    },
                    cost: Cost {
                        hops: node.cost.hops + 1,
                        cost: node.cost.cost + 1,
                    },
                },
                predecessor_index: Some(index),
                is_target: na + 1 == self.sequence_a.len() && nb == self.sequence_b.len(),
            }));
        }

        if nb < self.sequence_b.len() {
            // Insertion.
            output.extend(Some(NodeWithOptionalPredecessorAndIsTarget {
                node: Node {
                    identifier: Identifier {
                        position_a: na,
                        position_b: nb + 1,
                    },
                    cost: Cost {
                        hops: node.cost.hops + 1,
                        cost: node.cost.cost + 1,
                    },
                },
                predecessor_index: Some(index),
                is_target: na == self.sequence_a.len() && nb + 1 == self.sequence_b.len(),
            }));
        }
    }

    fn is_target(&self, node: &Self::Node) -> bool {
        node.identifier.position_a == self.sequence_a.len()
            && node.identifier.position_b == self.sequence_b.len()
    }
}

impl MulticriteriaDijkstraNode for Node {
    type Identifier = Identifier;

    type Cost = Cost;

    fn identifier(&self) -> &Self::Identifier {
        &self.identifier
    }

    fn cost(&self) -> &Self::Cost {
        &self.cost
    }
}

impl MulticriteriaDijkstraIdentifier for Identifier {}

impl MulticriteriaDijkstraCost for Cost {
    fn dominates(&self, other: &Self) -> bool {
        self.hops <= other.hops && self.cost <= other.cost
    }
}

fn count_path_operations(path: &[&Node]) -> (usize, usize, usize) {
    let mut masub = 0;
    let mut insertion = 0;
    let mut deletion = 0;
    for window in path.windows(2) {
        let from = &window[0].identifier;
        let to = &window[1].identifier;
        if from.position_a + 1 == to.position_a && from.position_b + 1 == to.position_b {
            // Match or mismatch.
            masub += 1;
        } else if from.position_a + 1 == to.position_a && from.position_b == to.position_b {
            deletion += 1;
        } else if from.position_a == to.position_a && from.position_b + 1 == to.position_b {
            // Insertion.
            insertion += 1;
        } else {
            panic!("Invalid path");
        }
    }
    (masub, insertion, deletion)
}

#[test]
fn minimum_hop_alignment_1() {
    let context = Context {
        sequence_a: b"G".to_vec(),
        sequence_b: b"T".to_vec(),
    };
    let mut dijkstra = MulticriteriaDijkstra::<PruneList, _>::new(context);
    dijkstra.search();

    let target_identifier = Identifier {
        position_a: 1,
        position_b: 1,
    };

    let mut expected_costs = [Cost { hops: 1, cost: 3 }, Cost { hops: 2, cost: 2 }];
    expected_costs.sort_unstable();
    let mut actual_costs = dijkstra
        .iter_costs(&target_identifier)
        .cloned()
        .collect::<Vec<_>>();
    actual_costs.sort_unstable();
    assert_eq!(actual_costs, expected_costs);

    let mut expected_paths = [(1, 0, 0), (0, 1, 1)];
    expected_paths.sort_unstable();
    let mut actual_paths: Vec<_> = dijkstra
        .backtrack(&target_identifier)
        .unwrap()
        .into_iter()
        .map(|path| count_path_operations(&path))
        .collect();
    actual_paths.sort_unstable();
    assert_eq!(actual_paths, expected_paths);
}

#[test]
fn minimum_hop_alignment_4() {
    let context = Context {
        sequence_a: b"GGGG".to_vec(),
        sequence_b: b"TTTT".to_vec(),
    };
    let mut dijkstra = MulticriteriaDijkstra::<PruneList, _>::new(context);
    dijkstra.search();

    let target_identifier = Identifier {
        position_a: 4,
        position_b: 4,
    };

    let mut expected_costs = [
        Cost { hops: 4, cost: 12 },
        Cost { hops: 5, cost: 11 },
        Cost { hops: 6, cost: 10 },
        Cost { hops: 7, cost: 9 },
        Cost { hops: 8, cost: 8 },
    ];
    expected_costs.sort_unstable();
    let mut actual_costs = dijkstra
        .iter_costs(&target_identifier)
        .cloned()
        .collect::<Vec<_>>();
    actual_costs.sort_unstable();
    assert_eq!(actual_costs, expected_costs);

    let mut expected_paths = [(4, 0, 0), (3, 1, 1), (2, 2, 2), (1, 3, 3), (0, 4, 4)];
    expected_paths.sort_unstable();
    let mut actual_paths: Vec<_> = dijkstra
        .backtrack(&target_identifier)
        .unwrap()
        .into_iter()
        .map(|path| count_path_operations(&path))
        .collect();
    actual_paths.sort_unstable();
    assert_eq!(actual_paths, expected_paths);
}
