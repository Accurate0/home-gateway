#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiDescription {
    pub namespaces: Vec<ApiNamespace>,
    pub globals: Vec<ApiField>,
    pub aliases: Vec<ApiAlias>,
    pub classes: Vec<ApiClass>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiNamespace {
    pub name: String,
    pub fields: Vec<ApiField>,
    pub functions: Vec<ApiFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiFunction {
    pub name: String,
    pub params: Vec<ApiField>,
    pub returns: Option<String>,
    pub scope: Option<String>,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiField {
    pub name: String,
    pub ty: String,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiAlias {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiClass {
    pub name: String,
    pub fields: Vec<ApiField>,
}

impl ApiField {
    pub fn declaration(&self) -> String {
        let marker = if self.optional { "?" } else { "" };

        format!("{}{marker}: {}", self.name, self.ty)
    }
}

impl ApiFunction {
    pub fn build_signature(namespace: &str, name: &str, params: &[ApiField]) -> String {
        let params = params
            .iter()
            .map(ApiField::declaration)
            .collect::<Vec<_>>()
            .join(", ");

        format!("{namespace}.{name}({params})")
    }
}
