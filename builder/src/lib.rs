use proc_macro::TokenStream;
use syn::spanned::Spanned;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as syn::DeriveInput);
    match do_expand(&st) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

type StructFields = syn::punctuated::Punctuated<syn::Field,syn::Token!(,)>;

fn get_fields_from_derive_input(d: &syn::DeriveInput) -> syn::Result<&StructFields> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = d.data{
        return Ok(named)
    }
    Err(syn::Error::new_spanned(d, "Must define on a Struct, not Enum".to_string()))
}

fn generate_builder_struct_fields_def(fields: &StructFields) -> syn::Result<proc_macro2::TokenStream>{
    let idents:Vec<_> = fields.iter().map(|f| {&f.ident}).collect();
    let types:Vec<_> = fields.iter().map(|f| {&f.ty}).collect();

    let token_stream = quote!{
        #(#idents: std::option::Option<#types>),*
    };
    Ok(token_stream)
}

fn generate_builder_struct_factory_init_clauses(fields: &StructFields) -> syn::Result<Vec<proc_macro2::TokenStream>>{
    let init_clauses: Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        quote!{
            #ident: std::option::Option::None
        }
    }).collect();

    Ok(init_clauses)
}

fn generate_builder_setter_functions(fields: &StructFields) -> syn::Result<Vec<proc_macro2::TokenStream>>{
    let setters: Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote!{
            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                self.#ident = std::option::Option::Some(#ident);
                self
            }
        }
    }).collect();

    Ok(setters)
}

fn do_expand(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name_literal = st.ident.to_string();
    let builder_name_literal = format!("{}Builder", struct_name_literal);
    let builder_name_ident = syn::Ident::new(&builder_name_literal, st.span());

    let struct_ident = &st.ident;

    let fields = get_fields_from_derive_input(st)?;
    let builder_struct_fields_def = generate_builder_struct_fields_def(fields)?;
    let builder_struct_factory_init_clauses = generate_builder_struct_factory_init_clauses(fields)?;
    let builder_struct_setter_funcations: Vec<proc_macro2::TokenStream> = generate_builder_setter_functions(fields)?;

    let ret = quote! {
        pub struct #builder_name_ident {
            // 在当前这个`quote!`宏中，引用了其他`quote!`宏返回的结果
            // 在这里把不同的代码碎片拼接起来，就像搭积木一样
            #builder_struct_fields_def
        }
        impl #struct_ident {
            pub fn builder() -> #builder_name_ident {
                #builder_name_ident{
                    #(#builder_struct_factory_init_clauses),*
                }
            }
        }
        impl #builder_name_ident {
            #(#builder_struct_setter_funcations)*
        }
    };
    return Ok(ret);
}