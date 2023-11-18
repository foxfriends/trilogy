pub(crate) trait LabelMaker {
    fn make_label(&mut self, label: &str) -> String;
}

#[macro_export]
macro_rules! delegate_label_maker {
    ($t:ty, $f:ident) => {
        impl LabelMaker for $t {
            fn make_label(&mut self, label: &str) -> String {
                self.$f.make_label(label)
            }
        }
    };
}
