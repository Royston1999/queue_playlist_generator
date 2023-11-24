#[macro_export]
macro_rules! lock {
    ($x:expr) => {{
        $x.lock().unwrap()
    }};
}

#[macro_export]
macro_rules! text_edit {
    ($name:expr, $hint:expr, $value:expr, $ui:expr, $line:ident) => {{
        $ui.horizontal(|ui| {
            let name_label = ui.label($name);
            egui::TextEdit::$line(&mut $value).hint_text($hint).show(ui).response.labelled_by(name_label.id);
        });
    }};
    ($name:expr, $hint:expr, $value:expr, $ui:expr) => {{ text_edit!($name, $hint, $value, $ui, singleline) }};
}