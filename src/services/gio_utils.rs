use crate::config::G_LOG_DOMAIN;
use gtk::gio;
use gtk::gio::prelude::DataInputStreamExt;

const TEMPLATE_PREFIX: &str = "/com/github/sixpounder/GameOfLife/templates/";

pub struct Template {}

impl Template {
    pub fn read_template(name: &str) -> Result<Vec<u8>, glib::Error> {
        let template_resource = format!("{}{}.univ", TEMPLATE_PREFIX, name.to_lowercase());
        glib::g_debug!(G_LOG_DOMAIN, "Reading template from {}", template_resource);
        match gio::resources_open_stream(template_resource.as_str(), gio::ResourceLookupFlags::NONE)
        {
            Ok(input_stream) => {
                let mut buffer = vec![];
                let data_stream = gio::DataInputStream::new(&input_stream);
                let mut bytes_read: usize = 0;
                while let Ok(byte) = data_stream.read_byte(gio::Cancellable::NONE) {
                    buffer.push(byte);
                    bytes_read += 1;
                }

                glib::g_debug!(G_LOG_DOMAIN, "Read {} bytes from template", bytes_read);
                Ok(buffer)
            }
            Err(err) => Err(err),
        }
    }
}


