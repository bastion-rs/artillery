use super::chain_node::ChainNode;
use super::errors::*;
use std::fmt;
use std::fmt::Display;

///
/// Representation of a closed-loop CRAQ chain
#[derive(Default, Debug)]
pub struct CraqChain {
    /// List of nodes in this chain, in order.
    nodes: Vec<ChainNode>,
    /// Index of this node.
    node_idx: usize,
}

impl CraqChain {
    ///
    /// Create a new chain.
    pub fn new(nodes: &[ChainNode], node_idx: usize) -> Result<Self> {
        ensure!(
            node_idx < nodes.len(),
            "Node index can't be greater than chain length."
        );

        Ok(Self {
            nodes: nodes.to_vec(),
            node_idx,
        })
    }

    ///
    /// Returns whether this node is the head of its chain.
    pub fn is_head(&self) -> bool {
        self.node_idx == 0
    }

    ///
    /// Returns the successor node if exists
    pub fn is_tail(&self) -> bool {
        self.node_idx == self.nodes.len().saturating_sub(1)
    }

    ///
    /// Returns the successor node if exists
    pub fn get_successor(&self) -> Option<&ChainNode> {
        if self.is_tail() {
            None
        } else {
            self.nodes.get(self.node_idx.saturating_add(1))
        }
    }

    ///
    /// Returns the tail node.
    pub fn get_tail(&self) -> Option<&ChainNode> {
        self.nodes.last()
    }

    ///
    /// Returns the chain node associated with the current node index.
    pub fn get_node(&self) -> Option<&ChainNode> {
        self.nodes.get(self.node_idx)
    }

    ///
    /// Returns the current node index.
    pub fn get_index(&self) -> usize {
        self.node_idx
    }

    ///
    /// Returns the size of this chain.
    pub fn chain_size(&self) -> usize {
        self.nodes.len()
    }
}

///
/// Human-readable display impl for the Chain
impl Display for CraqChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CR: Index [{}] in chain: {:#?}",
            self.node_idx, self.nodes
        )
    }
}
