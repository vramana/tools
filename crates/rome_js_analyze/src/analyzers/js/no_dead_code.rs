use std::{cmp::Ordering, collections::VecDeque, vec::IntoIter};

use roaring::bitmap::RoaringBitmap;
use rome_analyze::{context::RuleContext, declare_rule, Rule, RuleCategory, RuleDiagnostic};
use rome_console::markup;
use rome_control_flow::{builder::BlockId, ExceptionHandler, Instruction, InstructionKind};
use rome_js_syntax::{JsLanguage, JsReturnStatement, JsSyntaxElement, JsSyntaxKind, TextRange};
use rome_rowan::AstNode;
use rustc_hash::FxHashMap;

use crate::control_flow::ControlFlowGraph;

declare_rule! {
    /// Disallow unreachable code
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// function example() {
    ///     return;
    ///     neverCalled();
    /// }
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// function example() {
    ///     for(let i = 0; i < 10; ++i) {
    ///         break;
    ///     }
    /// }
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// function example() {
    ///     for(const key in value) {
    ///         continue;
    ///         neverCalled();
    ///     }
    /// }
    /// ```
    pub(crate) NoDeadCode = "noDeadCode"
}

impl Rule for NoDeadCode {
    const CATEGORY: RuleCategory = RuleCategory::Lint;

    type Query = ControlFlowGraph;
    type State = UnreachableRange;
    type Signals = UnreachableRanges;

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        let mut signals = UnreachableRanges::new();

        let cfg = ctx.query();

        if exceeds_complexity_threshold(cfg) {
            analyze_simple(cfg, &mut signals)
        } else {
            analyze_fine(cfg, &mut signals)
        }

        signals
    }

    fn diagnostic(_: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let mut diagnostic = RuleDiagnostic::warning(
            state.text_trimmed_range,
            markup! {
                "This code is unreachable"
            },
        )
        .unnecessary();

        /// Primary label of the diagnostic if it comes earlier in the source than its secondary labels
        const PRIMARY_LABEL_BEFORE: &str = "This code will never be reached ...";
        /// Primary label of the diagnostic if it comes later in the source than its secondary labels
        const PRIMARY_LABEL_AFTER: &str = "... before it can reach this code";

        // Pluralize and adapt the error message accordingly based on the
        // number and position of secondary labels
        match state.terminators.as_slice() {
            // The CFG didn't contain enough informations to determine a cause
            // for this range being unreachable
            [] => {}
            // A single node is responsible for this range being unreachable
            [node] => {
                if node.range.start() < state.text_trimmed_range.start() {
                    diagnostic = diagnostic
                        .secondary(
                            node.range,
                            format_args!("This statement will {} ...", node.reason()),
                        )
                        .primary(PRIMARY_LABEL_AFTER);
                } else {
                    diagnostic = diagnostic.primary(PRIMARY_LABEL_BEFORE).secondary(
                        node.range,
                        format_args!(
                            "... because this statement will {} beforehand",
                            node.reason()
                        ),
                    );
                }
            }
            // The range has two dominating terminator instructions
            [node_a, node_b] => {
                if node_a.kind == node_b.kind {
                    diagnostic = diagnostic
                        .secondary(node_a.range, "Either this statement ...")
                        .secondary(
                            node_b.range,
                            format_args!("... or this statement will {} ...", node_b.reason()),
                        )
                        .primary(PRIMARY_LABEL_AFTER);
                } else {
                    diagnostic = diagnostic
                        .secondary(
                            node_a.range,
                            format_args!("Either this statement will {} ...", node_a.reason()),
                        )
                        .secondary(
                            node_b.range,
                            format_args!("... or this statement will {} ...", node_b.reason()),
                        )
                        .primary(PRIMARY_LABEL_AFTER);
                }
            }
            // The range has three or more dominating terminator instructions
            terminators => {
                // SAFETY: This substraction is safe since the match expression
                // ensures the slice has at least 3 elements
                let last = terminators.len() - 1;

                // Do not repeat the reason for each terminator if they all have the same kind
                let (_, has_homogeneous_kind) = terminators
                    .iter()
                    .fold(None, |prev_kind, terminator| match prev_kind {
                        Some((kind, state)) => Some((kind, state && terminator.kind == kind)),
                        None => Some((terminator.kind, true)),
                    })
                    // SAFETY: terminators has at least 3 elements
                    .unwrap();

                if has_homogeneous_kind {
                    for (index, node) in terminators.iter().enumerate() {
                        if index == 0 {
                            diagnostic =
                                diagnostic.secondary(node.range, "Either this statement, ...");
                        } else if index < last {
                            diagnostic =
                                diagnostic.secondary(node.range, "... this statement, ...");
                        } else {
                            diagnostic = diagnostic.secondary(
                                node.range,
                                format_args!("... or this statement will {} ...", node.reason()),
                            );
                        }
                    }
                } else {
                    for (index, node) in terminators.iter().enumerate() {
                        if index == 0 {
                            diagnostic = diagnostic.secondary(
                                node.range,
                                format_args!("Either this statement will {}, ...", node.reason()),
                            );
                        } else if index < last {
                            diagnostic = diagnostic.secondary(
                                node.range,
                                format_args!("... this statement will {}, ...", node.reason()),
                            );
                        } else {
                            diagnostic = diagnostic.secondary(
                                node.range,
                                format_args!("... or this statement will {} ...", node.reason()),
                            );
                        }
                    }
                }

                diagnostic = diagnostic.primary(PRIMARY_LABEL_AFTER);
            }
        }

        Some(diagnostic)
    }
}

/// Any function with a complexity score higher than this value will use the
/// simple reachability analysis instead of the fine analysis
const COMPLEXITY_THRESHOLD: u32 = 20;

/// Returns true if the "complexity score" for the [ControlFlowGraph] is higher
/// than [COMPLEXITY_THRESHOLD]. This score is an arbritrary value (the formula
/// is similar to the cyclomatic complexity of the function but this is only
/// approximative) used to determine whether the NoDeadCode rule should perform
/// a fine reachability analysis or fall back to a simpler algorithm to avoid
/// spending too much time analyzing exceedingly complex functions
fn exceeds_complexity_threshold(cfg: &ControlFlowGraph) -> bool {
    let nodes = cfg.blocks.len() as u32;

    let mut edges: u32 = 0;
    let mut conditionals: u32 = 0;

    for block in &cfg.blocks {
        for inst in &block.instructions {
            if let InstructionKind::Jump { conditional, .. } = inst.kind {
                edges += 1;

                if conditional {
                    conditionals += 1;
                }

                let complexity = edges.saturating_sub(nodes) + conditionals / 2;
                if complexity > COMPLEXITY_THRESHOLD {
                    return true;
                }
            }
        }
    }

    false
}

/// Perform a simple reachability analysis, does not attempt to determine a
/// terminator instruction for unreachable ranges allowing blocks to be visited
/// at most once and ensuring the algorithm finishes in a bounded time
fn analyze_simple(cfg: &ControlFlowGraph, signals: &mut UnreachableRanges) {
    // Perform a simple reachability analysis on the control flow graph by
    // traversing the function starting at the entry point
    let mut reachable_blocks = RoaringBitmap::new();
    let mut queue = VecDeque::new();

    if !cfg.blocks.is_empty() {
        reachable_blocks.insert(0);
        queue.push_back((0, None));
    }

    while let Some((index, handlers)) = queue.pop_front() {
        let index = index as usize;
        let block = &cfg.blocks[index];

        // Lookup the existence of an exception edge for this block but
        // defer its creation until an instruction that can throw is encountered
        let mut exception_handlers = block.exception_handlers.split_first();

        // Tracks whether this block is "terminated", if an instruction
        // that unconditionally aborts the control flow of this block has
        // been encountered
        let mut has_terminator = false;

        for inst in &block.instructions {
            // If this block is terminated, mark this instruction as unreachable and continue
            if has_terminator {
                if let Some(node) = &inst.node {
                    signals.push(node, None);
                }
                continue;
            }

            // Do not create exception edges for instructions with no side effects
            if has_side_effects(inst) {
                // If this block has a pending exception edge, create an
                // additional path diverging towards the corresponding
                // catch or finally block
                if let Some((handler, handlers)) = exception_handlers.take() {
                    if reachable_blocks.insert(handler.target) {
                        queue.push_back((handler.target, Some(handlers)));
                    }
                }
            }

            match inst.kind {
                InstructionKind::Statement => {}
                InstructionKind::Jump {
                    conditional,
                    block,
                    finally_fallthrough,
                } => {
                    if finally_fallthrough && handlers.is_some() {
                        // Jump towards the corresponding block if there are pending exception
                        // handlers, otherwise return from the function
                        let handlers = handlers.and_then(<[_]>::split_first);

                        if let Some((handler, handlers)) = handlers {
                            if reachable_blocks.insert(handler.target) {
                                queue.push_back((handler.target, Some(handlers)));
                            }
                        }
                    } else if reachable_blocks.insert(block.index()) {
                        // Insert an edge if this jump is reachable
                        queue.push_back((block.index(), handlers));
                    }

                    // Jump is a terminator instruction if it's unconditional
                    if !conditional {
                        has_terminator = true;
                    }
                }
                InstructionKind::Return => {
                    if let Some((handler, handlers)) = block.cleanup_handlers.split_first() {
                        if reachable_blocks.insert(handler.target) {
                            queue.push_back((handler.target, Some(handlers)));
                        }
                    }

                    has_terminator = true;
                }
            }
        }
    }

    // Detect blocks that were never reached by the above traversal
    for (index, block) in cfg.blocks.iter().enumerate() {
        let index = index as u32;
        if reachable_blocks.contains(index) {
            continue;
        }

        for inst in &block.instructions {
            if let Some(node) = &inst.node {
                signals.push(node, None);
            }
        }
    }
}

/// Performs a fine reachability analysis of the control flow graph: this
/// algorithm traverse all the possible paths through the function to determine
/// the reachability of each block and instruction but also find one or more
/// "terminator instructions" for each unreachable range of code that cause it
/// to be impossible to reach
fn analyze_fine(cfg: &ControlFlowGraph, signals: &mut UnreachableRanges) {
    // Traverse the CFG and calculate block / instruction reachability
    let block_paths = traverse_cfg(cfg, signals);

    // Detect unreachable blocks using the result of the above traversal
    'blocks: for (index, block) in cfg.blocks.iter().enumerate() {
        let index = index as u32;
        match block_paths.get(&index) {
            // Block has incoming paths, but may be unreachable if they all
            // have a dominating terminator intruction
            Some(paths) => {
                let mut terminators = Vec::new();
                for path in paths {
                    if let Some(terminator) = *path {
                        terminators.push(terminator);
                    } else {
                        // This path has no terminator, the block is reachable
                        continue 'blocks;
                    }
                }

                // Mark each instruction in the block as unreachable with
                // the appropriate terminator labels
                for inst in &block.instructions {
                    if let Some(node) = &inst.node {
                        for terminator in &terminators {
                            signals.push(node, *terminator);
                        }
                    }
                }
            }
            // Block has no incoming paths, is completely cut off from the CFG
            // In theory this shouldn't happen as our CFG also stores
            // unreachable edges, if we get here there might be a bug in
            // the control flow analysis
            None => {
                for inst in &block.instructions {
                    if let Some(node) = &inst.node {
                        // There is no incoming control flow so we can't
                        // determine a terminator instruction for this
                        // unreachable range
                        signals.push(node, None);
                    }
                }
            }
        }
    }
}

/// Individual entry in the traversal queue, holding the state for a
/// single "linearly independent path" through the function as it gets
/// created during the control flow traversal
struct PathState<'cfg> {
    /// Index of the next block to visit
    next_block: u32,
    /// Set of all blocks already visited on this path
    visited: RoaringBitmap,
    /// Current terminating instruction for the path, if one was
    /// encountered
    terminator: Option<Option<PathTerminator>>,
    exception_handlers: Option<&'cfg [ExceptionHandler]>,
}

/// Perform a simple reachability analysis on the control flow graph by
/// traversing the function starting at the entry points
fn traverse_cfg(
    cfg: &ControlFlowGraph,
    signals: &mut UnreachableRanges,
) -> FxHashMap<u32, Vec<Option<Option<PathTerminator>>>> {
    let mut queue = VecDeque::new();

    queue.push_back(PathState {
        next_block: 0,
        visited: RoaringBitmap::new(),
        terminator: None,
        exception_handlers: None,
    });

    // This maps holds a list of "path state", the active terminator
    // intruction for each path that can reach the block
    let mut block_paths = FxHashMap::default();

    while let Some(mut path) = queue.pop_front() {
        // Add the block to the visited set for the path, and the current
        // state of the path to the global reachable blocks map
        path.visited.insert(path.next_block);

        block_paths
            .entry(path.next_block)
            .or_insert_with(Vec::new)
            .push(path.terminator);

        let index = path.next_block as usize;
        let block = &cfg.blocks[index];

        // Lookup the existence of an exception edge for this block but
        // defer its creation until an instruction that can throw is encountered
        let mut exception_handlers = block.exception_handlers.split_first();

        // Set to true if the `terminator` is found inside of this block
        let mut has_direct_terminator = false;

        for inst in &block.instructions {
            // Do not create exception edges for instructions with no side effects
            if has_side_effects(inst) {
                // If this block has a pending exception edge, create an
                // additional path diverging towards the corresponding
                // catch or finally block
                if let Some((handler, handlers)) = exception_handlers.take() {
                    if !path.visited.contains(handler.target) {
                        queue.push_back(PathState {
                            next_block: handler.target,
                            visited: path.visited.clone(),
                            terminator: path.terminator,
                            exception_handlers: Some(handlers),
                        });
                    }
                }
            }

            // If this block has already ended, immediately mark this instruction as unreachable
            if let Some(terminator) = path.terminator.filter(|_| has_direct_terminator) {
                if let Some(node) = &inst.node {
                    signals.push(node, terminator);
                }
            }

            match inst.kind {
                InstructionKind::Statement => {}
                InstructionKind::Jump {
                    conditional,
                    block,
                    finally_fallthrough,
                } => {
                    handle_jump(&mut queue, &path, block, finally_fallthrough);

                    // Jump is a terminator instruction if it's unconditional
                    if path.terminator.is_none() && !conditional {
                        path.terminator = Some(inst.node.as_ref().map(|node| PathTerminator {
                            kind: node.kind(),
                            range: node.text_trimmed_range(),
                        }));
                        has_direct_terminator = true;
                    }
                }
                InstructionKind::Return => {
                    handle_return(&mut queue, &path, &block.cleanup_handlers);

                    if path.terminator.is_none() {
                        path.terminator = Some(inst.node.as_ref().map(|node| PathTerminator {
                            kind: node.kind(),
                            range: node.text_trimmed_range(),
                        }));
                        has_direct_terminator = true;
                    }
                }
            }
        }
    }

    block_paths
}

/// Returns `true` if `inst` can potentially have side effects. Due to the
/// dynamic nature of JavaScript this is a conservative check, biased towards
/// returning false positives
fn has_side_effects(inst: &Instruction<JsLanguage>) -> bool {
    let element = match inst.node.as_ref() {
        Some(element) => element,
        None => return false,
    };

    match element.kind() {
        JsSyntaxKind::JS_RETURN_STATEMENT => {
            let node = JsReturnStatement::unwrap_cast(element.as_node().unwrap().clone());
            node.argument().is_some()
        }

        JsSyntaxKind::JS_BREAK_STATEMENT | JsSyntaxKind::JS_CONTINUE_STATEMENT => false,
        kind => element.as_node().is_some() && !kind.is_literal(),
    }
}

/// Create an additional visitor path from a jump instruction and push it to the queue
fn handle_jump<'cfg>(
    queue: &mut VecDeque<PathState<'cfg>>,
    path: &PathState<'cfg>,
    block: BlockId,
    finally_fallthrough: bool,
) {
    // If this jump is exiting a finally clause and and this path is visiting
    // an exception handlers chain
    if finally_fallthrough && path.exception_handlers.is_some() {
        // Jump towards the corresponding block if there are pending exception
        // handlers, otherwise return from the function
        let handlers = path.exception_handlers.and_then(<[_]>::split_first);

        if let Some((handler, handlers)) = handlers {
            if !path.visited.contains(handler.target) {
                queue.push_back(PathState {
                    next_block: handler.target,
                    visited: path.visited.clone(),
                    terminator: path.terminator,
                    exception_handlers: Some(handlers),
                });
            }
        }
    } else if !path.visited.contains(block.index()) {
        // Push the jump target block to the queue if it hasn't
        // been visited yet in this path
        queue.push_back(PathState {
            next_block: block.index(),
            visited: path.visited.clone(),
            terminator: path.terminator,
            exception_handlers: path.exception_handlers,
        });
    }
}

/// Create an additional visitor path from a return instruction and push it to
/// the queue if necessary
fn handle_return<'cfg>(
    queue: &mut VecDeque<PathState<'cfg>>,
    path: &PathState<'cfg>,
    cleanup_handlers: &'cfg [ExceptionHandler],
) {
    if let Some((handler, handlers)) = cleanup_handlers.split_first() {
        if !path.visited.contains(handler.target) {
            queue.push_back(PathState {
                next_block: handler.target,
                visited: path.visited.clone(),
                terminator: path.terminator,
                exception_handlers: Some(handlers),
            });
        }
    }
}

/// Stores a list of unreachable code ranges, sorted in ascending source order
#[derive(Debug)]
pub(crate) struct UnreachableRanges {
    ranges: Vec<UnreachableRange>,
}

impl UnreachableRanges {
    fn new() -> Self {
        UnreachableRanges { ranges: Vec::new() }
    }

    fn push(&mut self, node: &JsSyntaxElement, terminator: Option<PathTerminator>) {
        let text_range = node.text_range();
        let text_trimmed_range = node.text_trimmed_range();

        // Perform a binary search on the ranges already in storage to find an
        // appropriate position for either merging or inserting the incoming range
        let insertion = self.ranges.binary_search_by(|entry| {
            if entry.text_range.end() < text_range.start() {
                Ordering::Less
            } else if text_range.end() < entry.text_range.start() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        match insertion {
            // The search returned an existing overlapping range, extend it to
            // cover the incoming range
            Ok(index) => {
                let entry = &mut self.ranges[index];
                entry.text_range = entry.text_range.cover(text_range);
                entry.text_trimmed_range = entry.text_trimmed_range.cover(text_trimmed_range);

                if let Some(terminator) = terminator {
                    // Terminator labels are also stored in ascending order to
                    // faciliate the generation of labels when the diagnostic
                    // gets emitted
                    let terminator_insertion = entry
                        .terminators
                        .binary_search_by_key(&terminator.range.start(), |node| node.range.start());

                    if let Err(index) = terminator_insertion {
                        entry.terminators.insert(index, terminator);
                    }
                }
            }
            // No overlapping range was found, insert at the appropriate
            // position to preserve the ordering instead
            Err(index) => {
                self.ranges.insert(
                    index,
                    UnreachableRange {
                        text_range,
                        text_trimmed_range,
                        terminators: terminator.into_iter().collect(),
                    },
                );
            }
        }
    }
}

impl IntoIterator for UnreachableRanges {
    type Item = UnreachableRange;
    type IntoIter = IntoIter<UnreachableRange>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_iter()
    }
}

/// Stores the trimmed and un-trimmed ranges for a block of unreachable source
/// code, along with a list of secondary labels pointing to the dominating
/// terminator instructions that cause it to be unreachable
#[derive(Debug)]
pub(crate) struct UnreachableRange {
    text_range: TextRange,
    text_trimmed_range: TextRange,
    terminators: Vec<PathTerminator>,
}

#[derive(Debug, Clone, Copy)]
struct PathTerminator {
    kind: JsSyntaxKind,
    range: TextRange,
}

impl PathTerminator {
    /// Returns a message explaining why this paths is unreachable
    fn reason(&self) -> &'static str {
        match self.kind {
            JsSyntaxKind::JS_BREAK_STATEMENT => "break the flow of the code",
            JsSyntaxKind::JS_CONTINUE_STATEMENT => "continue the loop",
            JsSyntaxKind::JS_RETURN_STATEMENT => "return from the function",
            JsSyntaxKind::JS_THROW_STATEMENT => "throw an exception",
            _ => "stop the flow of the code",
        }
    }
}
