extern crate proc_macro;

use syn::spanned::Spanned;
use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index};
use quote::{quote, quote_spanned};


#[ proc_macro_derive( Merge ) ]
//
pub fn derive_merge( input: TokenStream ) -> TokenStream
{
	let input = parse_macro_input!( input as DeriveInput );

	// Add a bound `T: Merge` to every type parameter T.
	//
	let generics = add_trait_bounds( input.generics );
	let ( impl_generics, ty_generics, where_clause ) = generics.split_for_impl();


	let name           = input.ident;
	let merge_children = merge_children( &input.data );

	let expanded = quote!
	{
		// The generated impl.
		//
		impl #impl_generics ::ekke_merge::Merge for #name #ty_generics #where_clause
		{
			fn merge( &mut self, other: Self ) -> MergeResult<()>
			{
				#merge_children
				Ok(())
			}
		}
	};

	proc_macro::TokenStream::from( expanded )
}


// Add a bound `T: Merge` to every type parameter T.
//
fn add_trait_bounds( mut generics: Generics ) -> Generics
{
	for param in &mut generics.params
	{
		if let GenericParam::Type(ref mut type_param) = *param
		{
			type_param.bounds.push( parse_quote!( ekke_merge::Merge ) );
		}
	}

	generics
}


// Generate an expression to call merge on each field.
//
fn merge_children( data: &Data ) -> proc_macro2::TokenStream
{
	match *data
	{
		Data::Struct( ref data ) =>
		{
			match data.fields
			{
				// Process named fields
				//
				Fields::Named( ref fields ) =>
				{
					let recurse = fields.named.iter().map( |f|
					{
						let name = &f.ident;

						quote_spanned! { f.span() => self.#name.merge( other.#name )?; }
					});

					quote! { #(#recurse)* }
				}


				Fields::Unnamed( ref fields ) =>
				{
					let recurse = fields.unnamed.iter().enumerate().map( |(i, f)|
					{
						let index = Index::from( i );

						quote_spanned! { f.span() => self.#index.merge( other.#index )?; }
					});

					quote! { #(#recurse)* }
				}

				Fields::Unit =>
				{
					// Unit structs cannot merge any data.
					quote!()
				}
			}
		}

		Data::Enum(_) | Data::Union(_) => unimplemented!(),
	}
}



