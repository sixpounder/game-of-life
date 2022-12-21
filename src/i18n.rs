use gettextrs::gettext;

pub fn i18n(format: &str) -> String {
    gettext(format)
}

pub fn translators_list() -> Vec<&'static str> {
    vec![
        "Andrea Coronese (English, Italian)",
        "Rene Coty (French)"
    ]
}

