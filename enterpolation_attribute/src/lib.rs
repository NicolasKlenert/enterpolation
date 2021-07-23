extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, ItemImpl, ImplItem, PathArguments, Block, ImplItemMethod, Type, Path, ReturnType,
    FnArg, Pat,
    parse_macro_input, parse_quote};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result, Error};
use syn::spanned::Spanned;
// use syn::fold::Fold;

enum OutputType {
    Result,
    Source,
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
    fn function_handler(&self, input: &mut ImplItemMethod) -> Result<()>{
        let target = &self.target;
        let method_name = &input.sig.ident;
        let method_input = &mut input.sig.inputs;
        let (args, mutable) = map_input(method_input);
        let closure_arg : Pat = match mutable {
            true => parse_quote!(mut director),
            false => parse_quote!(director),
        };
        //change method_input to be always self instead of &mut self
        match method_input.first_mut()
                        .expect("There should always be a self in the method to chain results") {
                FnArg::Receiver(ref mut rec) => {rec.mutability = None; rec.reference = None;},
                FnArg::Typed(_) => {return Err(Error::new(method_input.span(),"we are only able to chain results if it is a method!"))},
        }


        // find source and if we return a result
        let (output_type, source) = self.returns_result(&mut input.sig.output)?;
        let mut constructed = target.clone();
        // copy generics of source to our target
        match constructed.segments.last_mut() {
            Some(segment) => segment.arguments = source.segments.last().unwrap().arguments.clone(),
            None => {},//TODO: return error?
        }
        // change output to target (with the same generics)
        input.sig.output = parse_quote!(
            -> #constructed
        );
        // change block depending on what we return
        match output_type{
            // we do not use the original output but the inner output of the result
            // todo: one may check if result had the correct error such that we don't need to use into()
            OutputType::Result => {
                input.block = parse_quote!({
                    #target {
                        inner: self.inner.and_then(|#closure_arg| #source::#method_name(#args)
                            .err_map(|err| err.into()))
                    }
                });
            },

            // we do use the original output (changed to target) and wrap our body with an Ok()
            OutputType::Source => {
                input.block = parse_quote!({
                    #target {
                        inner: self.inner.and_then(|#closure_arg| Ok(#source::#method_name(#args)))
                    }
                });
            },
        }
        Ok(())
    }
    /// Functions which checks if we have a return with result and if so returns the inner Ok type
    fn returns_result(&self, input: &mut ReturnType) -> Result<(OutputType, Path)> {
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
                                        Some(segment) => Ok((OutputType::Result,self.get_first_generic(&segment.arguments)?)),
                                        None => Err(Error::new(input.span(), "Return Type must exist")),   // this should be an error
                                    }
                                } else {
                                    Ok((OutputType::Source,type_path.path.clone()))
                                }
                            },
                            // we use default result, that is, just Result
                            None => match type_path.path.segments.last() {
                                Some(segment) => {
                                    if segment.ident == "Result" {
                                        Ok((OutputType::Result,self.get_first_generic(&segment.arguments)?))
                                    } else {
                                        Ok((OutputType::Source,type_path.path.clone()))
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

fn map_input(input: &Punctuated<FnArg,Token![,]>) -> (Punctuated<Pat, Token![,]>, bool) {
    let mut mutable = true;
    let map = input.iter().map(|arg| {
        match arg {
            //todo: director may be & or &mut such check this and add this to the parser!
            FnArg::Receiver(rec) => {
                //TODO: there should always be self in the first slot and it should always be mutable
                let pat : Pat = match rec.reference {
                    Some(ref and) => match and.1 {
                        Some(ref lifetime) => parse_quote!(&#lifetime mut director),
                        None => parse_quote!(&mut director),
                    },
                    None => {
                        mutable = false;
                        return parse_quote!(director);
                    },
                };
                pat
            },
            FnArg::Typed(pat_type) => (*pat_type.pat).clone(),
        }
    }).collect();
    (map, mutable)
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
    let item = args.implication_handler(item).unwrap();
    println!("changed block: {}",item);
    input.extend(item);
    input
}
