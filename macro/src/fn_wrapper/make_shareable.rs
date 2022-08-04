/*
 *   Copyright (c) 2022 R3BL LLC
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use quote::quote;
use syn::parse_macro_input;

use super::{gen_fn_input_args_expr_list,
            get_fn_input_args_ident_ref_from_fn_ty,
            get_fn_output_type_from,
            IdentRef};
use crate::fn_wrapper::custom_syntax_parser::{make_opt_where_clause_from_generic_args,
                                              SafeFnWrapperSyntaxInfo};

/// Example of using this macro:
///
/// ```no_run
/// make_shareable_fn_wrapper! {
///   named FnWrapper4<S, A>
///   containing fn_mut
///   of_type Fn(&S, &A) -> S
/// }
/// ```
///
/// Code equivalent to the following hand written code is generated by this
/// macro. ```no_run
/// use std::sync::Arc;
///
/// /// Reducer function signature. This is not [`Sized`].
/// pub type ReducerFn<S, A> = dyn Fn(&S, &A) -> S + Sync + Send + 'static;
///
/// /// [`ReducerFn`] has to be wrapped in an [`Arc`] because it is [`Sized`]
/// and safe to /// share between threads.
/// /// 1. It does not allow interior mutability.
/// /// 2. It is not thread safe, since it performs no locking.
/// pub struct ShareableReducerFn<S, A> {
///   fn_mut: Arc<ReducerFn<S, A>>,
/// }
///
/// impl<S, A> ShareableReducerFn<S, A>
/// where
///   S: Sync + Send + 'static,
///   A: Sync + Send + 'static,
/// {
///   /// Constructing a [`ShareableReducerFn`] using a sized argument `fn_mut`,
/// which can be a   /// normal function or a lambda.
///   pub fn from(fn_mut: impl Fn(&S, &A) -> S + Send + Sync + 'static) -> Self
/// {     Self {
///       fn_mut: Arc::new(fn_mut),
///     }
///   }
///
///   pub fn invoke(&self, state: &S, action: &A) -> S {
///     let fn_mut_ref = self.fn_mut.clone();
///     fn_mut_ref(state, action)
///   }
/// }
/// ```
pub fn fn_proc_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let safe_wrapper_syntax_info: SafeFnWrapperSyntaxInfo = parse_macro_input!(input);

  let SafeFnWrapperSyntaxInfo {
    wrapper_name_ident: _,
    wrapper_name_type,
    wrapper_name_type_generic_args,
    property_name_ident,
    property_fn_type,
  } = safe_wrapper_syntax_info;

  let fn_input_arg_type_vec: Vec<IdentRef> =
    get_fn_input_args_ident_ref_from_fn_ty(&property_fn_type);

  let (fn_input_arg_expr_vec, fn_input_arg_name_ident_vec) =
    gen_fn_input_args_expr_list(&fn_input_arg_type_vec);

  let fn_output_return_type = get_fn_output_type_from(&property_fn_type);

  let opt_generic_args = if wrapper_name_type_generic_args.is_some() {
    let args = wrapper_name_type_generic_args.as_ref().unwrap();
    quote! { < #args > }
  } else {
    quote! {}
  };

  let opt_where_clause = make_opt_where_clause_from_generic_args(wrapper_name_type_generic_args);

  quote! {
    // Type aliases to make the code more readable.
    type ARC<T> = std::sync::Arc<T>;
    type RWLOCK<T> = std::sync::RwLock<T>;
    type WRITE_G<'a, T> = std::sync::RwLockWriteGuard<'a, T>;
    type READ_G<'a, T> = std::sync::RwLockReadGuard<'a, T>;

    pub struct #wrapper_name_type {
      pub #property_name_ident: ARC<RWLOCK<dyn #property_fn_type + Send + Sync + 'static>>
    }

    impl #opt_generic_args #wrapper_name_type
    #opt_where_clause
    {
      pub fn from(
        #property_name_ident: impl #property_fn_type + Send + Sync + 'static
      ) -> Self {
        Self { #property_name_ident: ARC::new(RWLOCK::new(#property_name_ident)) }
      }

      pub fn get_ref(&self) -> ARC<RWLOCK<dyn #property_fn_type + Send + Sync + 'static>> {
        self.#property_name_ident.clone()
      }

      pub fn invoke(&self, #(#fn_input_arg_expr_vec),*) -> #fn_output_return_type {
        let arc_lock_fn_mut = self.get_ref();
        let mut fn_mut = arc_lock_fn_mut.write().unwrap();
        fn_mut(#(#fn_input_arg_name_ident_vec),*)
      }
    }

  }
  .into()
}
