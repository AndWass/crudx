use quote::quote;
use syn::visit::Visit;

extern crate proc_macro;

trait FieldExt {
    fn has_attribute(&self, attribute: &str) -> bool;
    fn is_primary_key(&self) -> bool {
        self.has_attribute("primary_key")
    }

    fn is_read_only(&self) -> bool {
        self.has_attribute("read_only")
    }
}

impl FieldExt for syn::Field {
    fn has_attribute(&self, attribute: &str) -> bool {
        self.attrs
            .iter()
            .find(|x| x.tokens.is_empty() && x.path.is_ident(attribute))
            .is_some()
    }
}

#[derive(Debug)]
enum PrimaryKey {
    None,
    FromAttribute(syn::Field),
    FromName(syn::Field),
}

impl PrimaryKey {
    fn is_none(&self) -> bool {
        matches!(self, &PrimaryKey::None)
    }
}

#[derive(Debug)]
struct TableData {
    visibility: syn::Visibility,
    name: syn::Ident,
    primary_key: PrimaryKey,
    rw_fields: Vec<syn::Field>,
    ro_fields: Vec<syn::Field>,
}

impl TableData {
    fn add_field(&mut self, field: syn::Field) {
        if field.is_read_only() {
            self.ro_fields.push(field);
        } else {
            self.rw_fields.push(field);
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for TableData {
    fn visit_field(&mut self, i: &'ast syn::Field) {
        if let Some(ident) = &i.ident {
            if ident == "id" && self.primary_key.is_none() && !i.has_attribute("not_defaulted") {
                self.primary_key = PrimaryKey::FromName(i.clone());
            } else if i.is_primary_key() {
                let old =
                    std::mem::replace(&mut self.primary_key, PrimaryKey::FromAttribute(i.clone()));
                match old {
                    PrimaryKey::None => {}
                    PrimaryKey::FromAttribute(_) => {
                        panic!("Multiple primary key attributes not allowed")
                    }
                    PrimaryKey::FromName(old) => self.add_field(old),
                }
            } else {
                self.add_field(i.clone())
            }
        }
    }
}

fn new_table(data: &TableData) -> proc_macro::TokenStream {
    let name = quote::format_ident!("New{}", data.name);
    let visibility = data.visibility.clone();

    let fields: Vec<_> = data
        .rw_fields
        .iter()
        .map(|f| {
            let field = f.clone();
            quote! {
                #field
            }
        })
        .collect();
    
    
    
    let quoted = quote! {
        #visibility struct #name {
            #(#fields),*
        }
    };

    proc_macro::TokenStream::from(quoted)
}

#[proc_macro_derive(Table)]
pub fn derive_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();

    let mut table_data = TableData {
        visibility: input.vis.clone(),
        name: input.ident.clone(),
        primary_key: PrimaryKey::None,
        ro_fields: Vec::new(),
        rw_fields: Vec::new(),
    };

    table_data.visit_derive_input(&input);

    new_table(&table_data)
}

#[proc_macro_attribute]
pub fn primary_key(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}
