const NAME: &str = "html";

pub const RESPONSE_FILTER_BUILDER: [super::super::base::regex::ResponseFilterBuilder; 9] = [
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
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"<body[ \.\_\-\=A-Za-z0-9'"]*>"#,
        rep: r#"${{0}}<script>(function() {{ var proxied = Element.prototype.appendChild; Element.prototype.appendChild = function() {{ if (arguments[0].src !== undefined && arguments[0].src.startsWith('/') && !arguments[0].src.startsWith('{base_url}')) {{ arguments[0].src = arguments[0].src.replace('/', '{base_url}'); }} if (arguments[0].src !== undefined && arguments[0].src.startsWith('{scheme}://{host}/') && !arguments[0].src.startsWith('{scheme}://{base_url_with_host}')) {{ arguments[0].src = arguments[0].src.replace('{scheme}://{host}/', '{scheme}://{base_url_with_host}'); }} if (arguments[0].href !== undefined && arguments[0].href.startsWith('{scheme}://{host}/') && !arguments[0].href.startsWith('{scheme}://{base_url_with_host}')) {{ arguments[0].href = arguments[0].href.replace('{scheme}://{host}/', '{scheme}://{base_url_with_host}'); }} return proxied.apply(this, [].slice.call(arguments)); if (arguments[0].url !== undefined && arguments[0].url.startsWith('{scheme}://{host}/') && !arguments[0].url.startsWith('{scheme}://{base_url_with_host}')) {{ arguments[0].url = arguments[0].url.replace('{scheme}://{host}/', '{scheme}://{base_url_with_host}'); }} }}; }})()</script>"#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"<body[ \.\_\-\=A-Za-z0-9'"]*>"#,
        rep: r#"${{0}}<script>(function() {{ var proxied = Element.prototype.setAttribute; Element.prototype.setAttribute = function(key, value) {{ if (['href', 'src', 'url'].includes(key)) {{ if (value.startsWith('/') && !value.startsWith('{base_url}')) {{ value = value.replace('/', '{base_url}'); }} else if (value.startsWith('{scheme}://{host}/') && !value.startsWith('{scheme}://{base_url_with_host}')) {{ value = value.replace('{scheme}://{host}/', '{scheme}://{base_url_with_host}'); }} }} return proxied.apply(this, [key, value]); }}; }})();</script>"#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"<body[ \.\_\-\=A-Za-z0-9'"]*>"#,
        rep: r#"${{0}}<script>(function() {{ var proxied = window.XMLHttpRequest.prototype.open; window.XMLHttpRequest.prototype.open = function() {{ if (arguments[1].startsWith('{scheme}://{host}/') && !arguments[1].startsWith('{scheme}://{base_url_with_host}')) {{ arguments[1] = arguments[1].replace('{scheme}://{host}/', '{scheme}://{base_url_with_host}'); }} return proxied.apply(this, [].slice.call(arguments)); }}; }})();</script>"#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"<body[ \.\_\-\=A-Za-z0-9'"]*>"#,
        rep: r#"${{0}}<script>(function() {{ var proxied = window.XMLHttpRequest.prototype.send; window.XMLHttpRequest.prototype.send = function() {{ if (arguments[1].startsWith('{scheme}://{host}/') && !arguments[1].startsWith('{scheme}://{base_url_with_host}')) {{ arguments[1] = arguments[1].replace('{scheme}://{host}/', '{scheme}://{base_url_with_host}'); }} return proxied.apply(this, [].slice.call(arguments)); }}; }})();</script>"#,
    },
    super::super::base::regex::ResponseFilterBuilder {
        name: NAME,
        re: r#"<body[ \.\_\-\=A-Za-z0-9'"]*>"#,
        rep: r#"${{0}}<script>(function() {{ var proxied = window.fetch; window.fetch = function() {{ if (arguments[0].startsWith('/') && !arguments[0].startsWith('{base_url}')) {{ arguments[0] = arguments[0].replace('/', '{base_url}'); }} return Promise.resolve(proxied.apply(this, [].slice.call(arguments))); }}; }})();</script>"#,
    },
];
