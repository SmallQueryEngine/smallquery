use serde_derive::Serialize;
use std::path;

#[derive(Debug)]
pub struct WorkspaceName(pub String);

impl WorkspaceName {
    pub fn new(name: String) -> Self {
        // TODO: Add dynamic validation for workspace name.
        WorkspaceName(name)
    }
}

impl AsRef<path::Path> for &WorkspaceName {
    fn as_ref(&self) -> &path::Path {
        self.0.as_ref()
    }
}

impl AsRef<str> for &WorkspaceName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct WorkspacePath(pub path::PathBuf);

impl WorkspacePath {
    pub fn new(path: path::PathBuf) -> Self {
        let mut sanitized_components = Vec::new();
        for component in path.components() {
            match component {
                path::Component::CurDir | path::Component::ParentDir => (),
                component => sanitized_components.push(component),
            }
        }
        if sanitized_components.is_empty() {
            sanitized_components.push(path::Component::RootDir);
        }
        let mut path = path::PathBuf::from_iter(sanitized_components);
        if path.starts_with("/") {
            path = path.strip_prefix("/").unwrap().to_path_buf();
        }
        WorkspacePath(path)
    }

    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap()
    }
}

impl AsRef<path::Path> for &WorkspacePath {
    fn as_ref(&self) -> &path::Path {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct WorkspaceVersion(pub String);

impl WorkspaceVersion {
    pub fn new(version: String) -> Self {
        WorkspaceVersion(version)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for &WorkspaceVersion {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Into<String> for &WorkspaceVersion {
    fn into(self) -> String {
        self.0.clone()
    }
}

pub struct WorkspaceQuery {
    pub workspace_name: WorkspaceName,
    pub workspace_path: WorkspacePath,
    pub workspace_version: WorkspaceVersion,
}

#[derive(Debug, Serialize)]
pub enum WorkspaceQueryResult {
    File { name: String, contents: String },
    Directory { name: String, items: Vec<String> },
}
