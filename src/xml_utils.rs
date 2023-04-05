use scraper::ElementRef;
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
