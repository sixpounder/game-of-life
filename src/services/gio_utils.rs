use gtk::gio;
use gtk::gio::prelude::InputStreamExtManual;
use crate::config::G_LOG_DOMAIN;

const TEMPLATE_PREFIX: &str = "/com/github/sixpounder/GameOfLife/templates/";

pub struct Template {}

impl Template {
    pub fn read_template(name: &str) -> Result<Vec<u8>, glib::Error> {
        let template_resource = format!("{}{}.univ", TEMPLATE_PREFIX, name.to_lowercase());
        glib::g_debug!(G_LOG_DOMAIN, "Reading template from {}", template_resource);
        match gio::resources_open_stream(template_resource.as_str(), gio::ResourceLookupFlags::NONE) {
            Ok(input_stream) => {
                let mut buffer = vec![];
                let (bytes_read, error) = input_stream.read_all(&mut buffer, gio::Cancellable::NONE)?;
                if error.is_some() {
                    return Err(error.unwrap());
                } else {
                    glib::g_debug!(G_LOG_DOMAIN, "Read {} bytes from template", bytes_read);
                    Ok(buffer)
                }
            },
            Err(err) => Err(err)
        }
    }
}
