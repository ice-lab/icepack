use std::collections::HashMap;

///
/// ```js
/// config: [ "lib-a" ]
/// ```
///
/// ```js
/// config: [{
///     "lib-a": {
///         to: "lib-a/x",
///         import_type: "named",
///     },
/// }]
/// ```
#[derive(Debug)]
pub enum Config {
    LiteralConfig(String),
    SpecificConfig(SpecificConfigs),
}

#[derive(Debug)]
pub struct SpecificConfigs {
    pub(crate) name: String,
    pub(crate) map: HashMap<String, MapProperty>,
}

#[derive(Debug)]
pub struct MapProperty {
    pub(crate) to: String,
    pub(crate) import_type: Option<ImportType>,
    pub(crate) name: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum ImportType {
    Named,
    Default,
}
