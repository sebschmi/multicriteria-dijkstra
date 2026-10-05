use crate::node::MulticriteriaDijkstraNode;

pub trait MulticriteriaDijkstraPruneList:
    Default
    + Extend<NodeWithOptionalPredecessorAndIsTarget<Self::Node, Self::Index>>
    + Extend<NodeWithIsTarget<Self::Node>>
{
    type Node: MulticriteriaDijkstraNode;

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
        node: NodeWithOptionalPredecessorAndIsTarget<Self::Node, Self::Index>,
    ) -> Option<Self::Index>;

    /// Returns and closes an arbitrary open node, or `None` if there are no open nodes.
    fn next(&mut self) -> Option<IndexedNode<&Self::Node, Self::Index>>;

    /// Returns the node with the given index and its predecessor, or `None` if it has been discarded.
    fn get(
        &self,
        index: Self::Index,
    ) -> Option<NodeWithOptionalPredecessor<&Self::Node, Self::Index>>;

    /// Iterate over the nodes with predecessor indices at the given identifier.
    fn iter_nodes_with_predecessor_at_identifier<'this, 'identifier>(
        &'this self,
        identifier: &'identifier <Self::Node as MulticriteriaDijkstraNode>::Identifier,
    ) -> impl use<'this, Self>
    + Iterator<Item = NodeWithOptionalPredecessor<&'this Self::Node, Self::Index>>
    where
        Self::Node: 'this;

    fn iter_costs_at_identifier<'this, 'identifier>(
        &'this self,
        identifier: &'identifier <Self::Node as MulticriteriaDijkstraNode>::Identifier,
    ) -> impl use<'this, Self> + Iterator<Item = &'this <Self::Node as MulticriteriaDijkstraNode>::Cost>
    where
        Self::Node: 'this,
    {
        self.iter_nodes_with_predecessor_at_identifier(identifier)
            .map(|node_with_predecessor| node_with_predecessor.node.cost())
    }

    /// Remove all nodes from the list.
    fn clear(&mut self);

    /// Returns true if this prune list supports backtracking.
    fn supports_backtracking(&self) -> bool;
}

pub trait MulticriteriaDijkstraPruneListIndex: Copy {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeWithOptionalPredecessorAndIsTarget<Node, Index> {
    pub node: Node,
    pub predecessor_index: Option<Index>,
    pub is_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeWithIsTarget<Node> {
    pub node: Node,
    pub is_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeWithOptionalPredecessor<Node, Index> {
    pub node: Node,
    pub predecessor_index: Option<Index>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl MulticriteriaDijkstraPruneListIndex for u8 {}
impl MulticriteriaDijkstraPruneListIndex for u16 {}
impl MulticriteriaDijkstraPruneListIndex for u32 {}
impl MulticriteriaDijkstraPruneListIndex for u64 {}
impl MulticriteriaDijkstraPruneListIndex for u128 {}
impl MulticriteriaDijkstraPruneListIndex for usize {}
