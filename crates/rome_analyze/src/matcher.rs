use std::{cmp::Ordering, collections::BinaryHeap};

use rome_diagnostics::file::FileId;
use rome_rowan::{Language, TextRange};

use crate::{AnalyzerSignal, Phases, QueryMatch, Rule, RuleFilter, RuleGroup, ServiceBag};

/// The [QueryMatcher] trait is responsible of running lint rules on
/// [QueryMatch] instances emitted by the various [Visitor](crate::Visitor)
/// and push signals wrapped in [SignalEntry] to the signal queue
pub trait QueryMatcher<L: Language> {
    /// Return a unique identifier for a rule group if it's known by this query matcher
    fn find_group(&self, group: &str) -> Option<GroupKey>;
    /// Return a unique identifier for a rule if it's known by this query matcher
    fn find_rule(&self, group: &str, rule: &str) -> Option<RuleKey>;

    /// Execute a single query match
    fn match_query(&mut self, params: MatchQueryParams<L>);
}

/// Parameters provided to [QueryMatcher::match_query] and require to run lint rules
pub struct MatchQueryParams<'a, L: Language> {
    pub phase: Phases,
    pub file_id: FileId,
    pub root: &'a L::Root,
    pub query: QueryMatch<L>,
    pub services: &'a ServiceBag,
    pub signal_queue: &'a mut BinaryHeap<SignalEntry<L>>,
}

/// Opaque identifier for a group of rule
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GroupKey {
    group: &'static str,
}

impl GroupKey {
    pub(crate) fn new(group: &'static str) -> Self {
        Self { group }
    }

    pub fn group<G: RuleGroup>() -> Self {
        Self::new(G::NAME)
    }
}

impl From<GroupKey> for RuleFilter<'static> {
    fn from(key: GroupKey) -> Self {
        RuleFilter::Group(key.group)
    }
}

/// Opaque identifier for a single rule
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RuleKey {
    group: &'static str,
    rule: &'static str,
}

impl RuleKey {
    pub(crate) fn new(group: &'static str, rule: &'static str) -> Self {
        Self { group, rule }
    }

    pub fn rule<G: RuleGroup, R: Rule>() -> Self {
        Self::new(G::NAME, R::NAME)
    }
}

impl From<RuleKey> for RuleFilter<'static> {
    fn from(key: RuleKey) -> Self {
        RuleFilter::Rule(key.group, key.rule)
    }
}

impl PartialEq<RuleKey> for RuleFilter<'static> {
    fn eq(&self, other: &RuleKey) -> bool {
        match *self {
            RuleFilter::Group(group) => group == other.group,
            RuleFilter::Rule(group, rule) => group == other.group && rule == other.rule,
        }
    }
}

/// Entry for a pending signal in the `signal_queue`
pub struct SignalEntry<L: Language> {
    /// Boxed analyzer signal to be emitted
    pub signal: Box<dyn AnalyzerSignal<L>>,
    /// Unique identifier for the rule that emitted this signal
    pub rule: RuleKey,
    /// Text range in the document this signal covers
    pub text_range: TextRange,
}

// SignalEntry is ordered based on the starting point of its `text_range`
impl<L: Language> Ord for SignalEntry<L> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.text_range.start().cmp(&self.text_range.start())
    }
}

impl<L: Language> PartialOrd for SignalEntry<L> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<L: Language> Eq for SignalEntry<L> {}

impl<L: Language> PartialEq for SignalEntry<L> {
    fn eq(&self, other: &Self) -> bool {
        self.text_range.start() == other.text_range.start()
    }
}

#[cfg(test)]
mod tests {
    use rome_diagnostics::Diagnostic;
    use rome_rowan::{
        raw_language::{RawLanguage, RawLanguageKind, RawLanguageRoot, RawSyntaxTreeBuilder},
        AstNode, TextRange, TextSize, TriviaPiece, TriviaPieceKind,
    };

    use crate::{
        signals::DiagnosticSignal, Analyzer, AnalyzerContext, AnalyzerSignal, ControlFlow, Never,
        Phases, QueryMatch, QueryMatcher, RuleKey, ServiceBag, SignalEntry, SyntaxVisitor,
    };

    use super::{GroupKey, MatchQueryParams};

    struct SuppressionMatcher;

    impl QueryMatcher<RawLanguage> for SuppressionMatcher {
        // Recognize group name "group" and rule name "rule"
        fn find_group(&self, group: &str) -> Option<GroupKey> {
            if group == "group" {
                Some(GroupKey::new("group"))
            } else {
                None
            }
        }

        fn find_rule(&self, group: &str, rule: &str) -> Option<RuleKey> {
            match group {
                "group" => match rule {
                    "rule" => Some(RuleKey::new("group", "rule")),
                    _ => None,
                },
                _ => None,
            }
        }

        /// Emits a warning diagnostic for all literal expressions
        fn match_query(&mut self, params: MatchQueryParams<RawLanguage>) {
            let node = match params.query {
                QueryMatch::Syntax(node) => node,
                QueryMatch::ControlFlowGraph(..) => unreachable!(),
            };

            if node.kind() != RawLanguageKind::LITERAL_EXPRESSION {
                return;
            }

            let span = node.text_trimmed_range();
            params.signal_queue.push(SignalEntry {
                signal: Box::new(DiagnosticSignal::new(move || {
                    Diagnostic::warning(0, "test_suppression", "test_suppression").primary(span, "")
                })),
                rule: RuleKey::new("group", "rule"),
                text_range: span,
            });
        }
    }

    #[test]
    fn suppressions() {
        let root = {
            let mut builder = RawSyntaxTreeBuilder::new();

            builder.start_node(RawLanguageKind::ROOT);
            builder.start_node(RawLanguageKind::EXPRESSION_LIST);

            builder.start_node(RawLanguageKind::LITERAL_EXPRESSION);
            builder.token_with_trivia(
                RawLanguageKind::STRING_TOKEN,
                "//group\n\"warn_here\"",
                &[
                    TriviaPiece::new(TriviaPieceKind::SingleLineComment, 7),
                    TriviaPiece::new(TriviaPieceKind::Newline, 1),
                ],
                &[],
            );
            builder.finish_node();

            builder.token_with_trivia(
                RawLanguageKind::SEMICOLON_TOKEN,
                ";\n",
                &[],
                &[TriviaPiece::new(TriviaPieceKind::Newline, 1)],
            );

            builder.start_node(RawLanguageKind::LITERAL_EXPRESSION);
            builder.token_with_trivia(
                RawLanguageKind::STRING_TOKEN,
                "//group/rule\n\"warn_here\"",
                &[
                    TriviaPiece::new(TriviaPieceKind::SingleLineComment, 12),
                    TriviaPiece::new(TriviaPieceKind::Newline, 1),
                ],
                &[],
            );
            builder.finish_node();

            builder.token_with_trivia(
                RawLanguageKind::SEMICOLON_TOKEN,
                ";\n",
                &[],
                &[TriviaPiece::new(TriviaPieceKind::Newline, 1)],
            );

            builder.start_node(RawLanguageKind::LITERAL_EXPRESSION);
            builder.token_with_trivia(
                RawLanguageKind::STRING_TOKEN,
                "//unknown_group\n\"warn_here\"",
                &[
                    TriviaPiece::new(TriviaPieceKind::SingleLineComment, 15),
                    TriviaPiece::new(TriviaPieceKind::Newline, 1),
                ],
                &[],
            );
            builder.finish_node();

            builder.token_with_trivia(
                RawLanguageKind::SEMICOLON_TOKEN,
                ";\n",
                &[],
                &[TriviaPiece::new(TriviaPieceKind::Newline, 1)],
            );

            builder.start_node(RawLanguageKind::LITERAL_EXPRESSION);
            builder.token_with_trivia(
                RawLanguageKind::STRING_TOKEN,
                "//group/unknown_rule\n\"warn_here\"",
                &[
                    TriviaPiece::new(TriviaPieceKind::SingleLineComment, 20),
                    TriviaPiece::new(TriviaPieceKind::Newline, 1),
                ],
                &[],
            );
            builder.finish_node();

            builder.token_with_trivia(
                RawLanguageKind::SEMICOLON_TOKEN,
                ";\n",
                &[],
                &[TriviaPiece::new(TriviaPieceKind::Newline, 1)],
            );

            builder.finish_node();
            builder.finish_node();

            RawLanguageRoot::unwrap_cast(builder.finish())
        };

        let mut diagnostics = Vec::new();
        let mut emit_signal = |signal: &dyn AnalyzerSignal<RawLanguage>| -> ControlFlow<Never> {
            let diag = signal.diagnostic().expect("diagnostic");

            let code = diag.code.expect("code");
            let label = diag.primary.expect("primary label");

            diagnostics.push((code, label.span.range));
            ControlFlow::Continue(())
        };

        fn parse_suppression_comment(comment: &str) -> Vec<Option<&str>> {
            comment
                .trim_start_matches("//")
                .split(' ')
                .map(Some)
                .collect()
        }

        let mut analyzer = Analyzer::new(
            SuppressionMatcher,
            parse_suppression_comment,
            &mut emit_signal,
        );

        analyzer.add_visitor(SyntaxVisitor::default());

        let ctx: AnalyzerContext<RawLanguage> = AnalyzerContext {
            phase: Phases::Syntax,
            file_id: 0,
            root,
            range: None,
            services: ServiceBag::default(),
        };

        let result: Option<Never> = analyzer.run(ctx);
        assert!(result.is_none());

        assert_eq!(
            diagnostics.as_slice(),
            &[
                (
                    String::from("Linter"),
                    TextRange::new(TextSize::from(47), TextSize::from(62))
                ),
                (
                    String::from("test_suppression"),
                    TextRange::new(TextSize::from(63), TextSize::from(74))
                ),
                (
                    String::from("Linter"),
                    TextRange::new(TextSize::from(76), TextSize::from(96))
                ),
                (
                    String::from("test_suppression"),
                    TextRange::new(TextSize::from(97), TextSize::from(108))
                ),
            ]
        );
    }
}
