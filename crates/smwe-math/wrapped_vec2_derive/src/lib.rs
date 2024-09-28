use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(WrappedVec2)]
pub fn wrapped_vec2_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_wrapped_vec2(&ast)
}

fn impl_wrapped_vec2(ast: &syn::DeriveInput) -> TokenStream {
    let wrapper_name = &ast.ident;

    let gen = quote! {
        impl #wrapper_name<emath::Vec2> {
            pub const ZERO: Self = Self::new(0., 0.);

            #[inline(always)]
            pub const fn new(x: f32, y: f32) -> Self {
                Self(emath::Vec2::new(x, y))
            }

            #[inline(always)]
            pub fn splat(v: f32) -> Self {
                Self(emath::Vec2::splat(v))
            }

            #[inline(always)]
            pub fn to_pos2(self) -> #wrapper_name<emath::Pos2> {
                #wrapper_name(self.0.to_pos2())
            }

            #[inline(always)]
            pub fn relative_to(self, other: Self) -> Self {
                Self(self.0 - other.0)
            }

            #[inline(always)]
            #[must_use]
            pub fn normalized(self) -> Self {
                Self(self.0.normalized())
            }

            #[inline(always)]
            pub fn rot90(self) -> Self {
                Self(self.0.rot90())
            }

            #[inline(always)]
            pub fn angled(angle: f32) -> Self {
                Self(emath::Vec2::angled(angle))
            }

            #[inline]
            pub fn dot(self, other: Self) -> f32 {
                self.0.dot(other.0)
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

        impl Neg for #wrapper_name<emath::Vec2> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(self.0.neg())
            }
        }

        impl Mul for #wrapper_name<emath::Vec2> {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0.mul(rhs.0))
            }
        }

        impl MulAssign<f32> for #wrapper_name<emath::Vec2> {
            fn mul_assign(&mut self, rhs: f32) {
                self.0.mul_assign(rhs);
            }
        }

        impl Mul<f32> for #wrapper_name<emath::Vec2> {
            type Output = Self;

            fn mul(self, rhs: f32) -> Self::Output {
                Self(self.0.mul(rhs))
            }
        }

        impl Mul<#wrapper_name<emath::Vec2>> for f32 {
            type Output = #wrapper_name<emath::Vec2>;

            fn mul(self, rhs: #wrapper_name<emath::Vec2>) -> Self::Output {
                #wrapper_name(self.mul(rhs.0))
            }
        }

        impl Div for #wrapper_name<emath::Vec2> {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0.div(rhs.0))
            }
        }

        impl Div<f32> for #wrapper_name<emath::Vec2> {
            type Output = Self;

            fn div(self, rhs: f32) -> Self::Output {
                Self(self.0.div(rhs))
            }
        }

        impl AddAssign<#wrapper_name<emath::Vec2>> for #wrapper_name<emath::Vec2> {
            fn add_assign(&mut self, rhs: #wrapper_name<emath::Vec2>) {
                self.0.add_assign(rhs.0);
            }
        }

        impl SubAssign<#wrapper_name<emath::Vec2>> for #wrapper_name<emath::Vec2> {
            fn sub_assign(&mut self, rhs: #wrapper_name<emath::Vec2>) {
                self.0.sub_assign(rhs.0);
            }
        }

        impl Add<#wrapper_name<emath::Vec2>> for #wrapper_name<emath::Vec2> {
            type Output = Self;

            fn add(self, rhs: #wrapper_name<emath::Vec2>) -> Self::Output {
                Self(self.0.add(rhs.0))
            }
        }

        impl Sub<#wrapper_name<emath::Vec2>> for #wrapper_name<emath::Vec2> {
            type Output = Self;

            fn sub(self, rhs: #wrapper_name<emath::Vec2>) -> Self::Output {
                Self(self.0.sub(rhs.0))
            }
        }
    };

    gen.into()
}
