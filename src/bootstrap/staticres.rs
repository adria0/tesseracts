#[derive(RustEmbed)]
#[folder = "tmpl"]
struct Asset;

use handlebars::Handlebars;

pub fn load_handlebars_templates(hb: &mut Handlebars) {
    // process assets
    for asset in Asset::iter() {
        let file = asset.into_owned();

        let tmpl = String::from_utf8(
            Asset::get(file.as_str())
                .unwrap_or_else(|| panic!("Unable to read file {}", file))
                .to_vec(),
        )
        .unwrap_or_else(|_| panic!("Unable to decode file {}", file));

        hb.register_template_string(file.as_str(), &tmpl)
            .unwrap_or_else(|_| panic!("Invalid template {}", file));
    }
} 