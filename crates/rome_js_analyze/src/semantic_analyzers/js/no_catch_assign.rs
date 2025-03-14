use crate::{semantic_services::Semantic, JsRuleAction};
use rome_analyze::{context::RuleContext, declare_rule, Rule, RuleCategory, RuleDiagnostic};
use rome_console::markup;
use rome_js_syntax::{JsCatchClause, JsIdentifierAssignment, JsSyntaxNode};
use rome_rowan::AstNode;

declare_rule! {
    /// Disallow reassigning exceptions in catch clauses
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// try {
    ///
    /// } catch (e) {
    ///   e;
    ///   e = 10;
    /// }
    /// ```
    ///
    /// ### Valid
    ///
    /// ```js
    /// try {
    ///
    /// } catch (e) {
    ///   let e = 10;
    ///   e = 100;
    /// }
    /// ```
    pub(crate) NoCatchAssign = "noCatchAssign"
}

impl Rule for NoCatchAssign {
    const CATEGORY: RuleCategory = RuleCategory::Lint;

    /// Why use [JsCatchClause] instead of [JsIdentifierAssignment] ? Because this could reduce search range.
    /// We only compare the declaration of [JsCatchClause] with all descent [JsIdentifierAssignment] of its body.
    type Query = Semantic<JsCatchClause>;
    /// The first element of `State` is the reassignment of catch parameter, the second element of `State` is the declaration of catch clause.
    type State = (JsIdentifierAssignment, JsSyntaxNode);
    type Signals = Vec<Self::State>;

    fn run(ctx: &RuleContext<Self>) -> Vec<Self::State> {
        let catch_clause = ctx.query();
        let model = ctx.model();

        catch_clause
            .declaration()
            .and_then(|decl| {
                // catch_binding
                // ## Example
                // try {

                // } catch (catch_binding) {
                //          ^^^^^^^^^^^^^
                // }
                let catch_binding = decl.binding().ok()?;
                let catch_binding_syntax = catch_binding.syntax();
                let body = catch_clause.body().ok()?;
                let mut invalid_assign = vec![];

                for assignment in body
                    .syntax()
                    .descendants()
                    .filter_map(JsIdentifierAssignment::cast)
                {
                    let decl_binding = model.declaration(&assignment).unwrap();
                    if decl_binding.syntax() == catch_binding_syntax {
                        invalid_assign.push((assignment, catch_binding_syntax.clone()));
                    }
                }
                Some(invalid_assign)
            })
            .unwrap_or_default()
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let (assignment, catch_binding_syntax) = state;
        let diagnostic = RuleDiagnostic::warning(
            assignment.syntax().text_trimmed_range(),
            markup! {
                " Do not "<Emphasis>"reassign catch parameters."</Emphasis>""
            },
        )
        .secondary(
            catch_binding_syntax.text_trimmed_range(),
            markup! {
                "The catch parameter is declared here"
            },
        );

        Some(diagnostic.footer_note("Use a local variable instead."))
    }

    fn action(_: &RuleContext<Self>, _: &Self::State) -> Option<JsRuleAction> {
        None
    }
}
