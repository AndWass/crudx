use syn::visit::Visit;

extern crate proc_macro;

enum PrimaryKey
{
    None,
    FromAttribute(syn::Field),
    FromName(syn::Field),
}

impl PrimaryKey {
    fn is_none(&self) -> bool {
        matches!(self, &PrimaryKey::None)
    }
}

struct TableData {
    visibility: syn::Visibility,
    name: syn::Ident,
    primary_key: PrimaryKey,
    fields: Vec<syn::Field>,
}

impl<'ast> syn::visit::Visit<'ast> for TableData {
    fn visit_field(&mut self, i: &'ast syn::Field) {
        if let Some(ident) = &i.ident {
            if ident == "id" && self.primary_key.is_none() {
                self.primary_key = PrimaryKey::FromName(i.clone());
            }
            else if i.attrs.iter().filter(|x| {
                x.tokens.is_empty() && x.path.is_ident("primary_key")
            }).count() > 0 {
                let old = std::mem::replace(&mut self.primary_key, PrimaryKey::FromAttribute(i.clone()));
                match old {
                    PrimaryKey::None => {},
                    PrimaryKey::FromAttribute(_) => panic!("Multiple primary key attributes not allowed"),
                    PrimaryKey::FromName(old) => self.fields.push(old),
                }
            }
            else {
                self.fields.push(i.clone());
            }
        }
    }
}


#[proc_macro_derive(Table)]
pub fn derive_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();
    
    let mut table_data = TableData {
        visibility: input.vis.clone(),
        name: input.ident.clone(),
        primary_key: PrimaryKey::None,
        fields: Vec::new(),
    };

    table_data.visit_derive_input(&input);

    proc_macro::TokenStream::new()
}
