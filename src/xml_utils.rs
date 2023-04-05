use ego_tree::iter::{Edge, Traverse};
use markup5ever::{LocalName, QualName};
use scraper::node::Element;
use scraper::{ElementRef, Node};
#[cfg(test)]
use scraper::{Html, Selector};
use xml5ever::serialize::{serialize, SerializeOpts, TraversalScope};

pub(crate) trait XmlSerializable {
    fn serialize_xml(&self, traversal_scope: TraversalScope) -> String;

    /// Returns the XML of this element.
    fn xml(&self) -> String {
        self.serialize_xml(TraversalScope::IncludeNode)
    }

    /// Returns the inner XML of this element.
    fn inner_xml(&self) -> String {
        self.serialize_xml(TraversalScope::ChildrenOnly(None))
    }
}

impl<'a> XmlSerializable for ElementRef<'a> {
    fn serialize_xml(&self, traversal_scope: TraversalScope) -> String {
        let opts = SerializeOpts { traversal_scope };
        let mut buf = Vec::new();
        serialize(&mut buf, self, opts).unwrap();
        String::from_utf8(buf)
            .unwrap()
            // The serializer does not support XML fragments,
            // we need to remove redundant namespace attributes.
            .replace(r#" xmlns="http://www.w3.org/1999/xhtml""#, "")
    }
}

/// Creates a qualified name for a HTML element.
pub fn html_elem_name(name: &str) -> QualName {
    QualName::new(None, ns!(html), LocalName::from(name))
}

/// Helper trait that will allow us to call `text_filter` on `ElementRef`.
pub(crate) trait FilterableTree<'a, P> {
    fn text_filter(&self, is_allowed: P) -> TextFiltered<'a, P>
    where
        Self: Sized,
        P: FnMut(&Element) -> bool;
}

impl<'a, P> FilterableTree<'a, P> for ElementRef<'a> {
    fn text_filter(&self, is_allowed: P) -> TextFiltered<'a, P>
    where
        Self: Sized,
        P: FnMut(&Element) -> bool,
    {
        TextFiltered {
            inner: self.traverse(),
            is_in_filtered: 0,
            is_allowed,
        }
    }
}

/// Iterator over descendent text nodes
/// that allows pruning elements in the tree using a predicate.
pub struct TextFiltered<'a, P> {
    /// Internal tree iterator.
    inner: Traverse<'a, Node>,
    /// Counter representing how deeply we are in removed node, element-wise.
    is_in_filtered: usize,
    /// When this predicate returns false on a node,
    /// its child text nodes will not be yielded from the iterator.
    is_allowed: P,
}

impl<'a, P> Iterator for TextFiltered<'a, P>
where
    Self: Sized,
    P: FnMut(&Element) -> bool,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        for edge in &mut self.inner {
            match edge {
                Edge::Open(node) => {
                    match node.value() {
                        Node::Element(ref element) => {
                            if !(self.is_allowed)(element) || self.is_in_filtered > 0 {
                                // If we enter a forbidden element, or dive in deeper inside it.
                                self.is_in_filtered += 1;
                            }
                        }
                        Node::Text(ref text) => {
                            if self.is_in_filtered == 0 {
                                return Some(&**text);
                            }
                        }
                        _ => {
                            // Other types of nodes do not affect what is yielded
                            // or whether we should move in the stack.
                        }
                    }
                }
                Edge::Close(node) => {
                    if let (Node::Element(_), true) = (node.value(), self.is_in_filtered > 0) {
                        self.is_in_filtered -= 1;
                    }
                }
            }
        }
        None
    }
}

#[test]
fn test_filter_text_iterator() {
    let doc = Html::parse_fragment(
        "<p>hullo <a href='prev'>Prev chap</a> hey <a href='prev'>Next <strong>chap</strong></a><em>foo</em></p>",
    );
    let p = doc.select(&Selector::parse("p").unwrap()).next().unwrap();
    assert_eq!(
        "hullo  hey foo",
        p.text_filter(|elem| elem.name != html_elem_name("a")).collect::<String>()
    );
}
