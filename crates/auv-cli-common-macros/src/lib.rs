mod table_row;

use proc_macro::TokenStream;
use syn::{DeriveInput, Error, parse_macro_input};

#[proc_macro_derive(TableRow, attributes(table))]
pub fn derive_table_row(input: TokenStream) -> TokenStream {
  table_row::expand(parse_macro_input!(input as DeriveInput)).unwrap_or_else(Error::into_compile_error).into()
}
