use html5ever::tree_builder::{NodeOrText, TreeSink};
use markup5ever::QualName;
#[cfg(test)]
use scraper::Selector;
use scraper::{Html, Node};
#[cfg(test)]
use xml_utils::html_attr_name;

pub(crate) enum DomOperation<Handle> {
    /// Will remove the attribute `attr_name` of element `node_id`.
    RemoveAttribute {
        node_id: Handle,
        attr_name: QualName,
    },
    /// Will set the value of attribute `attr_name` of element `node_id` to `value`.
    SetAttribute {
        node_id: Handle,
        attr_name: QualName,
        value: String,
    },
    /// Will remove all children of element with given `node_id`.
    RemoveChildren {
        node_id: Handle,
    },
    /// Will add an element with `child_id` as the last child of element with `parent_id`.
    /// If the child already has a parent, it will be moved.
    AppendChild {
        parent_id: Handle,
        child_id: Handle,
    },
    /// Will remove element with `node_id` and put replacement in its place in the tree.
    ReplaceElement {
        node_id: Handle,
        replacement: NodeOrText<Handle>,
    },
    /// Replaces element with `node_id` by its children,
    /// essentially removing the element’s opening and closing tags.
    DissolveElement {
        node_id: Handle,
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
            DomOperation::RemoveAttribute { node_id, attr_name } => {
                if let Some(mut node) = self.tree.get_mut(node_id) {
                    if let Node::Element(ref mut elem) = node.value() {
                        elem.attrs.remove(&attr_name);
                    }
                }
            }
            DomOperation::SetAttribute {
                node_id,
                attr_name,
                value,
            } => {
                if let Some(mut node) = self.tree.get_mut(node_id) {
                    if let Node::Element(ref mut elem) = node.value() {
                        elem.attrs.insert(attr_name, value.into());
                    }
                }
            }
            DomOperation::RemoveChildren { node_id } => {
                let mut child_ids = Vec::new();
                if let Some(node) = self.tree.get(node_id) {
                    for child in node.children() {
                        child_ids.push(child.id());
                    }
                }
                for id in child_ids {
                    self.remove_from_parent(&id);
                }
            }
            DomOperation::AppendChild { parent_id, child_id } => {
                if let Some(mut node) = self.tree.get_mut(parent_id) {
                    node.append_id(child_id);
                }
            }
            DomOperation::ReplaceElement { node_id, replacement } => {
                self.append_before_sibling(&node_id, replacement);
                self.remove_from_parent(&node_id);
            }
            DomOperation::DissolveElement { node_id } => {
                let mut child_ids = Vec::new();
                if let Some(node) = self.tree.get(node_id) {
                    for child in node.children() {
                        child_ids.push(child.id());
                    }
                }

                for id in child_ids {
                    self.append_before_sibling(&node_id, NodeOrText::AppendNode(id));
                }

                self.remove_from_parent(&node_id);
            }
        }
    }
}

#[test]
fn test_remove_attribute() {
    let mut doc = Html::parse_fragment("<em title='foo'>Emphasis</em> <strong id='important' title='bar' data-test='other'>Bold</strong> <del title='bar'>Wrong</del>");
    let strong = doc.select(&Selector::parse("strong").unwrap()).next().expect("Strong tag not found.");

    doc.perform_operation(DomOperation::RemoveAttribute {
        node_id: strong.id(),
        attr_name: html_attr_name("title"),
    });

    assert_eq!(
        Html::parse_fragment("<em title='foo'>Emphasis</em> <strong id='important' data-test='other'>Bold</strong> <del title='bar'>Wrong</del>"),
        Html::parse_fragment(&doc.root_element().inner_html()),
    );
}

#[test]
fn test_set_attribute() {
    let mut doc = Html::parse_fragment("<em title='foo'>Emphasis</em> <strong data-test='other'>Bold</strong> <del title='bar'>Wrong</del>");
    let strong = doc.select(&Selector::parse("strong").unwrap()).next().expect("Strong tag not found.");

    doc.perform_operation(DomOperation::SetAttribute {
        node_id: strong.id(),
        attr_name: html_attr_name("id"),
        value: String::from("Bar"),
    });

    assert_eq!(
        Html::parse_fragment("<em title='foo'>Emphasis</em> <strong id='Bar' data-test='other'>Bold</strong> <del title='bar'>Wrong</del>"),
        Html::parse_fragment(&doc.root_element().inner_html()),
    );
}

#[test]
fn test_set_attribute_existing() {
    let mut doc = Html::parse_fragment("<em title='foo'>Emphasis</em> <strong id='Foo' data-test='other'>Bold</strong> <del title='bar'>Wrong</del>");
    let strong = doc.select(&Selector::parse("strong").unwrap()).next().expect("Strong tag not found.");

    doc.perform_operation(DomOperation::SetAttribute {
        node_id: strong.id(),
        attr_name: html_attr_name("id"),
        value: String::from("Bar"),
    });

    assert_eq!(
        Html::parse_fragment("<em title='foo'>Emphasis</em> <strong id='Bar' data-test='other'>Bold</strong> <del title='bar'>Wrong</del>"),
        Html::parse_fragment(&doc.root_element().inner_html()),
    );
}

#[test]
fn test_remove_children() {
    let mut doc = Html::parse_fragment("<div><p class='nop'><em>Foo</em></p><p class='figure'><em>Empha<strong title='secret'>sis</strong></em> <strong>Bold</strong></p> <a href='#'>After</a></div>");
    let figure = doc.select(&Selector::parse(".figure").unwrap()).next().expect("Figure not found.");

    doc.perform_operation(DomOperation::RemoveChildren {
        node_id: figure.id(),
    });

    assert_eq!(
        Html::parse_fragment("<div><p class='nop'><em>Foo</em></p><p class='figure'></p> <a href='#'>After</a></div>"),
        Html::parse_fragment(&doc.root_element().inner_html()),
    );
}

#[test]
fn test_append_child() {
    let mut doc = Html::parse_fragment("<em>Emphasis</em> <strong>Bold</strong> <del>Wrong</del>");
    let strong = doc.select(&Selector::parse("strong").unwrap()).next().expect("Strong tag not found.");

    doc.perform_operation(DomOperation::AppendChild {
        parent_id: doc.root_element().id(),
        child_id: strong.id(),
    });

    assert_eq!(
        Html::parse_fragment("<em>Emphasis</em>  <del>Wrong</del><strong>Bold</strong>"),
        Html::parse_fragment(&doc.root_element().inner_html()),
    );
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

#[test]
fn test_dissolve_element() {
    let mut doc = Html::parse_fragment("<div><p class='nop'><em>Foo</em></p><p class='figure'><em>Empha<strong title='secret'>sis</strong></em> <strong>Bold</strong></p> <a href='#'>After</a></div>");
    let figure = doc.select(&Selector::parse(".figure").unwrap()).next().expect("Figure not found.");

    doc.perform_operation(DomOperation::DissolveElement {
        node_id: figure.id(),
    });

    assert_eq!(
        Html::parse_fragment("<div><p class='nop'><em>Foo</em></p><em>Empha<strong title='secret'>sis</strong></em> <strong>Bold</strong> <a href='#'>After</a></div>"),
        Html::parse_fragment(&doc.root_element().inner_html()),
    );
}
