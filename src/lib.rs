use crate::{
    error::Error,
    node::MulticriteriaDijkstraNode,
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

    pub fn search(&mut self) {
        // Initialise with root nodes.
        self.context
            .generate_root_nodes(&mut self.datastructures.prune_list);

        while let Some(indexed_node) = self.datastructures.prune_list.next() {
            // Generate successors of the node.
            self.context
                .generate_successors(indexed_node.cloned(), &mut self.datastructures.prune_list);
        }
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
