use crate::{
    error::Error,
    node::{MulticriteriaDijkstraCost, MulticriteriaDijkstraNode},
    prune_list::{
        IndexedNode, MulticriteriaDijkstraPruneList, NodeWithIsTarget, NodeWithOptionalPredecessor,
        NodeWithOptionalPredecessorAndIsTarget,
    },
};

pub mod error;
pub mod node;
pub mod prune_list;
#[cfg(test)]
mod tests;

pub struct MulticriteriaDijkstra<PruneList, Context> {
    datastructures: MulticriteriaDijkstraDatastructures<PruneList>,
    context: Context,
}

pub struct MulticriteriaDijkstraDatastructures<PruneList> {
    prune_list: PruneList,
}

pub trait MulticriteriaDijkstraContext {
    type Node: MulticriteriaDijkstraNode;

    fn generate_root_nodes(&self, output: &mut impl Extend<NodeWithIsTarget<Self::Node>>);

    fn generate_successors<Index: Copy>(
        &self,
        node: IndexedNode<Self::Node, Index>,
        output: &mut impl Extend<NodeWithOptionalPredecessorAndIsTarget<Self::Node, Index>>,
    );

    fn is_target(&self, node: &Self::Node) -> bool;
}

impl<
    Node: MulticriteriaDijkstraNode,
    PruneList: MulticriteriaDijkstraPruneList<Node = Node>,
    Context: MulticriteriaDijkstraContext<Node = Node>,
> MulticriteriaDijkstra<PruneList, Context>
{
    pub fn new(context: Context) -> Self {
        Self {
            datastructures: MulticriteriaDijkstraDatastructures::default(),
            context,
        }
    }

    pub fn from_datastructures(
        datastructures: MulticriteriaDijkstraDatastructures<PruneList>,
        context: Context,
    ) -> Self {
        Self {
            datastructures,
            context,
        }
    }

    pub fn into_datastructures(self) -> MulticriteriaDijkstraDatastructures<PruneList> {
        self.datastructures
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn reset(&mut self, context_resetter: impl FnOnce(&mut Context)) {
        self.datastructures.reset();
        context_resetter(&mut self.context);
    }

    pub fn prune_list(&self) -> &PruneList {
        &self.datastructures.prune_list
    }

    pub fn prune_list_mut(&mut self) -> &mut PruneList {
        &mut self.datastructures.prune_list
    }

    /// Search until there are no open nodes left.
    pub fn search(&mut self) {
        self.search_with_prune_condition(|_, _| false);
    }

    /// Search until there are no open nodes left, pruning nodes that fulfil a condition.
    ///
    /// `prune_condition` is called for each generated node, including root nodes, before it is inserted into the prune list.
    /// If it returns true, then the node is discarded.
    /// Hence, it is never expanded and does not appear in the results of the search.
    pub fn search_with_prune_condition(
        &mut self,
        mut prune_condition: impl FnMut(&Context, &Node) -> bool,
    ) {
        let mut prune_list = ConditionallyPruningPruneList {
            prune_list: &mut self.datastructures.prune_list,
            context: &self.context,
            prune_condition: &mut prune_condition,
        };

        // Initialise with root nodes.
        self.context.generate_root_nodes(&mut prune_list);

        while let Some(indexed_node) = prune_list.prune_list.next() {
            // Generate successors of the node.
            self.context
                .generate_successors(indexed_node.cloned(), &mut prune_list);
        }
    }

    /// Search until there are no open nodes left, pruning nodes that exceed the cost limit.
    ///
    /// A node exceeds the cost limit if its cost does not dominate `cost_limit`.
    /// For example, with two criteria and a cost limit of `(5, 5)`, a node with cost `(5, 5)` is kept,
    /// but nodes with costs `(6, 5)` or `(4, 6)` are pruned.
    ///
    /// Note that the limit applies to all nodes, not only to targets.
    /// If the costs can decrease along an edge, then nodes within the limit that are only reachable
    /// via nodes exceeding the limit are not found.
    pub fn search_until_cost_limit(
        &mut self,
        cost_limit: &<Context::Node as MulticriteriaDijkstraNode>::Cost,
    ) {
        self.search_with_prune_condition(|_, node| !node.cost().dominates(cost_limit));
    }

    pub fn iter_costs<'this>(
        &'this self,
        identifier: &<Context::Node as MulticriteriaDijkstraNode>::Identifier,
    ) -> impl Iterator<Item = &'this <Context::Node as MulticriteriaDijkstraNode>::Cost>
    where
        Node: 'this,
    {
        self.datastructures
            .prune_list
            .iter_costs_at_identifier(identifier)
    }

    pub fn backtrack<'this>(
        &'this self,
        identifier: &<Context::Node as MulticriteriaDijkstraNode>::Identifier,
    ) -> Result<Vec<Vec<&'this Context::Node>>, Error> {
        if !self.datastructures.prune_list.supports_backtracking() {
            return Err(Error::BacktrackingNotSupported);
        }

        let mut paths = Vec::new();

        for NodeWithOptionalPredecessor {
            node,
            predecessor_index: mut optional_predecessor_index,
        } in self
            .datastructures
            .prune_list
            .iter_nodes_with_predecessor_at_identifier(identifier)
        {
            let mut path = Vec::new();

            path.push(node);
            while let Some(predecessor_index) = optional_predecessor_index {
                let NodeWithOptionalPredecessor {
                    node: predecessor_node,
                    predecessor_index: new_optional_predecessor_index,
                } = self
                    .datastructures
                    .prune_list
                    .get(predecessor_index)
                    .unwrap();

                path.push(predecessor_node);
                optional_predecessor_index = new_optional_predecessor_index;
            }

            path.reverse();
            paths.push(path);
        }

        Ok(paths)
    }
}

impl<PruneList: MulticriteriaDijkstraPruneList> MulticriteriaDijkstraDatastructures<PruneList> {
    pub fn reset(&mut self) {
        self.prune_list.clear();
    }

    pub fn prune_list(&self) -> &PruneList {
        &self.prune_list
    }

    pub fn prune_list_mut(&mut self) -> &mut PruneList {
        &mut self.prune_list
    }
}

impl<PruneList: MulticriteriaDijkstraPruneList> Default
    for MulticriteriaDijkstraDatastructures<PruneList>
{
    fn default() -> Self {
        Self {
            prune_list: PruneList::default(),
        }
    }
}

/// Wraps a prune list and discards nodes that fulfil the prune condition before inserting them.
struct ConditionallyPruningPruneList<
    'prune_list,
    'context,
    'prune_condition,
    PruneList,
    Context,
    PruneCondition,
> {
    prune_list: &'prune_list mut PruneList,
    context: &'context Context,
    prune_condition: &'prune_condition mut PruneCondition,
}

impl<
    PruneList: MulticriteriaDijkstraPruneList,
    Context,
    PruneCondition: FnMut(&Context, &PruneList::Node) -> bool,
> Extend<NodeWithOptionalPredecessorAndIsTarget<PruneList::Node, PruneList::Index>>
    for ConditionallyPruningPruneList<'_, '_, '_, PruneList, Context, PruneCondition>
{
    fn extend<
        T: IntoIterator<
            Item = NodeWithOptionalPredecessorAndIsTarget<PruneList::Node, PruneList::Index>,
        >,
    >(
        &mut self,
        iter: T,
    ) {
        for node in iter {
            if !(self.prune_condition)(self.context, &node.node) {
                self.prune_list.insert(node);
            }
        }
    }
}

impl<
    PruneList: MulticriteriaDijkstraPruneList,
    Context,
    PruneCondition: FnMut(&Context, &PruneList::Node) -> bool,
> Extend<NodeWithIsTarget<PruneList::Node>>
    for ConditionallyPruningPruneList<'_, '_, '_, PruneList, Context, PruneCondition>
{
    fn extend<T: IntoIterator<Item = NodeWithIsTarget<PruneList::Node>>>(&mut self, iter: T) {
        for NodeWithIsTarget { node, is_target } in iter {
            if !(self.prune_condition)(self.context, &node) {
                self.prune_list
                    .insert(NodeWithOptionalPredecessorAndIsTarget {
                        node,
                        predecessor_index: None,
                        is_target,
                    });
            }
        }
    }
}
