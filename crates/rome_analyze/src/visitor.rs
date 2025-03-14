use std::collections::BinaryHeap;

use rome_diagnostics::file::FileId;
use rome_rowan::{AstNode, Language, SyntaxNode, TextRange, WalkEvent};

use crate::{
    matcher::MatchQueryParams,
    registry::{NodeLanguage, Phases},
    LanguageRoot, QueryMatch, QueryMatcher, ServiceBag, SignalEntry,
};

/// Mutable context objects shared by all visitors
pub struct VisitorContext<'a, L: Language> {
    pub phase: Phases,
    pub file_id: FileId,
    pub root: &'a LanguageRoot<L>,
    pub services: &'a ServiceBag,
    pub range: Option<TextRange>,
    pub(crate) query_matcher: &'a mut dyn QueryMatcher<L>,
    pub(crate) signal_queue: &'a mut BinaryHeap<SignalEntry<L>>,
}

impl<'a, L: Language> VisitorContext<'a, L> {
    pub fn match_query(&mut self, query: QueryMatch<L>) {
        self.query_matcher.match_query(MatchQueryParams {
            phase: self.phase,
            file_id: self.file_id,
            root: self.root,
            query,
            services: self.services,
            signal_queue: self.signal_queue,
        })
    }
}

/// Visitors are the main building blocks of the analyzer: they receive syntax
/// [WalkEvent]s, process these events to build secondary data structures from
/// the syntax tree, and emit rule query matches through the [crate::RuleRegistry]
pub trait Visitor {
    type Language: Language;

    fn visit(
        &mut self,
        event: &WalkEvent<SyntaxNode<Self::Language>>,
        ctx: VisitorContext<Self::Language>,
    );
}

/// A node visitor is a special kind of visitor that does not have a persistent
/// state for the entire run of the analyzer. Instead these visitors are
/// transient, they get instantiated when the traversal enters the
/// corresponding node type and destroyed when the corresponding node exits
///
/// Due to these specificities node visitors do not implement [Visitor]
/// directly, instead one or more of these must the merged into a single
/// visitor type using the [merge_node_visitors] macro
pub trait NodeVisitor<V>: Sized {
    type Node: AstNode;

    fn enter(
        node: Self::Node,
        ctx: &mut VisitorContext<NodeLanguage<Self::Node>>,
        stack: &mut V,
    ) -> Self;

    fn exit(
        self,
        node: Self::Node,
        ctx: &mut VisitorContext<NodeLanguage<Self::Node>>,
        stack: &mut V,
    );
}

/// Creates a single struct implementing [Visitor] over a collection of type
/// implementing the [NodeVisitor] helper trait. Unlike the global [Visitor],
/// node visitors are transient: they get instantiated when the traversal
/// enters the corresponding node and destroyed when the node is exited. They
/// are intended as a building blocks for creating and managing the state of
/// complex visitors by allowing the implementation to be split over multiple
/// smaller components.
///
/// # Example
///
/// ```ignore
/// struct BinaryVisitor;
///
/// impl NodeVisitor for BinaryVisitor {
///     type Node = BinaryExpression;
/// }
///
/// struct UnaryVisitor;
///
/// impl NodeVisitor for UnaryVisitor {
///     type Node = UnaryExpression;
/// }
///
/// merge_node_visitors! {
///     // This declares a new `ExpressionVisitor` struct that implements
///     // `Visitor` and manages instances of `BinaryVisitor` and
///     // `UnaryVisitor`
///     pub(crate) ExpressionVisitor {
///         binary: BinaryVisitor,
///         unary: UnaryVisitor,
///     }
/// }
/// ```
#[macro_export]
macro_rules! merge_node_visitors {
    ( $vis:vis $name:ident { $( $id:ident: $visitor:ty, )+ } ) => {
        $vis struct $name {
            stack: Vec<(::std::any::TypeId, usize)>,
            $( $vis $id: Vec<(usize, $visitor)>, )*
        }

        impl $name {
            $vis fn new() -> Self {
                Self {
                    stack: Vec::new(),
                    $( $id: Vec::new(), )*
                }
            }
        }

        impl $crate::Visitor for $name {
            type Language = <( $( <$visitor as $crate::NodeVisitor<$name>>::Node, )* ) as ::rome_rowan::macros::UnionLanguage>::Language;

            fn visit(
                &mut self,
                event: &::rome_rowan::WalkEvent<::rome_rowan::SyntaxNode<Self::Language>>,
                mut ctx: $crate::VisitorContext<Self::Language>,
            ) {
                match event {
                    ::rome_rowan::WalkEvent::Enter(node) => {
                        let kind = node.kind();

                        $(
                            if <<$visitor as $crate::NodeVisitor<$name>>::Node as ::rome_rowan::AstNode>::can_cast(kind) {
                                let node = <<$visitor as $crate::NodeVisitor<$name>>::Node as ::rome_rowan::AstNode>::unwrap_cast(node.clone());
                                let state = <$visitor as $crate::NodeVisitor<$name>>::enter(node, &mut ctx, self);

                                let stack_index = self.stack.len();
                                let ty_index = self.$id.len();

                                self.$id.push((stack_index, state));
                                self.stack.push((::std::any::TypeId::of::<$visitor>(), ty_index));
                                return;
                            }
                        )*
                    }
                    ::rome_rowan::WalkEvent::Leave(node) => {
                        let kind = node.kind();

                        $(
                            if <<$visitor as $crate::NodeVisitor<$name>>::Node as ::rome_rowan::AstNode>::can_cast(kind) {
                                self.stack.pop().unwrap();
                                let (_, state) = self.$id.pop().unwrap();

                                let node = <<$visitor as $crate::NodeVisitor<$name>>::Node as ::rome_rowan::AstNode>::unwrap_cast(node.clone());
                                <$visitor as $crate::NodeVisitor<$name>>::exit(state, node, &mut ctx, self);
                                return;
                            }
                        )*
                    }
                }
            }
        }
    };
}
