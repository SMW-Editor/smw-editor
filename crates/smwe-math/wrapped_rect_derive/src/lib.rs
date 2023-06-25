use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(WrappedRect)]
pub fn wrapped_rect_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_wrapped_rect(&ast)
}

fn impl_wrapped_rect(ast: &syn::DeriveInput) -> TokenStream {
    let wrapper_name = &ast.ident;

    let gen = quote! {
        impl #wrapper_name<Rect> {
            #[inline(always)]
            pub const fn from_min_max(min: #wrapper_name<Pos2>, max: #wrapper_name<Pos2>) -> Self {
                Self(Rect::from_min_max(min.0, max.0))
            }

            #[inline(always)]
            pub fn from_min_size(min: #wrapper_name<Pos2>, size: #wrapper_name<Vec2>) -> Self {
                Self(Rect::from_min_size(min.0, size.0))
            }

            #[inline(always)]
            pub fn from_center_size(center: #wrapper_name<Pos2>, size: #wrapper_name<Vec2>) -> Self {
                Self(Rect::from_center_size(center.0, size.0))
            }

            #[inline(always)]
            pub fn from_x_y_ranges(x_range: impl Into<RangeInclusive<f32>>, y_range: impl Into<RangeInclusive<f32>>) -> Self {
                Self(Rect::from_x_y_ranges(x_range, y_range))
            }

            #[inline(always)]
            pub fn from_two_pos(a: #wrapper_name<Pos2>, b: #wrapper_name<Pos2>) -> Self {
                Self(Rect::from_two_pos(a.0, b.0))
            }

            pub fn from_points(points: &[#wrapper_name<Pos2>]) -> Self {
                // Can't call Rect::from_points using the slice of wrapped Pos2 without an extra allocation.
                let mut rect = Rect::NOTHING;
                for p in points {
                    rect.extend_with(p.0);
                }
                Self(rect)
            }

            #[inline]
            pub fn everything_right_of(left_x: f32) -> Self {
                Self(Rect::everything_right_of(left_x))
            }

            #[inline]
            pub fn everything_left_of(right_x: f32) -> Self {
                Self(Rect::everything_left_of(right_x))
            }

            #[inline]
            pub fn everything_below(top_y: f32) -> Self {
                Self(Rect::everything_below(top_y))
            }

            #[inline]
            pub fn everything_above(bottom_y: f32) -> Self {
                Self(Rect::everything_above(bottom_y))
            }

            #[must_use]
            pub fn expand(self, amnt: f32) -> Self {
                Self(self.0.expand(amnt))
            }

            #[must_use]
            pub fn expand2(self, amnt: #wrapper_name<Vec2>) -> Self {
                Self(self.0.expand2(amnt.0))
            }

            #[must_use]
            pub fn shrink(self, amnt: f32) -> Self {
                Self(self.0.shrink(amnt))
            }

            #[must_use]
            pub fn shrink2(self, amnt: #wrapper_name<Vec2>) -> Self {
                Self(self.0.shrink2(amnt.0))
            }

            #[inline]
            #[must_use]
            pub fn translate(self, amnt: #wrapper_name<Vec2>) -> Self {
                Self(self.0.translate(amnt.0))
            }

            #[inline]
            #[must_use]
            pub fn rotate_bb(self, rot: Rot2) -> Self {
                Self(self.0.rotate_bb(rot))
            }

            #[inline]
            #[must_use]
            pub fn intersects(self, other: Self) -> bool {
                self.0.intersects(other.0)
            }

            #[inline(always)]
            pub fn set_center(&mut self, center: #wrapper_name<Pos2>) {
                self.0.set_center(center.0)
            }

            #[inline]
            #[must_use]
            pub fn contains(self, other: #wrapper_name<Pos2>) -> bool {
                self.0.contains(other.0)
            }

            #[inline]
            #[must_use]
            pub fn contains_rect(self, other: Self) -> bool {
                self.0.contains_rect(other.0)
            }

            #[must_use]
            pub fn clamp(self, p: #wrapper_name<Pos2>) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.clamp(p.0))
            }

            #[inline(always)]
            pub fn extend_with(&mut self, p: #wrapper_name<Pos2>) {
                self.0.extend_with(p.0)
            }

            #[inline(always)]
            #[must_use]
            pub fn union(self, other: Self) -> Self {
                Self(self.0.union(other.0))
            }

            #[inline(always)]
            #[must_use]
            pub fn intersect(self, other: Self) -> Self {
                Self(self.0.intersect(other.0))
            }

            #[inline(always)]
            pub fn center(self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.center())
            }

            #[inline(always)]
            pub fn size(self) -> #wrapper_name<Vec2> {
                #wrapper_name(self.0.size())
            }

            #[inline(always)]
            pub fn square_proportions(self) -> #wrapper_name<Vec2> {
                #wrapper_name(self.0.square_proportions())
            }

            #[inline]
            pub fn distance_to_pos(self, p: #wrapper_name<Pos2>) -> f32 {
                self.0.distance_to_pos(p.0)
            }

            #[inline]
            pub fn distance_sq_to_pos(self, p: #wrapper_name<Pos2>) -> f32 {
                self.0.distance_sq_to_pos(p.0)
            }

            #[inline]
            pub fn signed_distance_to_pos(self, p: #wrapper_name<Pos2>) -> f32 {
                self.0.signed_distance_to_pos(p.0)
            }

            pub fn lerp_inside(self, t: #wrapper_name<Vec2>) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.lerp_inside(t.0))
            }

            pub fn lerp_towards(self, other: &Self, t: f32) -> Self {
                Self(self.0.lerp_towards(&other.0, t))
            }

            #[inline(always)]
            pub fn left_top(&self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.left_top())
            }

            #[inline(always)]
            pub fn center_top(&self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.center_top())
            }

            #[inline(always)]
            pub fn right_top(&self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.right_top())
            }

            #[inline(always)]
            pub fn left_center(&self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.left_center())
            }

            #[inline(always)]
            pub fn right_center(&self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.right_center())
            }

            #[inline(always)]
            pub fn left_bottom(&self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.left_bottom())
            }

            #[inline(always)]
            pub fn center_bottom(&self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.center_bottom())
            }

            #[inline(always)]
            pub fn right_bottom(&self) -> #wrapper_name<Pos2> {
                #wrapper_name(self.0.right_bottom())
            }

            pub fn split_left_right_at_fraction(&self, t: f32) -> (Self, Self) {
                let (a, b) = self.0.split_left_right_at_fraction(t);
                (Self(a), Self(b))
            }

            pub fn split_left_right_at_x(&self, split_x: f32) -> (Self, Self) {
                let (a, b) = self.0.split_left_right_at_x(split_x);
                (Self(a), Self(b))
            }

            pub fn split_top_bottom_at_fraction(&self, t: f32) -> (Self, Self) {
                let (a, b) = self.0.split_top_bottom_at_fraction(t);
                (Self(a), Self(b))
            }

            pub fn split_top_bottom_at_y(&self, split_y: f32) -> (Self, Self) {
                let (a, b) = self.0.split_top_bottom_at_y(split_y);
                (Self(a), Self(b))
            }
        }
    };

    gen.into()
}
