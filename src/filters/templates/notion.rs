/// Referred from: https://github.com/stephenou/fruitionsite.git

const NAME: &str = "notion";

pub const RESPONSE_FILTER_BUILDER: [super::super::base::regex::ResponseFilterBuilder; 3] = [
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"(domainBaseUrl:")https?://[\.\/a-z]+""#,
        rep: r#"${{1}}{scheme}://{base_url_with_host}""#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"(publicDomainName:")[\.a-z]+""#,
        rep: r#"${{1}}{host}""#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"<body[ \.\_\-\=A-Za-z0-9'"]*>"#,
        rep: r#"${{0}}<script>(function() {{ var proxied = window.history.replaceState; window.history.replaceState = function(state) {{ if (arguments[1] !== 'bypass') return; return proxied.apply(window.history, arguments); }}; }})();</script>"#,
    },
];
