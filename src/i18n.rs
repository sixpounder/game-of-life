use gettextrs::gettext;

pub fn i18n(format: &str) -> String {
    gettext(format)
}

pub fn translators_list() -> Vec<&'static str> {
    vec![
        "Rene Coty (French)"
    ]
}

