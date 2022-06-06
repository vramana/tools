use std::sync::Arc;

use rome_js_syntax::{JsAnyRoot, JsLanguage};
use rome_rowan::AstNode;

use crate::{registry::Rule, LanguageOfRule};

/// Specifies which services a language needs
/// to be able to run all possible lint rules.
///
/// Also instantiate all these services.
pub trait WithServiceBag {
    type Services;

    fn new(root: JsAnyRoot) -> Self::Services;
}

/// Easy access to which services a language needs to run all
/// lint rules.
type ServicesOf<Language> = <Language as WithServiceBag>::Services;

/// Carries all services a language needs to run all
/// lint rules
#[derive(Clone)]
pub struct RuleContextServiceBag<Language>
where
    Language: WithServiceBag,
{
    services: Arc<ServicesOf<Language>>,
}

impl<L> RuleContextServiceBag<L>
where
    L: WithServiceBag,
{
    pub fn new(root: JsAnyRoot) -> Self {
        Self {
            services: Arc::new(<L as WithServiceBag>::new(root)),
        }
    }
}

impl<Language> std::ops::Deref for RuleContextServiceBag<Language>
where
    Language: WithServiceBag,
{
    type Target = ServicesOf<Language>;

    fn deref(&self) -> &Self::Target {
        &self.services
    }
}

/// Gives lint Rules access to everything associated with
/// a lint analyze run, such as:
/// - Nodes and the parsed tree
/// - All services as specified for each language
pub(crate) struct RuleContext<TRule>
where
    TRule: Rule,
    LanguageOfRule<TRule>: WithServiceBag,
{
    query_result: <TRule as Rule>::Query,
    services: RuleContextServiceBag<LanguageOfRule<TRule>>,
}

impl<TRule> RuleContext<TRule>
where
    TRule: Rule,
    LanguageOfRule<TRule>: WithServiceBag,
{
    pub fn new(
        query_result: <TRule as Rule>::Query,
        services: RuleContextServiceBag<LanguageOfRule<TRule>>,
    ) -> Self {
        Self {
            query_result,
            services,
        }
    }

    pub fn query(&self) -> &<TRule as Rule>::Query {
        &self.query_result
    }
}

/// Specific methods for Javascript rules
pub trait JsRuleContext {
    fn root(&self) -> &JsAnyRoot;
}

impl<TRule> JsRuleContext for RuleContext<TRule>
where
    TRule: Rule,
    <TRule as Rule>::Query: AstNode<Language = JsLanguage>,
{
    fn root(&self) -> &JsAnyRoot {
        &self.services.0
    }
}

/// Specific services for Javascript Rules
impl WithServiceBag for JsLanguage {
    type Services = (JsAnyRoot,);

    fn new(root: JsAnyRoot) -> Self::Services {
        (root,)
    }
}
