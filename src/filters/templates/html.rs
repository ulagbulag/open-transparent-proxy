const NAME: &str = "html";

pub const RESPONSE_FILTER_BUILDER: [super::super::base::regex::ResponseFilterBuilder; 4] = [
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"(href=")/"#,
        rep: r#"${{1}}"#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"(src=")/"#,
        rep: r#"${{1}}"#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"(url=")/"#,
        rep: r#"${{1}}"#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"<head[ \.\_\-\=A-Za-z0-9'"]*>"#,
        rep: r#"${{0}}<base href="{base_url}">"#,
    },
];
