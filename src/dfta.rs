//! Deterministic finite tree automata.

use crate::ast_node::AstNode;
use egg::{Analysis, EGraph, Id, Language};
use std::{
    collections::{BTreeMap, BTreeSet},
    iter::FromIterator,
};

/// A deterministic finite tree automaton (DFTA) is a set of transition rules of
/// the form `op(s1, ..., sn) -> s0` where `op` is an operation of type `Op` and
/// `s0, ..., sn` are states of type `S`.
#[derive(Debug)]
pub(crate) struct Dfta<Op, S> {
    by_operation: BTreeMap<Op, BTreeSet<(Vec<S>, S)>>,
    by_output: BTreeMap<S, BTreeSet<(Op, Vec<S>)>>,
}

impl<Op, S> Dfta<Op, S>
where
    Op: Ord,
    S: Ord,
{
    /// Create an empty DFTA.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            by_operation: BTreeMap::new(),
            by_output: BTreeMap::new(),
        }
    }
}

impl<Op, S> Dfta<Op, S>
where
    Op: Clone + Ord,
    S: Clone + Ord,
{
    /// Adds a new transition rule to the DFTA.
    pub(crate) fn add_rule<I>(&mut self, operation: Op, inputs: I, output: S)
    where
        I: IntoIterator<Item = S>,
    {
        let inputs = Vec::from_iter(inputs);
        self.by_operation
            .entry(operation.clone())
            .or_default()
            .insert((inputs.clone(), output.clone()));
        self.by_output
            .entry(output)
            .or_default()
            .insert((operation, inputs));
    }

    /// Intersect the DFTA with itself to produce a new DFTA over pairs of
    /// states. Transitions to the off-diagonal states in the new DFTA (that is,
    /// states `(s1, s2)` where `s1 != s2`) represent potential antiunifications
    /// of the enodes in the equivalent egraph of this DFTA.
    #[must_use]
    pub(crate) fn cross_over(&self) -> Dfta<Op, (S, S)> {
        let mut new_dfta = Dfta::new();
        for (op, rules) in &self.by_operation {
            for rule1 in rules {
                for (inputs2, output2) in rules.range(rule1..) {
                    let (inputs1, output1) = rule1;
                    let new_inputs = inputs1.iter().cloned().zip(inputs2.iter().cloned());
                    let new_output = (output1.clone(), output2.clone());
                    new_dfta.add_rule(op.clone(), new_inputs, new_output);
                }
            }
        }
        new_dfta
    }
}

impl<Op, S> Dfta<Op, S>
where
    S: Ord,
{
    /// Returns an iterator over the states in the DFTA which are the output of
    /// some transition rule.
    pub(crate) fn output_states(&self) -> impl Iterator<Item = &S> {
        self.by_output.keys()
    }

    /// Get all the transition rules which have this state as an output. If
    /// there are no transition rules to this state, this is guaranteed to
    /// return `None`.
    #[must_use]
    pub(crate) fn get_by_output(&self, output: &S) -> Option<&BTreeSet<(Op, Vec<S>)>> {
        self.by_output.get(output)
    }
}

impl<Op, S> Default for Dfta<Op, S>
where
    Op: Ord,
    S: Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Op, A> From<&EGraph<AstNode<Op>, A>> for Dfta<Op, Id>
where
    Op: Clone + Ord,
    A: Analysis<AstNode<Op>>,
    AstNode<Op>: Language,
{
    /// Converts an egraph into its equivalent DFTA. Each enode `(op e1 ... en)`
    /// of each eclass `e` in the egraph is represented by a transition rule
    /// `op(e1, ..., en) -> e`.
    fn from(egraph: &EGraph<AstNode<Op>, A>) -> Self {
        let mut dfta = Dfta::new();
        for eclass in egraph.classes() {
            for enode in eclass.iter() {
                dfta.add_rule(
                    enode.operation().clone(),
                    enode.iter().map(|&id| egraph.find(id)),
                    egraph.find(eclass.id),
                );
            }
        }
        dfta
    }
}
