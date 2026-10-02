use crate::node::MulticriteriaDijkstraNode;

pub trait MulticriteriaDijkstraPruneList<Node: MulticriteriaDijkstraNode>:
    Default + Extend<NodeWithPredecessorAndIsTarget<Node, Self::Index>> + Extend<NodeWithIsTarget<Node>>
{
    /// A unique index for each node in the list.
    type Index: MulticriteriaDijkstraPruneListIndex;

    /// Insert a node into the list.
    ///
    /// The node is discarded if it is dominated by a node with the same identifier, or by a target node.
    /// If the node is not discarded and `is_target` is true, then all open nodes that are dominated by this node are discarded.
    ///
    /// # Returns
    ///
    /// `Some(Index)` if the node was inserted, `None` if it was discarded.
    /// The index is a unique identifier for the node in the list.
    fn insert(
        &mut self,
        node: NodeWithPredecessorAndIsTarget<Node, Self::Index>,
        is_target: bool,
    ) -> Option<Self::Index>;

    /// Returns an arbitrary open node, or `None` if there are no open nodes.
    fn next(&mut self) -> Option<IndexedNode<&Node, Self::Index>>;

    /// Returns the node with the given index and its predecessor, or `None` if it has been discarded.
    fn get(&self, index: Self::Index) -> Option<NodeWithOptionalPredecessor<&Node, Self::Index>>;

    /// Iterate over the nodes with predecessor indices at the given identifier.
    fn iter_nodes_with_predecessor_at_identifier<'this, 'identifier>(
        &'this self,
        identifier: &'identifier <Node as MulticriteriaDijkstraNode>::Identifier,
    ) -> impl use<'this, Self, Node>
    + Iterator<Item = NodeWithOptionalPredecessor<&'this Node, Self::Index>>
    where
        Node: 'this;

    /// Remove all nodes from the list.
    fn clear(&mut self);

    /// Returns true if this prune list supports backtracking.
    fn supports_backtracking(&self) -> bool;
}

pub trait MulticriteriaDijkstraPruneListIndex: Copy {}

pub struct NodeWithPredecessorAndIsTarget<Node, Index> {
    pub node: Node,
    pub predecessor_index: Index,
    pub is_target: bool,
}

pub struct NodeWithIsTarget<Node> {
    pub node: Node,
    pub is_target: bool,
}

pub struct NodeWithOptionalPredecessor<Node, Index> {
    pub node: Node,
    pub predecessor_index: Option<Index>,
}

pub struct IndexedNode<Node, Index> {
    pub node: Node,
    pub index: Index,
}

impl<Node: MulticriteriaDijkstraNode, Index: MulticriteriaDijkstraPruneListIndex>
    IndexedNode<&Node, Index>
{
    pub fn cloned(&self) -> IndexedNode<Node, Index> {
        IndexedNode {
            node: self.node.clone(),
            index: self.index,
        }
    }
}
