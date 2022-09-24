use gettextrs::gettext;

pub fn i18n(format: &str) -> String {
    gettext(format)
}

