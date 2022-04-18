use rome_js_parser::{
    lossless_tree_sink::{LosslessTreeSink2, SyntaxNode2},
    Parse, SourceType,
};
use rome_js_syntax::{JsAnyRoot, JsSyntaxKind, NodeOrToken, SyntaxNode, SyntaxToken, WalkEvent};
use std::time::Instant;

trait Pass {
    fn process(&mut self);
}

fn syntax2_simd_pipeline_1(tree: &LosslessTreeSink2) {
    let a = Instant::now();
    let mut count = 0;

    for i in 0..tree.kinds.len() {
        if tree.kinds[i] == JsSyntaxKind::JS_FUNCTION_DECLARATION {
            count += 1;
        }
    }

    let took = Instant::now() - a;
    println!("   1: took: {:?} = {}", took, count);
}

fn syntax2_simd_pipeline_10(tree: &LosslessTreeSink2) {
    let a = Instant::now();
    let mut count = 0;

    for _ in 0..10 {
        for i in 0..tree.kinds.len() {
            if tree.kinds[i] == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                count += 1;
            }
        }
    }

    let took = Instant::now() - a;
    println!("  10: took: {:?} = {}", took, count);
}

fn syntax2_simd_pipeline_100(tree: &LosslessTreeSink2) {
    let a = Instant::now();
    let mut count = 0;

    for _ in 0..100 {
        for i in 0..tree.kinds.len() {
            if tree.kinds[i] == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                count += 1;
            }
        }
    }

    let took = Instant::now() - a;
    println!(" 100: took: {:?} = {}", took, count);
}

fn syntax2_pipeline_1(tree: &LosslessTreeSink2) {
    let a = Instant::now();
    let mut count = 0;
    for t in tree.all() {
        match t {
            rome_js_syntax::WalkEvent::Enter(SyntaxNode2 { pos }) => {
                let kind = tree.kinds[pos];
                if kind == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                    count += 1;
                }
            }
            rome_js_syntax::WalkEvent::Leave(_) => {}
        }
    }
    let took = Instant::now() - a;
    println!("   1: took: {:?} = {}", took, count);
}

fn syntax2_pipeline_10(tree: &LosslessTreeSink2) {
    let a = Instant::now();
    let mut count = 0;
    for _ in 0..10 {
        for t in tree.all() {
            match t {
                rome_js_syntax::WalkEvent::Enter(SyntaxNode2 { pos }) => {
                    let kind = tree.kinds[pos];
                    if kind == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                        count += 1;
                    }
                }
                rome_js_syntax::WalkEvent::Leave(_) => {}
            }
        }
    }
    let took = Instant::now() - a;
    println!("  10: took: {:?}", took);
}

fn syntax2_pipeline_100(tree: &LosslessTreeSink2) {
    let a = Instant::now();
    let mut count = 0;
    for _ in 0..100 {
        for t in tree.all() {
            match t {
                rome_js_syntax::WalkEvent::Enter(SyntaxNode2 { pos }) => {
                    let kind = tree.kinds[pos];
                    if kind == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                        count += 1;
                    }
                }
                rome_js_syntax::WalkEvent::Leave(_) => {}
            }
        }
    }
    let took = Instant::now() - a;
    println!(" 100: took: {:?} = {}", took, count);
}

fn syntax2_pipeline_100_cached(tree: &LosslessTreeSink2) {
    let a = Instant::now();
    let v: Vec<_> = tree.all().collect();
    let mut count = 0;
    for _ in 0..100 {
        for t in v.iter() {
            match t {
                rome_js_syntax::WalkEvent::Enter(SyntaxNode2 { pos }) => {
                    let kind = tree.kinds[*pos];
                    if kind == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                        count += 1;
                    }
                }
                rome_js_syntax::WalkEvent::Leave(_) => {}
            }
        }
    }
    let took = Instant::now() - a;
    println!("$100: took: {:?}", took);
}

fn syntax_pipeline_1(result: &Parse<JsAnyRoot>) {
    let a = Instant::now();
    let mut count = 0;
    for _ in 0..1 {
        let root = result.syntax();
        for t in root.descendants_with_tokens() {
            match t {
                rome_js_syntax::NodeOrToken::Node(node) => {
                    if node.kind() == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                        count += 1;
                    }
                }
                rome_js_syntax::NodeOrToken::Token(_) => {}
            }
        }
    }
    let took = Instant::now() - a;
    println!("   1: took: {:?} = {}", took, count);
}

fn syntax_pipeline_10(result: &Parse<JsAnyRoot>) {
    let a = Instant::now();
    let mut count = 0;
    for _ in 0..10 {
        let root = result.syntax();
        for t in root.descendants_with_tokens() {
            match t {
                rome_js_syntax::NodeOrToken::Node(node) => {
                    if node.kind() == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                        count += 1;
                    }
                }
                rome_js_syntax::NodeOrToken::Token(_) => {}
            }
        }
    }
    let took = Instant::now() - a;
    println!("  10: took: {:?}", took);
}

fn syntax_pipeline_100(result: &Parse<JsAnyRoot>) {
    let a = Instant::now();
    let mut count = 0;
    for _ in 0..100 {
        let root = result.syntax();
        for t in root.descendants_with_tokens() {
            match t {
                rome_js_syntax::NodeOrToken::Node(node) => {
                    if node.kind() == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                        count += 1;
                    }
                }
                rome_js_syntax::NodeOrToken::Token(_) => {}
            }
        }
    }
    let took = Instant::now() - a;
    println!(" 100: took: {:?} = {}", took, count);
}

fn syntax_pipeline_100_cached(result: &Parse<JsAnyRoot>) {
    let a = Instant::now();

    let root = result.syntax();

    let all: Vec<_> = root.descendants_with_tokens().collect();
    let mut count = 0;
    for _ in 0..100 {
        for t in all.iter() {
            match t {
                rome_js_syntax::NodeOrToken::Node(node) => {
                    if node.kind() == JsSyntaxKind::JS_FUNCTION_DECLARATION {
                        count += 1;
                    }
                }
                rome_js_syntax::NodeOrToken::Token(_) => {}
            }
        }
    }

    let took = Instant::now() - a;
    println!("$100: took: {:?}", took);
}

trait PipelineStage {
    fn handle(&mut self, tree: &LosslessTreeSink2, node: &SyntaxNode2);
}

impl PipelineStage for () {
    fn handle(&mut self, tree: &LosslessTreeSink2, node: &SyntaxNode2) {}
}

macro_rules! stages {
    ($name:tt, $($stage:tt: $types:tt,)*) => {
        #[derive(Default)]
        struct $name<$($types,)*>
        where
            $($types: PipelineStage,)*
        {
            $($stage: $types,)*
        }

        impl<$($types,)*> PipelineStage for $name<$($types,)*>
        where
            $($types: PipelineStage,)*
        {
            fn handle(&mut self, tree: &LosslessTreeSink2, node: &SyntaxNode2) {
                $(self.$stage.handle(tree, node);)*
            }
        }
    };
}

stages!(
    Pipeline10,
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
    j: J,
);

#[derive(Default)]
struct LinkedListStages<TCurrent, TNext>
where
    TCurrent: PipelineStage,
    TNext: PipelineStage,
{
    current: TCurrent,
    next: TNext,
}

impl<TCurrent, TNext> LinkedListStages<TCurrent, TNext>
where
    TCurrent: PipelineStage,
    TNext: PipelineStage,
{
    fn new(current: TCurrent, next: TNext) -> Self
    where
        TCurrent: std::default::Default,
        TNext: std::default::Default,
    {
        Self {
            current: std::default::Default::default(),
            next: std::default::Default::default(),
        }
    }
}

impl<TCurrent, TNext> PipelineStage for LinkedListStages<TCurrent, TNext>
where
    TCurrent: PipelineStage,
    TNext: PipelineStage,
{
    fn handle(&mut self, tree: &LosslessTreeSink2, node: &SyntaxNode2) {
        self.current.handle(tree, node);
        self.next.handle(tree, node);
    }
}

type Stages2<A, B> = LinkedListStages<A, B>;
type Stages3<A, B, C> = Stages2<A, Stages2<B, C>>;
type Stages4<A, B, C, D> = Stages3<A, B, Stages2<C, D>>;
type Stages5<A, B, C, D, E> = Stages4<A, B, C, Stages2<D, E>>;

struct DynPipeline {
    stages: Vec<Box<dyn PipelineStage>>,
}

impl DynPipeline {
    fn new() -> Self {
        Self { stages: vec![] }
    }

    fn push<T: 'static + PipelineStage>(&mut self, stage: T) {
        self.stages.push(Box::new(stage));
    }

    pub fn run(&mut self, tree: &LosslessTreeSink2) {
        let v: Vec<_> = tree.all().collect();

        let a = Instant::now();

        for node in v {
            match node {
                rome_js_syntax::WalkEvent::Enter(node) => {
                    for stage in self.stages.iter_mut() {
                        stage.handle(tree, &node);
                    }
                }
                rome_js_syntax::WalkEvent::Leave(_) => {}
            }
        }

        let took = Instant::now() - a;
        println!("DynPipeline: took: {:?}", took);
    }
}

struct Pipeline<TStage>
where
    TStage: PipelineStage,
{
    stage: TStage,
}

impl<TStage> Pipeline<TStage>
where
    TStage: PipelineStage,
{
    fn new() -> Self
    where
        TStage: std::default::Default,
    {
        Self {
            stage: std::default::Default::default(),
        }
    }

    pub fn run(&mut self, tree: &LosslessTreeSink2) {
        let v: Vec<_> = tree.all().collect();

        let a = Instant::now();

        for node in v {
            match node {
                rome_js_syntax::WalkEvent::Enter(node) => {
                    self.stage.handle(tree, &node);
                }
                rome_js_syntax::WalkEvent::Leave(_) => {}
            }
        }

        let took = Instant::now() - a;
        println!("Pipeline: took: {:?}", took);
    }
}

#[derive(Default)]
struct CountFunctionsStage(u64);

impl PipelineStage for CountFunctionsStage {
    fn handle(&mut self, tree: &LosslessTreeSink2, node: &SyntaxNode2) {
        // match node {
        //     rome_js_syntax::NodeOrToken::Node(node) => {
        //         if node.kind() == JsSyntaxKind::JS_FUNCTION_DECLARATION {
        //             self.0 += 1;
        //         }
        //     }
        //     rome_js_syntax::NodeOrToken::Token(_) => {}
        // }

        let kind = tree.kinds[node.pos];
        if kind == JsSyntaxKind::JS_FUNCTION_DECLARATION {
            self.0 += 1;
        }
    }
}

#[inline(never)]
fn b(result: LosslessTreeSink2) {
    let mut p = DynPipeline::new();
    for _ in 0..10 {
        p.push(CountFunctionsStage::default());
    }
    p.run(&result);

    let mut pipeline: Pipeline<
        Pipeline10<
            CountFunctionsStage,
            CountFunctionsStage,
            CountFunctionsStage,
            CountFunctionsStage,
            CountFunctionsStage,
            CountFunctionsStage,
            CountFunctionsStage,
            CountFunctionsStage,
            CountFunctionsStage,
            CountFunctionsStage,
        >,
    > = Pipeline::new();
    pipeline.run(&result);
}

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let text = std::fs::read_to_string(input).unwrap();

    // println!("Today");
    // println!("-----");

    let a = Instant::now();
    let result = rome_js_parser::parse(&text, 0, SourceType::js_script());
    let took = Instant::now() - a;
    // println!("parse: {:?}", took);

    // syntax_pipeline_1(&result);
    // syntax_pipeline_10(&result);
    // syntax_pipeline_100(&result);
    // syntax_pipeline_100_cached(&result);

    // println!("Home Made Flat Tree");
    // println!("-----");

    let a = Instant::now();
    let result = rome_js_parser::parse2(&text, 0, SourceType::js_script());
    let took = Instant::now() - a;
    println!("parse: {:?}", took);

    // syntax2_pipeline_1(&result);
    // syntax2_pipeline_10(&result);
    // syntax2_pipeline_100(&result);
    // syntax2_pipeline_100_cached(&result);

    // println!("SIMD");
    // println!("-----");

    // syntax2_simd_pipeline_1(&result);
    // syntax2_simd_pipeline_10(&result);
    // syntax2_simd_pipeline_100(&result);

    b(result);
}
