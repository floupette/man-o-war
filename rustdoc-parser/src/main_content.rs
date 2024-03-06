use crate::{
    helper_types::{
        fragment::Fragment,
        html_element::HtmlElement,
        method::Method
    }
};

/// The main content of a single page, from the introduction to the last of its implementation blocks.
#[derive(Debug)]
pub struct MainContent(pub Vec<Section>);

/// One of the main content's sections.
#[derive(Debug)]
pub struct Section {
    pub name: Fragment,
    pub content: SectionContent,
}

impl MainContent {
    /// Parses the main content.
    pub fn parse(raw: &str) -> Self {
        let sections = raw
            .split("<h2")
            .skip(1)
            .fold(Vec::new(), |mut sections, str| {
                sections.push(format!("<h2{}", str));
                sections
            });

        MainContent(
            sections
                .iter()
                .map(|section| {
                    let mut split = section.split_inclusive("</h2>").collect::<Vec<&str>>();
                    let name = String::from(HtmlElement::parse(&mut split[0]).content[0].raw_content());
                    let content = match name.as_str() {
                        "Auto Trait Implementations" => SectionContent::parse_auto_trait_implementations(split[1]),
                        "Blanket Implementations" => SectionContent::parse_blanket_implementations(split[1]),
                        "Fields" | "Tuple Fields" => SectionContent::parse_fields(split[1]),
                        "Implementations" => SectionContent::parse_implementations(split[1]),
                        "Object Safety" => SectionContent::parse_object_safety(split[1]),
                        "Required Associated Types" => SectionContent::parse_required_associated_types(split[1]),
                        "Trait Implementations" => SectionContent::parse_trait_implementations(split[1]),
                        "Variants" => SectionContent::parse_variants(split[1]),
                        _ => unreachable!()
                    };
                    Section {
                        name: Fragment::Bold(Vec::from([Fragment::Raw(name)])),
                        content,
                    }
                })
                .collect(),
        )
    }
}

/// Represents the content of one of the MainContent's Sections.
#[derive(Debug)]
pub enum SectionContent {
    Dummy,
    Fields(Vec<Field>),
    Implementations(Vec<Implementation>),
    ObjectSafety(Vec<Fragment>),
    RequiredAssociatedTypes(Vec<RequiredAssociatedType>),
    TraitImplementations(Vec<TraitImplementation>),
    Variants(Vec<Variant>),
}

/// Represents a single struct field.
#[derive(Debug)]
pub struct Field {
    pub content: Vec<Fragment>,
    pub description: Vec<Fragment>,
}

/// Represents a single implementation for one specific type.
#[derive(Debug)]
pub struct Implementation {
    pub inherent_impl: Fragment,
    pub methods: Vec<Method>,
}

/// Represents one of a trait's required associated types.
#[derive(Debug)]
pub struct RequiredAssociatedType {
    pub name: Fragment,
    pub description: Fragment,
}

/// Represents a single trait implementation for one specific type.
#[derive(Debug)]
pub struct TraitImplementation {
    pub trait_impl: Fragment,
    pub methods: Vec<Method>,
}

/// Represents one of an enum's variants.
#[derive(Debug)]
pub struct Variant {
    pub name: Fragment,
    pub description: Vec<Fragment>,
}

impl SectionContent {
    /// Parses the fields of a struct, be they named or tuple fields.
    pub fn parse_fields(data: &str) -> SectionContent {
        SectionContent::Fields(
            data.split_terminator("</div>")
                .map(|field| {
                    let (_content, _description) = field.split_once("</span>").unwrap();
                    Field {
                        content: HtmlElement::extract("code", _content).zip_content(),
                        description: HtmlElement::extract("p", _description).zip_content(),
                    }
                })
                .collect(),
        )
    }
    /// Parses the Implementations section.
    pub fn parse_implementations(data: &str) -> SectionContent {
        Self::Implementations(
            data.split("<details class=\"toggle implementors-toggle\" open>")
                .skip(1)
                .map(|imp| {
                    let (unfiltered_content, _methods) = imp.split_once("</summary>").unwrap();
                    let inherent_impl =
                        Fragment::Bold(HtmlElement::extract("h3", unfiltered_content).zip_content());
                    let methods = _methods
                        .trim_end_matches("</div></details>")
                        .split("<details class=\"toggle method-toggle\" open>")
                        .skip(1)
                        .map(|method| Method::parse(method))
                        .collect();
                    Implementation {
                        inherent_impl,
                        methods,
                    }
                })
                .collect(),
        )
    }
    /// Parses a type's auto Trait Implementations.
    pub fn parse_auto_trait_implementations(data: &str) -> Self {
        Self::TraitImplementations(
            data.split("</section>")
                .filter(|split| *split != "</div>")
                .map(|implementation| TraitImplementation {
                    trait_impl: Fragment::Bold(HtmlElement::extract("h3", implementation).zip_content()),
                    methods: Vec::new(),
                })
                .collect(),
        )
    }
    /// Parses a type's blanket Trait Implementations.
    pub fn parse_blanket_implementations(data: &str) -> SectionContent {
        Self::TraitImplementations(
            data.split("<details class=\"toggle implementors-toggle\">")
                .skip(1)
                .map(|imp| {
                    let (unfiltered_content, _methods) = imp.split_once("</summary>").unwrap();
                    let trait_impl =
                        Fragment::Bold(HtmlElement::extract("h3", unfiltered_content).zip_content());
                    let methods = _methods
                        .trim_end_matches("</div></details>")
                        .split("<details class=\"toggle method-toggle\">")
                        .skip(1)
                        .map(|method| Method::parse(method))
                        .collect();
                    TraitImplementation {
                        trait_impl,
                        methods,
                    }
                })
                .collect(),
        )
    }
    /// Parses a type's Trait Implementations.
    pub fn parse_trait_implementations(data: &str) -> SectionContent {
        Self::TraitImplementations(
            data.split("</details>")
                .skip(1)
                .filter(|split| *split != "</div>")
                .map(|_impl| {
                    let trait_impl = Fragment::Bold(HtmlElement::extract("h3", _impl).zip_content());
                    let methods = if let Some((_, _methods))= _impl.split_once("</summary>") {
                        _methods
                            .trim_start_matches("<div class=\"impl-items\">")
                            .split_inclusive("<details")
                            .filter(|split| *split != "<details")
                            .map(|method| Method::parse(method))
                            .collect()
                    } else {
                        Vec::new()
                    };
                    TraitImplementation {
                        trait_impl,
                        methods,
                    }
                })
                .collect(),
        )
    }
    /// Parses the optional Object Safety section.
    pub fn parse_object_safety(mut data: &str) -> SectionContent {
        Self::ObjectSafety(HtmlElement::parse(&mut data).zip_content())
    }
    /// Parses a Trait's optional required associated types.
    pub fn parse_required_associated_types(data: &str) -> SectionContent {
        Self::RequiredAssociatedTypes(
            data.trim_end_matches("</div>")
                .split_terminator("</details>")
                .map(|associated_type| RequiredAssociatedType {
                    name: Fragment::Bold(HtmlElement::extract("h4", associated_type).zip_content()),
                    description: HtmlElement::extract("p", associated_type).content[0].clone(),
                })
                .collect(),
        )
    }
    /// Parses an Enum's Variants.
    pub fn parse_variants(data: &str) -> SectionContent {
        Self::Variants(
            data.split("</a>")
                .skip(1)
                .map(|variant| Variant {
                    name: Fragment::Bold(HtmlElement::extract("h3", variant).zip_content()),
                    description: HtmlElement::extract("p", variant).zip_content(),
                })
                .collect(),
        )
    }
}
