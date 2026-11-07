use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Prints the function's name each time the annotated function is called.
#[proc_macro_attribute]
pub fn log_call(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    let fn_name = input.sig.ident.clone();
    let original_block = input.block;

    input.block = Box::new(
        syn::parse_quote!({
            println!("Calling function: {}", stringify!(#fn_name));
            #original_block
        }),
    );

    quote!(#input).into()
}

/// Adds a greeting to the body of the annotated function while keeping its signature.
#[proc_macro_attribute]
pub fn test_fn1(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    let original_block = input.block;

    input.block = Box::new(syn::parse_quote!({
        println!("hello");
        #original_block
    }));

    quote!(#input).into()
}

fn add(one:u32,tow:u32)->u32{
    one + tow
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    // #[test]
    // fn dummy(){
    //     println!("calling the dummy fn inside the test");
    //     ok(true)
    // }
}
