/// A node in the multicriteria Dijkstra algorithm.
///
/// The domination relation is specified by the domination relation of the cost.
/// It is reflexive and transitive, but not antisymmetric.
/// Specifically, two equal nodes must dominate each other.
pub trait MulticriteriaDijkstraNode: Clone {
    /// The identifier of the node.
    ///
    /// The identifier of the node specifies which graph node this node is at.
    /// Due to the nature of the multicriteria search, multiple Dijkstra nodes can exist for the same graph node.
    type Identifier: MulticriteriaDijkstraIdentifier;

    /// The cost of the node.
    ///
    /// This can have multiple dimensions.
    type Cost: MulticriteriaDijkstraCost;

    /// Returns the identifier of this node.
    fn identifier(&self) -> &Self::Identifier;

    /// Returns the cost of this node.
    fn cost(&self) -> &Self::Cost;

    /// Returns true if this node dominates the other node, based on the node costs.
    fn dominates(&self, other: &Self) -> bool {
        self.cost().dominates(other.cost())
    }
}

pub trait MulticriteriaDijkstraIdentifier: Eq {}

pub trait MulticriteriaDijkstraCost {
    /// Returns true if this cost dominates the other cost.
    /// It is reflexive and transitive, but not antisymmetric.
    /// Specifically, two equal nodes must dominate each other.
    fn dominates(&self, other: &Self) -> bool;
}
