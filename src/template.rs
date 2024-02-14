use std::collections::HashMap;
use std::str;

#[derive(rust_embed::RustEmbed)]
#[folder = "src/templates"]
struct Templates;

pub struct Template {
    pub path: &'static str,
    pub values: HashMap<&'static str, String>,
}

#[derive(Clone)]
pub struct Registry(handlebars::Handlebars<'static>);

static TEMPLATE_FILE_PATHS: [&str; 5] = [
    "page.hbs",
    "error.hbs",
    "found_file.hbs",
    "found_directory.hbs",
    "workspace_query_result.hbs",
];

impl Registry {
    /// Create a new instance of the template registry.
    /// Template files are loaded from embedded templates, defined as the Templates struct.
    pub fn new() -> Self {
        let mut handlebars_registry = handlebars::Handlebars::new();
        for path in TEMPLATE_FILE_PATHS {
            let contents = match Templates::get(path) {
                Some(file) => file.data,
                None => panic!("Expected template file {} but it does not exist.", path),
            };
            handlebars_registry
                .register_template_string(path, str::from_utf8(&contents).unwrap())
                .unwrap();
        }
        Registry(handlebars_registry)
    }

    pub fn render(&self, template: &Template) -> String {
        self.0
            .render(template.path, &template.values)
            .unwrap_or_else(|e| e.to_string())
    }
}
