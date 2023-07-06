pub struct ResponseFilterBuilder<'a> {
    pub name: &'a str,
    pub re: &'a str,
    pub rep: &'a str,
}

impl super::super::templates::ResponseFilterBuilder for ResponseFilterBuilder<'static> {
    type FILTER = ResponseFilter;

    fn try_build(
        self,
    ) -> ::anyhow::Result<<Self as super::super::templates::ResponseFilterBuilder>::FILTER> {
        let Self { name, re, rep } = self;

        Ok(ResponseFilter {
            regex: ::regex::Regex::new(re).map_err(|e| {
                ::anyhow::anyhow!("failed to init a regex response filter ({name}): {e}")
            })?,
            rep,
        })
    }
}

pub struct ResponseFilter {
    regex: ::regex::Regex,
    rep: &'static str,
}

impl super::super::templates::ResponseFilter for ResponseFilter {
    fn filter(&self, config: &crate::config::ConfigMap, body: String) -> String {
        let Self { regex, rep } = self;

        match ::strfmt::strfmt(rep, config) {
            Ok(rep) => regex.replace_all(&body, rep).into_owned(),
            Err(_) => body,
        }
    }
}
