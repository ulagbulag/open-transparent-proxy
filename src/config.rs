use std::collections::HashMap;

use anyhow::Result;

macro_rules! define_config {
    (
        @define_field $field:ident : $type:ty = env ( $env:stmt , $map: expr )
    ) => {
        let $field: Result<$type> = ::ark_core::env::infer({ $env });
    };
    (
        @define_field $field:ident : $type:ty = default ( $default_fn:stmt , $map: expr )
    ) => {
        let $field: Result<$type> = $field.or_else(|_| Ok({ $default_fn }));
    };
    (
        @define_field $field:ident : $type:ty = format ( $format:stmt , $map: expr )
    ) => {
        let $field: Result<$type> = ::strfmt::strfmt({ $format }, $map)
            .map_err(|error| ::anyhow::anyhow!(
                "failed to parse field ({name}): {error}",
                name = stringify!($field),
        ));
    };
    (
        $vis:vis struct $name:ident
        {
            $(
                #[
                    $( $define_kind:ident = $define_value:stmt ),*
                ]
                $field_vis:vis $field:ident: $type:ty ,
            )*
        }
    ) => {
        $vis struct $name {
            $(
                $field_vis $field: $type,
            )*
        }

        impl Config {
            pub fn try_default() -> Result<Self> {
                let mut map = ConfigMapInner::default();
                $(
                    $(
                        define_config!(
                            @define_field $field: $type = $define_kind ( $define_value, &map )
                        );
                    )*
                    let $field: $type = $field?;
                    map.insert(stringify!($field).into(), $field.to_string());
                )*

                Ok(Self {
                    $(
                        $field,
                    )*
                })
            }

            pub fn to_map(&self) -> ConfigMap {
                let mut map = ConfigMapInner::default();
                $(
                    map.insert(stringify!($field).into(), self.$field.clone());
                )*
                ConfigMap(map)
            }
        }
    };
}

define_config!(
    pub struct Config {
        /*
            Derived from Environment Variables
        */

        #[env = "BASE_URL", default = "/".to_string()]
        pub base_url: String,

        #[env = "PROXY_BASE_URL", default = base_url.clone()]
        pub proxy_base_url: String,

        #[env = "PROXY_HOST"]
        pub proxy_host: String,

        #[env = "PROXY_SCHEME", default = "https".to_string()]
        pub proxy_scheme: String,

        /*
            Automatically Formatted
        */

        #[format = "{proxy_host}{proxy_base_url}"]
        pub proxy_base_url_with_host: String,
    }
);

#[derive(Clone, Debug)]
pub struct ConfigMap(ConfigMapInner);

impl ::std::ops::Deref for ConfigMap {
    type Target = ConfigMapInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ::std::ops::DerefMut for ConfigMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

type ConfigMapInner = HashMap<String, String>;
