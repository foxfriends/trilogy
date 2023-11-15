pub(crate) trait LabelMaker {
    fn make_label(&mut self, label: &str) -> String;
}
