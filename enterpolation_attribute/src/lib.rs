extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, ItemImpl, ImplItem, PathArguments, ImplItemMethod, Signature, Type, Path, ReturnType,
    parse_macro_input, parse_quote};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result, Error};
use syn::spanned::Spanned;
// use syn::fold::Fold;

enum OutputType {
    Result(Path),
    Source(Path),
}

struct Args {
    target: Path,
    error: Path,
    terminate: bool,
    result: Option<Path>,
}

impl Args {
    fn implication_handler(&self, mut input: ItemImpl) -> Result<TokenStream> {
        // we only allow path types in the signature
        match *input.self_ty {
            Type::Path(ref mut type_path) => {
                type_path.path = self.target.clone();
            },
            _ => {return Err(Error::new(input.span(), "expected path identifier"));}
        }
        // go through all item implication methods
        for method in input.items.iter_mut().filter_map(|item| match item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        }){
            self.function_handler(method)?;
        }
        Ok(quote!(#input).into())
    }
    //TODO: we have more assumptions! Return is either a result with inner type source or source itself!
    //TODO: otherwise our whole construct won't work!
    fn function_handler(&self, input: &mut ImplItemMethod) -> Result<()>{
        let target = &self.target;
        let method_name = &input.sig.ident;
        let method_input = &input.sig.inputs;
        //we want to change signature and block
        match self.returns_result(&mut input.sig.output)?{
            // we do not use the original output but the inner output of the result
            OutputType::Result(result_inner) => {
                let mut constructed = target.clone();
                // use generics of the return type
                match constructed.segments.last_mut() {
                    //todo: move (and not clone) should be fine here
                    Some(segment) => segment.arguments = result_inner.segments.last().unwrap().arguments.clone(),
                    None => {},//todo: return error?
                };
                input.sig.output = parse_quote!(
                    -> #constructed
                );
                input.block = parse_quote!(
                    #target {
                        inner: self.inner.and_then(|director| director.#method_name(#method_input)
                            .err_map(|err| err.into()))
                    }
                );
                Ok(())
            },
            // we do use the original output (changed to target) and wrap our body with an Ok()
            OutputType::Source(source) => {
                let mut constructed = target.clone();
                //use generics of return type
                match constructed.segments.last_mut() {
                    Some(segment) => segment.arguments = source.segments.last().unwrap().arguments.clone(),
                    None => {},//TODO: return error?
                }
                input.sig.output = parse_quote!(
                    -> #constructed
                );
                input.block = parse_quote!(
                    #target {
                        inner: self.inner.and_then(|director| Ok(director.#method_name(#method_input)))
                    }
                );
                Ok(())
            },
        }
    }
    /// Functions which checks if we have a return with result and if so returns the inner Ok type
    fn returns_result(&self, input: &mut ReturnType) -> Result<OutputType> {
        match *input {
            ReturnType::Default => Err(Error::new(input.span(), "Return Type must exist")),
            ReturnType::Type(_,ref mut boxed_type) => {
                match **boxed_type {
                    Type::Path(ref mut type_path) => {
                        // we have our path which we have to check
                        match self.result {
                            // we have to check if it is the same path as the given one
                            Some(ref path) => {
                                if type_path.path == *path {
                                    match type_path.path.segments.last() {
                                        Some(segment) => Ok(OutputType::Result(self.get_first_generic(&segment.arguments)?)),
                                        None => Err(Error::new(input.span(), "Return Type must exist")),   // this should be an error
                                    }
                                } else {
                                    Ok(OutputType::Source(type_path.path.clone()))
                                }
                            },
                            // we use default result, that is, just Result
                            None => match type_path.path.segments.last() {
                                Some(segment) => {
                                    if segment.ident == "Result" {
                                        Ok(OutputType::Result(self.get_first_generic(&segment.arguments)?))
                                    } else {
                                        Ok(OutputType::Source(type_path.path.clone()))
                                    }
                                },
                                None => Err(Error::new(input.span(), "Return Type must exist")),
                            }
                        }
                    },
                    _ => Err(Error::new(input.span(), "Return Type can only be `Result` or the director")),  //should be an error
                }
            },
        }
    }
    fn get_first_generic(&self, input: &PathArguments) -> Result<Path> {
        Err(Error::new(input.span(),"Not yet implemented"))
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<Path,Token![,]>::parse_terminated(input)?;
        if vars.len() < 2 {
            return Err(Error::new(vars.span(), "expected 2 identifiers"));
        }
        let mut terminate = false;
        let mut result = None;
        if vars.len() > 2 {
            if vars[2].is_ident("terminate"){
                terminate = true;
            }else{
                result = Some(vars[2].clone());
            }
        }
        if vars.len() > 3 && vars[3].is_ident("terminate"){
            terminate = true;
        }
        Ok(Args{
            target: vars[0].clone(),
            error: vars[1].clone(),
            terminate,
            result,
        })
    }
}


#[proc_macro_attribute]
pub fn chain_result(attr: TokenStream, original: TokenStream) -> TokenStream {
    //first we want to repeat everything in input
    let mut input = original.clone();
    let args = parse_macro_input!(attr as Args);
    // This ist not alway the case, at some point we would have to parse it as Item instead of ItemImpl
    let item = parse_macro_input!(original as ItemImpl);
    input.extend(args.implication_handler(item));
    input
}
