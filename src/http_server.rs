use git2;
use handlebars;
use rand::{self, Rng};
use std::collections::HashMap;
use std::fs;
use std::sync;
use std::{net, path};
use warp::Filter;
use serde_json::json;

pub struct HTTPServer {
    addr: net::SocketAddr,
}

impl HTTPServer {
    pub fn new(addr: net::SocketAddr) -> Self {
        HTTPServer { addr }
    }
}

struct WithTemplate<T: serde::Serialize> {
    name: &'static str,
    value: T,
}

fn render<T>(template: WithTemplate<T>, hbs: sync::Arc<handlebars::Handlebars<'_>>) -> impl warp::Reply
where
    T: serde::Serialize,
{
    let render = hbs
        .render(template.name, &template.value)
        .unwrap_or_else(|err| err.to_string());
    warp::reply::html(render)
}

impl HTTPServer {
    pub async fn run(self) -> Result<(), std::io::Error> {
        // TODO: Create a templating singleton that can render an impl warp::Reply.
        // The template inputs are:
        //  1. name
        //  2. key-value pairs
        // The variants are:
        //  1. Found file
        //  2. Found directory
        //  3. Error
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

        let mut hb = handlebars::Handlebars::new();
        hb.register_template_string("found_file", found_file_template)
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
        hb.register_template_string("found_directory", found_directory_template);
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
        hb.register_template_string("error", error_template);
        let hb = sync::Arc::new(hb);
        let handlebars =  move |with_template| render(with_template, hb.clone());

        // GET => / => "Hello, World!"
        let root = warp::get().and(warp::path::end()).map(|| "Hello, World!");
        // GET => /health => "Healthy!"
        let health = warp::get().and(warp::path("health")).map(|| "Healthy!");
        // GET => /workspaces => List all workspaces
        let list_workspaces = warp::get()
            .and(warp::path("workspaces"))
            .and(warp::path::end())
            .map(|| "List all workspaces");
        // GET => /workspaces/:workspace_name?version=<ref>&path=<path> => GET => Get files for a workspace.
        let detail_workspace = warp::get()
            .and(warp::path("workspaces"))
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .and(warp::query::<HashMap<String, String>>())
            .map(
                |input_workspace_name: String, query_params: HashMap<String, String>| {
                    let input_version_default = "latest".to_string();
                    let input_version = query_params
                        .get("version")
                        .map(String::to_owned)
                        .unwrap_or(input_version_default);
                    let input_path_default = "".to_string();
                    let input_path = query_params
                        .get("path")
                        .map(String::to_owned)
                        .unwrap_or(input_path_default);

                    let workspace_query = crate::core::WorkspaceQuery{
                        workspace_name: crate::core::WorkspaceName::new(input_workspace_name.clone()),
                        workspace_path: crate::core::WorkspacePath::new(path::PathBuf::from(&input_path)),
                        workspace_version: crate::core::WorkspaceVersion::new(input_version.clone()),
                    };

                    // Filesystem Adapter Code
                    // Create a temporary directory to checkout the workspace.
                    let tmp = std::env::temp_dir(); // TODO: Make this configurable.
                    let random_string = rand::random::<u64>().to_string();
                    let mut random_key = [0u8; 64];
                    rand::thread_rng().fill(&mut random_key);
                    let workdir_mount = tmp.join(&random_string);
                    fs::create_dir_all(&workdir_mount).unwrap();
                    // TODO: Cleanup the temporary directory after the request is complete.
                    // End Filesystem Adapter Code

                    let name = &workspace_query.workspace_name;
                    let version = &workspace_query.workspace_version;

                    // Git Adapter Code
                    let workspaces_mount_default = path::PathBuf::from("workspaces"); // TODO: Make this configurable
                    let workspaces_mount = workspaces_mount_default.canonicalize().unwrap();
                    let workspace_mount = workspaces_mount.join(name);
                    let repo = match git2::Repository::open(&workspace_mount) {
                        Ok(repo) => repo,
                        Err(git_error) => return WithTemplate{
                            name: "error",
                            value: json!({"error": format!("Error when opening workspace: code={:?} error={:?}", git_error.code(), git_error.message())}),
                        },
                    };

                    // Look-up and checkout the workspace version to the temporary directory.
                    let commit = if let Ok(reference) = repo.resolve_reference_from_short_name(version.as_str()) {
                        reference.peel_to_commit().unwrap()
                    } else if let Ok(commit) = repo.find_commit_by_prefix(version.as_str()) {
                        commit
                    } else {
                        return WithTemplate{
                            name: "error",
                            value: json!({"error": format!("version {:?} does not exist in workspace {:?}", version, name)}),
                        };
                    };

                    println!("Version: {:?} -> Commit: {:?}", workspace_query.workspace_version, commit.id());

                    let mut checkout_builder = git2::build::CheckoutBuilder::new();
                    checkout_builder.target_dir(&workdir_mount);
                    checkout_builder.recreate_missing(true);

                    if let Err(git_error) = repo.checkout_tree(&commit.into_object(), Some(&mut checkout_builder)) {
                        match git_error.code() {
                            git2::ErrorCode::UnbornBranch => {
                                return WithTemplate{
                                    name: "error",
                                    value: json!({"error": format!("version {:?} does not exist in workspace {:?}", version, name)}),
                                };
                            },
                            git2::ErrorCode::GenericError => {
                                return WithTemplate{
                                    name: "error",
                                    value: json!({"error": format!("Generic Error when checking out workspace: code={:?} error={:?}", git_error.code(), git_error.message())}),
                                };
                            },
                            _ => {
                                return WithTemplate{
                                name: "error",
                                value: json!({"error": format!("Unexpected Error when checking out workspace: code={:?} error={:?}", git_error.code(), git_error.message())}),
                                }; 
                            },
                        };
                    }
                    // End Git Adapter Code

                    let path = &workspace_query.workspace_path;
                    let workdir_path_mount = workdir_mount.join(path.as_ref());

                    let logs = format!("
                        -- Input --
                        Workspace Name: {:?}
                        Workspace Version: {:?}
                        Workspace Path: {:?}
                        -- Sanitized --
                        Workspace Name: {:?}
                        Workspace Path: {:?}
                        Workspace Version: {:?}
                        -- Configured --
                        Workspaces Mount: {:?}
                        Workspace Mount: {:?}
                        -- Computed --
                        WorkDir Mount: {:?}
                        WorkDir Path Mount: {:?}
                        ",
                        input_workspace_name,
                        input_version,
                        input_path,
                        workspace_query.workspace_name,
                        workspace_query.workspace_path,
                        workspace_query.workspace_version,
                        workspaces_mount,
                        workspace_mount,
                        workdir_mount,
                        workdir_path_mount,
                    );

                    println!("{}", logs);

                    // Filesystem Adapter Code
                    // In the checkout, list all items in the workspace path, or return the file content if it's a file.
                    if !path::Path::new(&workdir_path_mount).exists() {
                        return WithTemplate{
                            name: "error",
                            value: json!({"error": format!("path {:?} does not exist in workspace {:?}", path, name)}),
                        };
                    }

                    let workspace_query_result = if workdir_path_mount.is_file() {
                        crate::core::WorkspaceQueryResult::File{
                            name: workspace_query.workspace_path.as_str().to_string(),
                            contents: fs::read_to_string(&workdir_path_mount).unwrap(),
                        }
                    } else {
                        let mut items = vec![];
                        for entry in walkdir::WalkDir::new(&workdir_path_mount) {
                            let entry = entry.unwrap();
                            let path = entry.path().to_str().unwrap();
                            items.push(path.to_owned());
                        }                
                        crate::core::WorkspaceQueryResult::Directory{
                            name: workspace_query.workspace_path.as_str().to_string(),
                            items: items,
                        }
                    };
                    // End Filesystem Adapter Code

                    match workspace_query_result {
                        crate::core::WorkspaceQueryResult::File{ name, contents } => {
                            WithTemplate{
                                name: "found_file",
                                value: json!({"logs": logs, "workspace_query_result": format!("name={:?} contents={:?}", name, contents)}),
                            }
                        },
                        crate::core::WorkspaceQueryResult::Directory{ name, items } => {
                            WithTemplate{
                                name: "found_directory",
                                value: json!({"logs": logs, "workspace_query_result": format!("name={:?} items={:?}", name, items)}),
                            }
                        },
                    }
                },
            ).map(handlebars);

        let routes = root.or(health).or(detail_workspace).or(list_workspaces);

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(self.addr, async {
            shutdown_rx.await.ok();
        });

        tokio::spawn(server);

        tokio::signal::ctrl_c().await.ok();
        shutdown_tx.send(()).ok();

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::HTTPServer;

    #[tokio::test]
    async fn test_run() {
        let addr = ([127, 0, 0, 1], 3030).into();
        let server = HTTPServer::new(addr);
        server.run().await.unwrap();
    }
}
