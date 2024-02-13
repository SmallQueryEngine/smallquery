use git2;
use rand::{self, Rng};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::{net, path};
use warp::Filter;

pub struct HTTPServer {
    addr: net::SocketAddr,
}

impl HTTPServer {
    pub fn new(addr: net::SocketAddr) -> Self {
        HTTPServer { addr }
    }
}

impl HTTPServer {
    pub async fn run(self) -> Result<(), std::io::Error> {
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
            .and(warp::path::param())
            .and(warp::path::end())
            .and(warp::query::<HashMap<String, String>>())
            .map(
                |workspace_name: String, query_params: HashMap<String, String>| {
                    let version = query_params
                        .get("version")
                        .map(String::to_owned)
                        .unwrap_or("latest".into());
                    let path = query_params
                        .get("name")
                        .map(String::to_owned)
                        .unwrap_or("".into());

                    // TODO: Parse version string.
                    // TODO: Parse path string.

                    struct WorkspaceName(String);
                    struct WorkspaceVersion(String);

                    enum FolderItem {
                        File {
                            name: String,
                            contents: String,
                        },
                        Directory {
                            name: String,
                            items: Vec<FolderItem>,
                        },
                    }

                    enum WorkspaceContent {
                        File { name: String, contents: String },
                        Folder { items: Vec<FolderItem> },
                    }

                    struct Data {
                        name: WorkspaceName,
                        version: WorkspaceVersion,
                        path: String,
                        content: WorkspaceContent,
                    }

                    // Checkout the workspace with the given version.
                    let workspaces_root = path::PathBuf::from("./workspaces").canonicalize().unwrap();
                    let workspace_dir = workspaces_root.join(&workspace_name);
                    let repo = git2::Repository::open(&workspace_dir).unwrap();

                    let tmp = std::env::temp_dir();
                    let random_string = rand::random::<u64>().to_string();
                    let mut random_key = [0u8; 64];
                    rand::thread_rng().fill(&mut random_key);
                    let workdir = tmp.join(random_string);
                    fs::create_dir_all(&workdir).unwrap();

                    let commit = if let Ok(reference) = repo.resolve_reference_from_short_name(&version) {
                        reference.peel_to_commit().unwrap()
                    } else if let Ok(commit) = repo.find_commit_by_prefix(&version) {
                        commit
                    } else {
                        return format!(
                            "Version {:?} does not exist in workspace {:?}",
                            version, workspace_name
                        );
                    };

                    println!("Version: {:?} -> Commit: {:?}", version, commit.id());

                    let mut checkout_builder = git2::build::CheckoutBuilder::new();
                    checkout_builder.target_dir(&workdir);
                    checkout_builder.recreate_missing(true);

                    if let Err(git_error) = repo.checkout_tree(&commit.into_object(), Some(&mut checkout_builder)) {
                        return match git_error.code() {
                            git2::ErrorCode::UnbornBranch => {
                                format!(
                                    "Version {:?} does not exist in workspace {:?}",
                                    version, workspace_name,
                                )
                            },
                            git2::ErrorCode::GenericError => {
                                panic!(
                                    "Generic Error when checking out workspace: code={:?} error={:?}",
                                    git_error.code(),
                                    git_error.message(),
                                )
                            }
                            _ => panic!("Error checking out workspace: code={:?} error={:?}", git_error.code(), git_error.message()), 
                        };
                    }
                    // Navigate to the given path.
                    // List all files in the given path, or return the file content if it's a file.

                    let mut items = vec![];
                    let path = workdir.join(&path);
                    if path.is_file() {
                        let contents = fs::read_to_string(&path).unwrap();
                        return format!("File: {:?} with contents: {:?}", path, contents);
                    }
                
                    println!("Path: {:?}", path);
                    for entry in walkdir::WalkDir::new(&path) {
                        let entry = entry.unwrap();
                        let path = entry.path().to_str().unwrap();
                        items.push(path.to_owned());
                    }

                    format!(
                        "Get files for workspace: {} with version: {:?} and path: {:?}\nItems: {:?}",
                        workspace_name, version, path, items,
                    )
                },
            );

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
