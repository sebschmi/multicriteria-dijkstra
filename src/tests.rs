use crate::{
    MulticriteriaDijkstraContext,
    node::{MulticriteriaDijkstraCost, MulticriteriaDijkstraIdentifier, MulticriteriaDijkstraNode},
    prune_list::{MulticriteriaDijkstraPruneList, NodeWithIsTarget, NodeWithOptionalPredecessor},
};

struct PruneList {}

struct Context {
    sequence_a: Vec<u8>,
    sequence_b: Vec<u8>,
}

#[derive(Clone)]
struct Node {
    identifier: Identifier,
    cost: Cost,
}

#[derive(Clone, PartialEq, Eq)]
struct Identifier {
    position_a: usize,
    position_b: usize,
}

#[derive(Clone)]
struct Cost {
    hops: usize,
    cost: usize,
}

impl MulticriteriaDijkstraPruneList<Node> for PruneList {
    type Index = usize;

    fn insert(
        &mut self,
        node: crate::prune_list::NodeWithPredecessorAndIsTarget<Node, Self::Index>,
        is_target: bool,
    ) -> Option<Self::Index> {
        todo!()
    }

    fn next(&mut self) -> Option<crate::prune_list::IndexedNode<&Node, Self::Index>> {
        todo!()
    }

    fn get(
        &self,
        index: Self::Index,
    ) -> Option<crate::prune_list::NodeWithOptionalPredecessor<&Node, Self::Index>> {
        todo!()
    }

    fn iter_nodes_with_predecessor_at_identifier<'this, 'identifier>(
        &'this self,
        identifier: &'identifier Identifier,
    ) -> impl use<'this, PruneList, Node>
    + Iterator<Item = NodeWithOptionalPredecessor<&'this Node, Self::Index>>
    where
        Node: 'this,
    {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn supports_backtracking(&self) -> bool {
        false
    }
}

impl MulticriteriaDijkstraContext for Context {
    type Node = Node;

    fn generate_root_nodes(
        &self,
        output: &mut impl Extend<crate::prune_list::NodeWithIsTarget<Self::Node>>,
    ) {
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
        node: crate::prune_list::IndexedNode<Self::Node, Index>,
        output: &mut impl Extend<crate::prune_list::NodeWithPredecessorAndIsTarget<Self::Node, Index>>,
    ) {
        todo!()
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

#[test]
fn minimum_hop_alignment() {
    todo!()
}
