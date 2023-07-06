pub struct DefaultResponseFilter;

macro_rules! impl_response_filter_builder_for_default_response_filter {
    ( $( $feature:expr => $mod:ident , )* ) => {
        $(
            #[cfg(feature = $feature)]
            mod $mod;
        )*

        impl ResponseFilterBuilder for DefaultResponseFilter {
            type FILTER = ResponseFilters;

            fn try_build(self) -> ::anyhow::Result<<Self as ResponseFilterBuilder>::FILTER> {
                // NOTE: ordered!
                Ok(ResponseFilters(
                    vec![
                        $({
                            #[cfg(feature = $feature)]
                            Box::new(self::$mod::RESPONSE_FILTER_BUILDER.try_build()?)
                        }),*
                    ]
                ))
            }
        }
    };
}

impl_response_filter_builder_for_default_response_filter!(
    "filter-html" => html,
    "filter-notion" => notion,
);

pub struct ResponseFilters(Vec<Box<dyn ResponseFilter>>);

impl ResponseFilter for ResponseFilters {
    fn filter(&self, config: &crate::config::ConfigMap, body: String) -> String {
        self.0
            .iter()
            .fold(body, |body, filter| filter.filter(config, body))
    }
}

pub trait ResponseFilterBuilder {
    type FILTER: 'static + ResponseFilter;

    fn try_build(self) -> ::anyhow::Result<<Self as ResponseFilterBuilder>::FILTER>;
}

impl<T, const N: usize> ResponseFilterBuilder for [T; N]
where
    T: ResponseFilterBuilder,
{
    type FILTER = ResponseFilters;

    fn try_build(self) -> ::anyhow::Result<<Self as ResponseFilterBuilder>::FILTER> {
        self.into_iter()
            .map(|builder| {
                ResponseFilterBuilder::try_build(builder)
                    .map(|filter| Box::new(filter) as Box<dyn ResponseFilter>)
            })
            .collect::<::anyhow::Result<_>>()
            .map(ResponseFilters)
    }
}

pub trait ResponseFilter
where
    Self: Send + Sync,
{
    fn filter(&self, config: &crate::config::ConfigMap, body: String) -> String;
}
