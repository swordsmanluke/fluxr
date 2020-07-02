use std::cmp::{Ordering, min};

mod linear_layout;
mod text_widget;

/***
Dim: Represents a constraint on layout.
    WrapContent -> Takes its size from the size of its children.
    Fixed(n)    -> Always 'n' characters, until the limits of the container or terminal get in the way.
    UpTo(n)     -> Resizes based on content between 0 and n characters.
 */
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Dim {
    WrapContent,
    Fixed(usize),
    UpTo(usize),
    // Between(usize, usize),
}

impl Dim {
    fn to_ord(&self) -> usize {
        match self {
            Dim::Fixed(x) => *x,
            Dim::UpTo(x) => 1000 + *x,
            Dim::WrapContent => 1_000_000_000,
        }
    }
}

impl PartialOrd for Dim {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.to_ord().cmp(&other.to_ord()))
    }
}

impl Ord for Dim {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_ord().cmp(&other.to_ord())
    }
}
/***
View: A trait representing a render-able text widget.
 */
pub trait View {
    fn inflate(&mut self, parent_size: &Dimensions) -> Dimensions;
    fn constraints(&self) -> (Dim, Dim);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn render(&self) -> String;
    fn render_lines(&self) -> Vec<String>;
}

/***
TextWidget: A simple text container. Throw a String at it.
 */
pub struct TextWidget {
    task_id: String,
    raw_text: String,
    dirty: bool,
    dims: Dimensions,
    formatter: Box<dyn TextFormatter>
}

/***
TextFormatter: A trait for classes that convert from a raw string into a formatted one.
    Generic in order to allow different Terminal backends to use their own custom
    String-variants.
 */
pub trait TextFormatter {
    fn format(&self, s: String, max_len: usize) -> String;
}

struct DumbFormatter{}

impl TextFormatter for DumbFormatter {
    fn format(&self, s: String, n: usize) -> String {
        s[0..n].to_string()
    }
}

/***
Orientation: For a LinearLayout. You know what this does.
 */
#[derive(Copy, Clone)]
pub enum Orientation {
    HORIZONTAL,
    VERTICAL
}

/***
LinearLayout: Prints child View widgets' contents, stacked horizontally or vertically.
 */
pub struct LinearLayout {
    orientation: Orientation,
    children: Vec<Box<dyn View>>,
    dims: Dimensions,
}

/***
Dimensions: An internal struct used to track the constraints and actual size of a View
 */
#[derive(Copy, Clone)]
struct Dimensions {
    width_constraint: Dim,
    height_constraint: Dim,
    width: usize,
    height: usize
}

impl Dimensions {
    pub fn new(width: Dim, height: Dim) -> Dimensions{
        Dimensions {
            width_constraint: width,
            height_constraint: height,
            width: 0,   // Will be calculated during 'inflate' later.
            height: 0
        }
    }
}


/***
Handy methods for layout calculations.
TODO: Move these? They don't really need to be visible to consumers of this crate, just the
      classes within.
 */
pub fn calc_view_size(constraint: &Dim, my_dim: &Dim) -> usize {
    let my_desired_size = desired_size(my_dim);
    match constraint {
        Dim::WrapContent => my_desired_size,
        _ => min(my_desired_size, desired_size(constraint))
    }
}

pub fn desired_size(constraint: &Dim) -> usize {
    match constraint {
        Dim::WrapContent => 0, // If the constraint at this point is wrap content, we have to inflate children to see
        Dim::Fixed(x) => *x,
        Dim::UpTo(x) => *x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dims_can_be_sorted() {
        assert!(Dim::Fixed(0) < Dim::Fixed(1));
        assert!(Dim::Fixed(1) < Dim::UpTo(1));
        assert!(Dim::Fixed(1000) < Dim::WrapContent);
    }
}