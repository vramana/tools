use crate::token_source::Trivia;
use crate::{ParseDiagnostic, TreeSink};
use rome_js_syntax::{JsSyntaxKind, SyntaxNode, SyntaxTreeBuilder, TextRange, TextSize, WalkEvent};
use rome_rowan::TriviaPiece;

/// Structure for converting events to a syntax tree representation, while preserving whitespace.
///
/// `LosslessTreeSink` also handles attachment of trivia (whitespace) to nodes.
#[derive(Debug)]
pub struct LosslessTreeSink<'a> {
    text: &'a str,
    trivia_list: &'a [Trivia],
    text_pos: TextSize,
    trivia_pos: usize,
    parents_count: usize,
    errors: Vec<ParseDiagnostic>,
    inner: SyntaxTreeBuilder,
    /// Signal that the sink must generate an EOF token when its finishing. See [LosslessTreeSink::finish] for more details.
    needs_eof: bool,
    trivia_pieces: Vec<TriviaPiece>,
}

impl<'a> TreeSink for LosslessTreeSink<'a> {
    fn token(&mut self, kind: JsSyntaxKind, length: TextSize) {
        self.do_token(kind, length);
    }

    fn start_node(&mut self, kind: JsSyntaxKind) {
        self.inner.start_node(kind);
        self.parents_count += 1;
    }

    fn finish_node(&mut self) {
        self.parents_count -= 1;

        if self.parents_count == 0 && self.needs_eof {
            self.do_token(JsSyntaxKind::EOF, TextSize::default());
        }

        self.inner.finish_node();
    }

    fn errors(&mut self, errors: Vec<ParseDiagnostic>) {
        self.errors = errors;
    }
}

impl<'a> LosslessTreeSink<'a> {
    pub fn new(text: &'a str, trivia: &'a [Trivia]) -> Self {
        Self {
            text,
            trivia_list: trivia,
            text_pos: 0.into(),
            trivia_pos: 0,
            parents_count: 0,
            inner: SyntaxTreeBuilder::default(),
            errors: vec![],
            needs_eof: true,
            trivia_pieces: Vec::with_capacity(128),
        }
    }

    /// Finishes the tree and return the root node with possible parser errors.
    ///
    /// If tree is finished without a [SyntaxKind::EOF], one will be generated and all pending trivia
    /// will be appended to its leading trivia.
    pub fn finish(self) -> (SyntaxNode, Vec<ParseDiagnostic>) {
        (self.inner.finish(), self.errors)
    }

    #[inline]
    fn do_token(&mut self, kind: JsSyntaxKind, length: TextSize) {
        if kind == JsSyntaxKind::EOF {
            self.needs_eof = false;
        }

        let token_start = self.text_pos;

        // Every trivia up to the token (including line breaks) will be the leading trivia
        self.eat_trivia(false);
        let trailing_start = self.trivia_pieces.len();

        self.text_pos += length;

        // Everything until the next linebreak (but not including it)
        // will be the trailing trivia...
        self.eat_trivia(true);

        let token_range = TextRange::new(token_start, self.text_pos);

        let text = &self.text[token_range];
        let leading = &self.trivia_pieces[0..trailing_start];
        let trailing = &self.trivia_pieces[trailing_start..];

        self.inner.token_with_trivia(kind, text, leading, trailing);
        self.trivia_pieces.clear();
    }

    fn eat_trivia(&mut self, trailing: bool) {
        for trivia in &self.trivia_list[self.trivia_pos..] {
            if trailing != trivia.trailing() || self.text_pos != trivia.offset() {
                break;
            }

            let trivia_piece = TriviaPiece::new(trivia.kind(), trivia.len());
            self.trivia_pieces.push(trivia_piece);

            self.text_pos += trivia.len();
            self.trivia_pos += 1;
        }
    }
}

#[derive(Clone, Copy)]
pub struct SyntaxNode2 {
    pub pos: usize,
}

impl SyntaxNode2 {
    fn first_children(&self, tree: &LosslessTreeSink2) -> Option<SyntaxNode2> {
        tree.first_children[self.pos].map(|pos| SyntaxNode2 { pos })
    }

    fn next_sibling(&self, tree: &LosslessTreeSink2) -> Option<SyntaxNode2> {
        let my_depth = tree.depths[self.pos];

        let mut pos = self.pos + 1;
        loop {
            match tree.depths.get(pos) {
                Some(depth) => {
                    if tree.depths[pos] > my_depth {
                        pos += 1;
                    } else {
                        break;
                    }
                }
                None => return None,
            }
        }

        if tree.depths[pos] == my_depth {
            Some(SyntaxNode2 { pos })
        } else {
            None
        }
    }

    fn parent(&self, tree: &LosslessTreeSink2) -> Option<SyntaxNode2> {
        tree.parents[self.pos].map(|pos| SyntaxNode2 { pos })
    }
}

#[derive(Debug)]
pub struct LosslessTreeSink2<'a> {
    text: &'a str,
    trivia: Vec<Trivia>,
    pub kinds: Vec<JsSyntaxKind>,
    parents: Vec<Option<usize>>,
    first_children: Vec<Option<usize>>,
    depths: Vec<u16>,
    lengths: Vec<TextSize>,

    parent_stack: Vec<usize>,
    depth: u16,
    length_stack: Vec<TextSize>,
    length_idx_stack: Vec<usize>,
}

pub struct AllIterator<'a> {
    tree: &'a LosslessTreeSink2<'a>,
    next: Option<WalkEvent<SyntaxNode2>>,
}

impl<'a> Iterator for AllIterator<'a> {
    type Item = WalkEvent<SyntaxNode2>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();
        self.next = next.as_ref().and_then(|e| match e {
            WalkEvent::Enter(node) => {
                if let Some(first_children) = node.first_children(self.tree) {
                    Some(WalkEvent::Enter(first_children))
                } else {
                    Some(WalkEvent::Leave(node.clone()))
                }
            }
            WalkEvent::Leave(node) => {
                if let Some(sibling) = node.next_sibling(self.tree) {
                    Some(WalkEvent::Enter(sibling))
                } else if let Some(parent) = node.parent(self.tree) {
                    Some(WalkEvent::Leave(parent))
                } else {
                    None
                }
            }
        });

        next
    }
}

impl<'a> LosslessTreeSink2<'a> {
    pub fn new(text: &'a str, trivia: Vec<Trivia>, size_hint: usize) -> Self {
        Self {
            text,
            trivia,
            kinds: Vec::with_capacity(size_hint),
            parents: Vec::with_capacity(size_hint),
            first_children: Vec::with_capacity(size_hint),
            depths: Vec::with_capacity(size_hint),
            lengths: Vec::with_capacity(size_hint),

            depth: 0,
            parent_stack: Vec::with_capacity(16),
            length_stack: Vec::with_capacity(16),
            length_idx_stack: Vec::with_capacity(16),
        }
    }

    pub fn all(&'_ self) -> AllIterator<'_> {
        AllIterator {
            tree: self,
            next: Some(WalkEvent::Enter(SyntaxNode2 { pos: 0 })),
        }
    }
}

impl<'a> TreeSink for LosslessTreeSink2<'a> {
    fn start_node(&mut self, kind: JsSyntaxKind) {
        // tree structure
        let pos = self.parents.len();

        self.parents.push(self.parent_stack.last().cloned());
        self.depths.push(self.depth);
        self.first_children.push(None);

        if let Some(parent) = self.parent_stack.last() {
            let _ = self.first_children[*parent].get_or_insert(pos);
        }

        // node info
        self.kinds.push(kind);
        self.lengths.push(TextSize::of(""));

        self.parent_stack.push(pos);
        self.length_idx_stack.push(pos);
        self.length_stack.push(TextSize::of(""));

        self.depth += 1;
    }

    fn finish_node(&mut self) {
        self.depth -= 1;

        let _ = self.parent_stack.pop();
        let idx = self.length_idx_stack.pop().unwrap();
        let len = self.length_stack.pop().unwrap();

        self.lengths[idx] = len;

        if let Some(parent_len) = self.length_stack.last_mut() {
            *parent_len += len;
        }
    }

    fn token(&mut self, kind: JsSyntaxKind, length: TextSize) {
        // tree structure
        let pos = self.parents.len();

        self.parents.push(self.parent_stack.last().cloned());
        self.depths.push(self.depth);
        self.first_children.push(None);

        if let Some(parent) = self.parent_stack.last() {
            let _ = self.first_children[*parent].get_or_insert(pos);
        }

        // node info
        self.kinds.push(kind);
        self.lengths.push(length);

        *self.length_stack.last_mut().unwrap() += length;
    }

    fn errors(&mut self, errors: Vec<ParseDiagnostic>) {}
}
