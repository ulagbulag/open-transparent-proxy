const NAME: &str = "notion";

pub const RESPONSE_FILTER_BUILDER: [super::super::base::regex::ResponseFilterBuilder; 2] = [
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"(domainBaseUrl:")https?://[a-z\.]+""#,
        rep: r#"${{1}}{scheme}://{host}""#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"(publicDomainName:")[a-z\.]+""#,
        rep: r#"${{1}}{host}""#,
    },
];
