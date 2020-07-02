use crate::widgets::{View, TextWidget, Dim, Dimensions, calc_view_size, DumbFormatter};
use std::iter::empty;
use std::cmp::min;

impl TextWidget {
    pub fn new(task_id: String, width: Dim, height: Dim) -> TextWidget {
        TextWidget {
            task_id,
            raw_text: "".to_string(),
            dirty: false,
            dims: Dimensions {
                width_constraint: width,
                height_constraint: height,
                width: 0,
                height: 0
            },
            formatter: Box::new(DumbFormatter{})
        }
    }
}

impl View for TextWidget {
    fn inflate(&mut self, parent_dimensions: &Dimensions) -> Dimensions {
        let most_restrictive_width = min(self.dims.width_constraint, parent_dimensions.width_constraint);
        let most_restrictive_height = min(self.dims.height_constraint, parent_dimensions.height_constraint);

        let desired_width_constraint = Dim::UpTo(self.raw_text.split("\n").map(|c| c.len()).max().unwrap_or(0));
        let desired_height_constraint  = Dim::UpTo(self.raw_text.split("\n").count());

        self.dims = Dimensions{
            width_constraint: most_restrictive_width,
            height_constraint: most_restrictive_height,
            width: calc_view_size(&most_restrictive_width, &desired_width_constraint),
            height: calc_view_size(&most_restrictive_height, &desired_height_constraint),
        };

        match self.dims.width_constraint {
            Dim::WrapContent => {
                self.dims.width = self.raw_text.split("\n").map(|s| s.len()).max().unwrap_or(0);
            }
            _ => {}
        };

        match self.dims.height_constraint {
            // TODO: This should be a little smarter and consider wrapped lines, but that's a "tomorrow" feature
            Dim::WrapContent => {
                self.dims.height = self.raw_text.split("\n").count();
            }
            _ => {}
        };

        self.dims.clone()
    }

    fn constraints(&self) -> (Dim, Dim) {
        (self.dims.width_constraint.clone(), self.dims.height_constraint.clone())
    }

    fn width(&self) -> usize { self.dims.width }

    fn height(&self) -> usize { self.dims.height }

    fn render(&self) -> String {
        self.raw_text.
            split("\n").take(self.height()). // First n Lines
            map(|c| self.formatter.format(c.to_string(), self.width())). // Format them
            collect::<Vec<String>>().join("\n")     // Convert back into a single string
    }

    fn render_lines(&self) -> Vec<String> {
        self.render()
            .split("\n")
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_size_text_widget() -> TextWidget {
        TextWidget::new("task".to_string(), Dim::Fixed(10), Dim::Fixed(2))
    }

    fn wrap_content_text_widget() -> TextWidget {
        TextWidget::new("task".to_string(), Dim::WrapContent, Dim::WrapContent)
    }

    #[test]
    fn retrieves_constraints() {
        assert_eq!(fixed_size_text_widget().constraints(), (Dim::Fixed(10), Dim::Fixed(2)));
    }

    #[test]
    fn inflation_of_fixed_width_works_with_wrap_content_parent() {
        let mut tw = fixed_size_text_widget();
        tw.raw_text = String::from("line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.");
        tw.inflate(&Dimensions::new(Dim::WrapContent, Dim::WrapContent));
        assert_eq!(10, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_fixed_width_works_shrinks_to_fit_parent() {
        let mut tw = fixed_size_text_widget();
        tw.raw_text = String::from("line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.");
        tw.inflate(&Dimensions::new(Dim::Fixed(5), Dim::WrapContent));
        assert_eq!(5, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_wrap_content_width_expands_to_line_length() {
        let mut tw = wrap_content_text_widget();
        tw.raw_text = String::from("line 1 is pretty long\nline 2 is shorter.");
        tw.inflate(&Dimensions::new(Dim::WrapContent, Dim::WrapContent));
        assert_eq!("line 1 is pretty long".len(), tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn inflation_of_wrap_content_width_shrinks_to_fixed_parent_dims() {
        let mut tw = wrap_content_text_widget();
        tw.raw_text = String::from("line 1 is pretty long\nline 2 is shorter.\nline 3 is also fairly long.");
        tw.inflate(&Dimensions::new(Dim::Fixed(3), Dim::Fixed(2)));
        assert_eq!(3, tw.width());
        assert_eq!(2, tw.height());
    }

    #[test]
    fn renders_all_text_within_wrap_content() {
        let mut tw = wrap_content_text_widget();
        tw.raw_text = String::from("some\ntext");
        tw.inflate(&Dimensions::new(Dim::WrapContent, Dim::WrapContent));
        assert_eq!(String::from("some\ntext"), tw.render());
    }

    #[test]
    fn renders_partial_text_within_fixed_size() {
        let mut tw = fixed_size_text_widget();
        tw.raw_text = String::from("some really long text\nand another really long line\nthis line doesn't show up at all");
        tw.inflate(&Dimensions::new(Dim::WrapContent, Dim::WrapContent));
        assert_eq!(String::from("some reall\nand anothe"), tw.render());
    }
}