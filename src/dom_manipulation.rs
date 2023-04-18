use html5ever::tree_builder::{NodeOrText, TreeSink};
use scraper::Html;
#[cfg(test)]
use scraper::Selector;

pub(crate) enum DomOperation<Handle> {
    /// Will remove element with `node_id` and put replacement in its place in the tree.
    ReplaceElement {
        node_id: Handle,
        replacement: NodeOrText<Handle>,
    },
}

pub(crate) trait MutableDom: TreeSink {
    /// Performs given operation to mutate the tree.
    fn perform_operation(&mut self, op: DomOperation<Self::Handle>);

    /// Performs a sequence of scheduled operations.
    fn perform_operations(&mut self, ops: Vec<DomOperation<Self::Handle>>) {
        for op in ops {
            self.perform_operation(op);
        }
    }
}

impl MutableDom for Html {
    fn perform_operation(&mut self, op: DomOperation<Self::Handle>) {
        match op {
            DomOperation::ReplaceElement { node_id, replacement } => {
                self.append_before_sibling(&node_id, replacement);
                self.remove_from_parent(&node_id);
            }
        }
    }
}

#[test]
fn test_replace_element() {
    let mut doc = Html::parse_fragment("<em>Emphasis</em> <strong>Bold</strong> <del>Wrong</del>");
    let strong = doc.select(&Selector::parse("strong").unwrap()).next().expect("Strong tag not found.");

    doc.perform_operation(DomOperation::ReplaceElement {
        node_id: strong.id(),
        replacement: NodeOrText::AppendText("Foo".into()),
    });

    assert_eq!(
        Html::parse_fragment("<em>Emphasis</em> Foo <del>Wrong</del>"),
        Html::parse_fragment(&doc.root_element().inner_html()),
    );
}
