use std::collections::HashMap;
use std::sync;

pub struct Template {
    pub name: &'static str,
    pub values: HashMap<&'static str, String>,
}

impl Template {
    pub fn render(self, registry: sync::Arc<handlebars::Handlebars<'_>>) -> String {
        registry
            .render(self.name, &self.values)
            .unwrap_or_else(|err| err.to_string())
    }
}

pub fn new_registry() -> handlebars::Handlebars<'static> {
    let mut registry = handlebars::Handlebars::new();
    let found_file_template = "<!DOCTYPE html>
                    <html>
                    <head>
                        <title>Found file</title>
                    </head>
                    <body>
                        <h1>Found file</h1>
                        <h2>Workspace Logs:</h2>
                        <pre>{{logs}}</pre>
                        <h2>Workspace Query Results:</h2>
                        <pre>{{workspace_query_result}}</pre>
                    </body>
                    </html>
                    ";
    registry
        .register_template_string("found_file", found_file_template)
        .unwrap();
    let found_directory_template = "<!DOCTYPE html>
        <html>
        <head>
            <title>Found Directory</title>
        </head>
        <body>
            <h1>Found directory</h1>
            <h2>Workspace Logs:</h2>
            <pre>{{logs}}</pre>
            <h2>Workspace Query Results:</h2>
            <pre>{{workspace_query_result}}</pre>
        </body>
        </html>
        ";
    registry
        .register_template_string("found_directory", found_directory_template)
        .unwrap();
    let error_template = "<!DOCTYPE html>
        <html>
        <head>
            <title>Error</title>
        </head>
        <body>
            <h1>Error</h1>
            <h2>Workspace Logs:</h2>
            <pre>{{logs}}</pre>
            <h2>Error:</h2>
            <pre>{{error}}</pre>
        </body>
        </html>
        ";
    registry
        .register_template_string("error", error_template)
        .unwrap();

    registry
}
