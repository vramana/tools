use rome_js_parser::{
    lossless_tree_sink::{LosslessTreeSink2, SyntaxNode2},
    Parse, SourceType,
};
use rome_js_syntax::{JsAnyRoot, JsSyntaxKind};
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

    // let a = Instant::now();
    // let result = rome_js_parser::parse2(&text, 0, SourceType::js_script());
    // let took = Instant::now() - a;
    // println!("parse: {:?}", took);

    // syntax2_pipeline_1(&result);
    // syntax2_pipeline_10(&result);
    // syntax2_pipeline_100(&result);
    // syntax2_pipeline_100_cached(&result);

    // println!("SIMD");
    // println!("-----");

    // syntax2_simd_pipeline_1(&result);
    // syntax2_simd_pipeline_10(&result);
    // syntax2_simd_pipeline_100(&result);
}
