use crate::widgets::{LinearLayout, Orientation, View, Dim, Dimensions, desired_size, CharDims};
use std::cmp::{min, max};
use std::rc::Rc;
use std::cell::RefCell;
use log::info;
use regex::internal::Char;


impl LinearLayout {
    pub fn new(orientation: Orientation, width: Dim, height: Dim) -> LinearLayout {
        LinearLayout {
            orientation: orientation,
            dims: Dimensions::new(width, height),
            children: vec![],
        }
    }

    pub fn add_child(&mut self, child: Rc<RefCell<dyn View>>) {
        self.children.push(child);
    }

    fn render_vertical(&self) -> String {
        let lines: Vec<String> = self.children.iter().map(|c| c.borrow_mut().render()).collect();
        lines.iter().
            take(self.height()).
            map(|c| c.to_string()).
            collect::<Vec<String>>().
            join("\n")
    }

    fn render_horizontal(&self) -> String {
        let mut lines: Vec<String> = vec![];
        for _ in 0..self.height() {
            lines.push(String::from(""))
        }

        for c in &self.children {
            for i in 0..self.height() {
                let line = match c.borrow_mut().render_lines().get(i) {
                    None => String::from(""),
                    Some(l) => l.to_string()
                };

                lines[i] += line.as_str();
            }
        };

        lines.iter().
            map(|line| (&*format!("{:width$}", line, width = self.width())).to_string()).
            collect::<Vec<String>>().
            join("\n")
    }

    fn update_child_dims(orientation: Orientation, childrens_desired_dims: CharDims, child_dims: CharDims) -> CharDims {
        // Sum our children in the direction we are stacking them.
        // Capture the maximum in the direction we are stretching.
        // e.g. for Vertical, we stack by height, so sum those.
        //      ...then stretch sideways to the max child width.
        match orientation {
            Orientation::HORIZONTAL => {
                (childrens_desired_dims.0 + child_dims.0,
                 max(childrens_desired_dims.1, child_dims.1))
            }
            Orientation::VERTICAL => {
                (max(childrens_desired_dims.0, child_dims.0),
                 childrens_desired_dims.1 + child_dims.1)
            }
        }
    }

    fn update_parent_dims(orientation: Orientation, remaining_parent_dims: CharDims, child_dims: CharDims) -> CharDims {
        // Subtract remaining size in the direction we are stacking children.
        // Ignore in the direction we are stretching.
        // e.g. for Vertical, we stack by height, so subtract each child from that.
        match orientation {
            Orientation::VERTICAL => {
                (remaining_parent_dims.0,
                 if remaining_parent_dims.1 >= child_dims.1 { remaining_parent_dims.1 - child_dims.1 } else { 0 })
            }
            Orientation::HORIZONTAL => {
                (if remaining_parent_dims.0 >= child_dims.0 { remaining_parent_dims.0 - child_dims.0 } else { 0 },
                 remaining_parent_dims.1)
            }
        }
    }
}

impl View for LinearLayout {
    fn inflate(&mut self, parent_dimensions: &CharDims) -> CharDims {
        let mut childrens_desired_dims = (0, 0);
        let most_restrictive_width = min(self.dims.width_constraint, Dim::Fixed(parent_dimensions.0));
        let most_restrictive_height = min(self.dims.height_constraint, Dim::Fixed(parent_dimensions.1));

        self.dims.size = (desired_size(&most_restrictive_width),
                          desired_size(&most_restrictive_height));

        let mut remaining_parent_dims = self.dims.size.clone();

        for v in &mut self.children {
            let child_dims = v.borrow_mut().inflate(&remaining_parent_dims);
            childrens_desired_dims = LinearLayout::update_child_dims(self.orientation, childrens_desired_dims, child_dims);
            remaining_parent_dims = LinearLayout::update_parent_dims(self.orientation, remaining_parent_dims, child_dims);
        }

        match self.dims.width_constraint {
            Dim::WrapContent => {
                self.dims.size.0 = childrens_desired_dims.0;
            }
            _ => {}
        };

        match self.dims.height_constraint {
            Dim::WrapContent => {
                self.dims.size.1 = childrens_desired_dims.1;
            }
            _ => {}
        };

        if self.height() == 0 {
            info!("LL {:?} Dimensions: {}x{}", self.orientation, self.width(), self.height());
            info!("LL zero height child dims height: {}; constraint: {:?}", childrens_desired_dims.1, self.dims.height_constraint);
        }

        self.dims.size.clone()
    }

    fn constraints(&self) -> (Dim, Dim) { (self.dims.width_constraint, self.dims.height_constraint) }

    fn width(&self) -> usize { self.dims.size.0 }

    fn height(&self) -> usize { self.dims.size.1 }

    fn render(&self) -> String {
        match self.orientation {
            Orientation::VERTICAL => self.render_vertical(),
            Orientation::HORIZONTAL => self.render_horizontal()
        }
    }

    fn render_lines(&self) -> Vec<String> {
        self.render().split("\n").map(|c| c.to_string()).collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::TextView;

    fn fixed_size_text_widget() -> TextView {
        let mut tw = TextView::new(Dim::Fixed(10), Dim::Fixed(2));
        tw.raw_text = String::from("This is some raw text\nwith multiple lines\nand then another line.");
        tw
    }

    fn wrap_content_text_widget() -> TextView {
        let mut tw = TextView::new(Dim::WrapContent, Dim::WrapContent);
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
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.inflate(&(100, 100));
        assert_eq!(10, ll.width());
        assert_eq!(2, ll.height());
    }

    #[test]
    fn when_vert_wrapping_content_takes_horz_size_from_largest_child() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.add_child(Rc::new(RefCell::new(wrap_content_text_widget())));
        ll.inflate(&(100, 100));
        assert_eq!(ll.width(), 22);
    }

    #[test]
    fn when_vert_wrapping_content_takes_vert_size_from_summed_children() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.inflate(&(100, 100));
        assert_eq!(ll.height(), 4);
    }

    #[test]
    fn when_horz_wrapping_content_takes_horz_size_from_summed_children() {
        let mut ll = horz_ll_with_wrap_content();
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.inflate(&(100, 100));
        assert_eq!(ll.width(), 20);
    }

    #[test]
    fn when_horz_wrapping_content_takes_vert_size_from_tallest_child() {
        let mut ll = horz_ll_with_wrap_content();
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.add_child(Rc::new(RefCell::new(wrap_content_text_widget())));
        ll.inflate(&(100, 100));
        assert_eq!(ll.height(), 3);
    }

    #[test]
    fn vert_rendering_works() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.inflate(&(100, 100));

        assert_eq!("This is so\nwith multi".to_string(), ll.render());
    }

    #[test]
    fn vert_rendering_works_with_multiple_children() {
        let mut ll = vert_ll_with_wrap_content();
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.inflate(&(100, 100));

        assert_eq!("This is so\nwith multi\nThis is so\nwith multi".to_string(), ll.render());
    }

    #[test]
    fn horz_rendering_works_with_multiple_children() {
        let mut ll = horz_ll_with_wrap_content();
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.add_child(Rc::new(RefCell::new(fixed_size_text_widget())));
        ll.inflate(&(100, 100));

        assert_eq!("This is soThis is soThis is so\nwith multiwith multiwith multi".to_string(), ll.render());
    }
}