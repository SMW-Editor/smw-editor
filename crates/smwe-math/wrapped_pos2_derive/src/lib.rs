use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(WrappedPos2)]
pub fn wrapped_pos2_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_wrapped_pos2(&ast)
}

fn impl_wrapped_pos2(ast: &syn::DeriveInput) -> TokenStream {
    let wrapper_name = &ast.ident;

    let gen = quote! {
        impl #wrapper_name<emath::Pos2> {
            pub const ZERO: Self = Self::new(0., 0.);

            #[inline(always)]
            pub const fn new(x: f32, y: f32) -> Self {
                Self(emath::Pos2::new(x, y))
            }

            #[inline(always)]
            pub fn to_vec2(self) -> #wrapper_name<emath::Vec2> {
                #wrapper_name(self.0.to_vec2())
            }

            #[inline(always)]
            pub fn relative_to(self, other: Self) -> Self {
                Self(self.0 - other.0.to_vec2())
            }

            #[inline]
            pub fn distance(self, other: Self) -> f32 {
                self.0.distance(other.0)
            }

            #[inline]
            pub fn distance_sq(self, other: Self) -> f32 {
                self.0.distance_sq(other.0)
            }

            pub fn lerp(self, other: Self, t: f32) -> Self {
                Self(self.0.lerp(other.0, t))
            }

            #[inline(always)]
            pub fn floor(self) -> Self {
                Self(self.0.floor())
            }

            #[inline(always)]
            pub fn round(self) -> Self {
                Self(self.0.round())
            }

            #[inline(always)]
            pub fn ceil(self) -> Self {
                Self(self.0.ceil())
            }

            #[inline]
            #[must_use]
            pub fn min(self, other: Self) -> Self {
                Self(self.0.min(other.0))
            }

            #[inline]
            #[must_use]
            pub fn max(self, other: Self) -> Self {
                Self(self.0.max(other.0))
            }

            #[inline]
            #[must_use]
            pub fn clamp(self, min: Self, max: Self) -> Self {
                Self(self.0.clamp(min.0, max.0))
            }
        }

        impl Sub for #wrapper_name<emath::Pos2> {
            type Output = #wrapper_name<emath::Vec2>;

            fn sub(self, rhs: Self) -> Self::Output {
                #wrapper_name(self.0.sub(rhs.0))
            }
        }

        impl AddAssign<#wrapper_name<emath::Vec2>> for #wrapper_name<emath::Pos2> {
            fn add_assign(&mut self, rhs: #wrapper_name<emath::Vec2>) {
                self.0.add_assign(rhs.0);
            }
        }

        impl SubAssign<#wrapper_name<emath::Vec2>> for #wrapper_name<emath::Pos2> {
            fn sub_assign(&mut self, rhs: #wrapper_name<emath::Vec2>) {
                self.0.sub_assign(rhs.0);
            }
        }

        impl Add<#wrapper_name<emath::Vec2>> for #wrapper_name<emath::Pos2> {
            type Output = Self;

            fn add(self, rhs: #wrapper_name<emath::Vec2>) -> Self::Output {
                Self(self.0.add(rhs.0))
            }
        }

        impl Sub<#wrapper_name<emath::Vec2>> for #wrapper_name<emath::Pos2> {
            type Output = Self;

            fn sub(self, rhs: #wrapper_name<emath::Vec2>) -> Self::Output {
                Self(self.0.sub(rhs.0))
            }
        }
    };

    gen.into()
}
