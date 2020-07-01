

use crate::widgets::{LinearLayout, Orientation, View, Dim, Dimensions, calc_view_size};
use std::cmp::{min, max};

impl LinearLayout {
    pub fn new(orientation: Orientation, width: Dim, height: Dim) -> LinearLayout {
        LinearLayout {
            orientation: orientation,
            dims: Dimensions::new(width, height),
            children: vec![],
        }
    }

    pub fn add_child(&mut self, child: Box<dyn View>) {
        self.children.push(child);
    }

    fn render_vertical(&self) -> String {
        let lines: Vec<String> = self.children.iter().map( |c| c.render()).collect();
        lines.join("\n")
    }


    fn render_horizontal(&self) -> String {
        unimplemented!()
    }
}

impl View for LinearLayout {
    fn inflate(&mut self, parent_dimensions: &Dimensions) -> Dimensions {
        let mut max_child_dims = (0, 0);
        let most_restrictive_width = min(self.dims.width_constraint, parent_dimensions.width_constraint);
        let most_restrictive_height = min(self.dims.height_constraint, parent_dimensions.height_constraint);

        self.dims = Dimensions{
            width_constraint: most_restrictive_width,
            height_constraint: most_restrictive_height,
            width: calc_view_size(&most_restrictive_width, &self.dims.width_constraint),
            height: calc_view_size(&most_restrictive_width, &self.dims.width_constraint),
        };

        for mut v in &mut self.children {
            let new_dims = v.inflate(&self.dims);
            max_child_dims.0 = max(max_child_dims.0, new_dims.width);
            max_child_dims.1 = max(max_child_dims.1, new_dims.height);
        }

        match self.dims.width_constraint {
            Dim::WrapContent => {
                self.dims.width = max_child_dims.0;
            }
            _ => {}
        };

        match self.dims.height_constraint {
            Dim::WrapContent => {
                self.dims.height = max_child_dims.1;
            }
            _ => {}
        };

        self.dims.clone()
    }

    fn constraints(&self) -> (Dim, Dim) { (self.dims.width_constraint, self.dims.height_constraint) }

    fn width(&self) -> usize { self.dims.width }

    fn height(&self) -> usize {
        self.dims.height
    }

    fn render(&self) -> String {
        match self.orientation {
            Orientation::VERTICAL => self.render_vertical(),
            Orientation::HORIZONTAL => self.render_horizontal()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::TextWidget;

    fn fixed_size_text_widget() -> TextWidget {
        let mut tw = TextWidget::new("task".to_string(), Dim::Fixed(10), Dim::Fixed(2));
        tw.raw_text = String::from("This is some raw text\nwith multiple lines\nand then another line.");
        tw
    }

    fn wrap_content_text_widget() -> TextWidget {
        let mut tw = TextWidget::new("task".to_string(), Dim::WrapContent, Dim::WrapContent);
        tw.raw_text = String::from("This is some raw text\nwith multiple lines\nand then another line.");
        tw
    }

    fn vert_ll_with_wrap_content() -> LinearLayout {
        LinearLayout::new(Orientation::VERTICAL, Dim::WrapContent, Dim::WrapContent)
    }

    fn vert_ll_with_fixed_size() -> LinearLayout {
        LinearLayout::new(Orientation::VERTICAL, Dim::Fixed(5), Dim::Fixed(2))
    }

    fn horz_ll_with_wrap_content() -> LinearLayout {
        LinearLayout::new(Orientation::HORIZONTAL, Dim::WrapContent, Dim::WrapContent)
    }

    #[test]
    fn retrieves_constraints() {
        assert_eq!(vert_ll_with_fixed_size().constraints(), (Dim::Fixed(5), Dim::Fixed(2)));
    }

    #[test]
    fn when_wrapping_content_takes_size_from_children() {
        let mut tw = fixed_size_text_widget();
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Box::new(tw));
        ll.inflate(&Dimensions::new(Dim::WrapContent, Dim::WrapContent));
        assert_eq!(10, ll.width());
        assert_eq!(2, ll.height());
    }

    #[test]
    fn rendering_works() {
        let mut tw = fixed_size_text_widget();
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Box::new(tw));
        ll.inflate(&Dimensions::new(Dim::WrapContent, Dim::WrapContent));

        assert_eq!(ll.render(), "This \nwith ".to_string());
    }
}