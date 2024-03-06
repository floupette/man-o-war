use super::{
	description::Description,
	fragment::Fragment,
	html_element::HtmlElement
};

/// Represents one of several possible methods.
#[derive(Debug)]
pub struct Method {
    pub signature: Vec<Fragment>,
    pub description: Description,
}

impl Method {
    /// Parses the one method into its signature and description.
    pub fn parse(data: &str) -> Self {
        Self {
            signature: HtmlElement::extract("h4", data).zip_content(),
            description: Description::parse(data),
        }
    }
}
