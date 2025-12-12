use crate::core::auth::ServerAuth;
use crate::core::error::Result;
use crate::remote::protocol::{CloneResponse, FetchResponse, PullResponse, PushResponse};
use crate::remote::git_compat;
use crate::core::repo::Repository;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, middleware, web};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// MUG server state
pub struct ServerState {
    /// Base directory for repositories
    pub repos_dir: PathBuf,
    /// Authentication manager
    pub auth: Arc<Mutex<ServerAuth>>,
}

/// Extract and validate token from request
fn extract_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
}

/// Push endpoint: POST /repo/{name}/push
async fn push_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<crate::remote::protocol::PushRequest>,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing authorization token"}));
        }
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "write") {
        Ok(true) => {}
        _ => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"error": "Permission denied"}));
        }
    }
    drop(auth);

    // Get or create repository
    let repo_path = state.repos_dir.join(&repo_name);
    let repo =
        match Repository::open(&repo_path) {
            Ok(r) => r,
            Err(_) => {
                // Try to initialize if doesn't exist
                match Repository::init(&repo_path) {
                    Ok(r) => r,
                    Err(e) => return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": format!("Failed to initialize repo: {}", e)}),
                    ),
                }
            }
        };

    // Process push: Store blobs, trees, and commits
    for blob in &body.blobs {
        if let Err(e) = repo.get_store().store_blob(&blob.content) {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to store blob: {}", e)}),
            );
        }
    }

    for tree in &body.trees {
        if let Err(e) = repo.get_store().store_tree(tree.entries.clone()) {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to store tree: {}", e)}),
            );
        }
    }

    // Store commits in database
    for commit in &body.commits {
        if let Ok(serialized) = serde_json::to_vec(commit) {
            if let Err(e) = repo.get_db().set("commits", &commit.id, &serialized) {
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": format!("Failed to store commit: {}", e)}),
                );
            }
        }
    }

    // Update branch reference
    if let Err(e) = repo.get_db().set("branches", body.branch.as_bytes(), &body.head.as_bytes()) {
        return HttpResponse::InternalServerError().json(
            serde_json::json!({"error": format!("Failed to update branch: {}", e)}),
        );
    }

    HttpResponse::Ok().json(PushResponse {
        success: true,
        message: "Push successful".to_string(),
        head: Some(body.head.clone()),
    })
}

/// Pull endpoint: POST /repo/{name}/pull
async fn pull_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<crate::remote::protocol::PullRequest>,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing authorization token"}));
        }
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "read") {
        Ok(true) => {}
        _ => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"error": "Permission denied"}));
        }
    }
    drop(auth);

    let repo_path = state.repos_dir.join(&repo_name);
    let repo = match Repository::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": format!("Repository not found: {}", e)}));
        }
    };

    // Gather commits, blobs, trees for the requested branch
    let branch_name = &body.branch;
    
    match gather_branch_objects(&repo, branch_name, &body.current_head) {
        Ok((commits, blobs, trees, head)) => {
            HttpResponse::Ok().json(PullResponse {
                success: true,
                commits,
                blobs,
                trees,
                head,
                message: "Pull successful".to_string(),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to gather objects: {}", e)}),
            )
        }
    }
}

/// Fetch endpoint: POST /repo/{name}/fetch
async fn fetch_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<crate::remote::protocol::FetchRequest>,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing authorization token"}));
        }
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "read") {
        Ok(true) => {}
        _ => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"error": "Permission denied"}));
        }
    }
    drop(auth);

    let repo_path = state.repos_dir.join(&repo_name);
    let repo = match Repository::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": format!("Repository not found: {}", e)}));
        }
    };

    // Gather branches and their heads
    match gather_all_branches(&repo, body.branch.as_deref()) {
        Ok(branches) => {
            HttpResponse::Ok().json(FetchResponse {
                success: true,
                branches,
                message: "Fetch successful".to_string(),
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to fetch branches: {}", e)}),
            )
        }
    }
}

/// Clone endpoint: POST /repo/{name}/clone
async fn clone_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
    _body: web::Json<crate::remote::protocol::CloneRequest>,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing authorization token"}));
        }
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "read") {
        Ok(true) => {}
        _ => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"error": "Permission denied"}));
        }
    }
    drop(auth);

    let repo_path = state.repos_dir.join(&repo_name);
    let repo = match Repository::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": format!("Repository not found: {}", e)}));
        }
    };

    // Gather all commits, blobs, trees, and branches for complete clone
    match gather_complete_repository(&repo) {
        Ok((commits, blobs, trees, branches, default_branch)) => {
            HttpResponse::Ok().json(CloneResponse {
                commits,
                blobs,
                trees,
                branches,
                default_branch,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to gather repository: {}", e)}),
            )
        }
    }
}

/// Migrate Git repository to MUG
async fn migrate_from_git(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing authorization token"}));
        }
    };

    // Verify write permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "write") {
        Ok(true) => {}
        _ => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"error": "Permission denied"}));
        }
    }
    drop(auth);

    // Get Git path from request
    let git_path = match body.get("git_path") {
        Some(serde_json::Value::String(p)) => p.clone(),
        _ => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Missing git_path in request"}));
        }
    };

    let mug_path = state.repos_dir.join(&repo_name);

    // Perform migration
    match git_compat::migrate_git_to_mug(&git_path, mug_path.to_str().unwrap_or("")) {
        Ok(message) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": message,
                "repo": repo_name
            }))
        }
        Err(e) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Migration failed: {}", e)
            }))
        }
    }
}

/// Health check
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

/// Start HTTP server
pub async fn run_server(repos_dir: PathBuf, host: &str, port: u16) -> Result<()> {
    let auth = Arc::new(Mutex::new(ServerAuth::new()));

    let state = web::Data::new(ServerState { repos_dir, auth });

    println!("Starting MUG HTTP server on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .route("/health", web::get().to(health))
            .route("/repo/{name}/push", web::post().to(push_handler))
            .route("/repo/{name}/pull", web::post().to(pull_handler))
            .route("/repo/{name}/fetch", web::post().to(fetch_handler))
            .route("/repo/{name}/clone", web::post().to(clone_handler))
            .route("/repo/{name}/list-branches", web::get().to(list_branches_handler))
            .route("/repo/{name}/info", web::get().to(repo_info_handler))
            .route("/repo/{name}/migrate-from-git", web::post().to(migrate_from_git))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;

    Ok(())
}

/// Gather all objects for a specific branch
fn gather_branch_objects(
    repo: &Repository,
    branch: &str,
    _current_head: &Option<String>,
) -> Result<(Vec<crate::core::commit::Commit>, Vec<crate::core::store::Blob>, Vec<crate::core::store::Tree>, String)> {
    // Get commits for branch
    let commits = repo.log()?
        .into_iter()
        .map(|log_line| {
            // Parse log line to extract commit info
            let parts: Vec<&str> = log_line.lines().collect();
            let id = parts.first().map(|s| s.to_string()).unwrap_or_default();
            crate::core::commit::Commit {
                id,
                tree_hash: String::new(),
                parent: None,
                author: String::new(),
                message: String::new(),
                timestamp: String::new(),
            }
        })
        .collect();
    
    // Gather blobs from repository
    // Full implementation would require iterating through .mug/objects directory
    // and deserializing blob objects. For now, return empty as placeholder.
    let blobs = Vec::new();
    
    // Gather trees from repository
    // Full implementation would require querying object store for tree objects
    // and deserializing them. For now, return empty as placeholder.
    let trees = Vec::new();
    
    let head = format!("refs/heads/{}", branch);

    Ok((commits, blobs, trees, head))
}

/// Gather all branches and their heads
fn gather_all_branches(
    repo: &Repository,
    specific_branch: Option<&str>,
) -> Result<std::collections::HashMap<String, String>> {
    let mut branches = std::collections::HashMap::new();

    // Fetch all branches from repository
    let all_branches = repo.branches()?;
    
    if let Some(filter) = specific_branch {
        // Return only the specific branch if requested
        if all_branches.contains(&filter.to_string()) {
            // Get the head commit for this branch
            let log = repo.log()?;
            let head = log.first()
                .and_then(|l| l.lines().next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "HEAD".to_string());
            branches.insert(filter.to_string(), head);
        }
    } else {
        // Return all branches with their heads
        let log = repo.log()?;
        let head = log.first()
            .and_then(|l| l.lines().next())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "HEAD".to_string());
        
        for branch in all_branches {
            branches.insert(branch, head.clone());
        }
    }

    Ok(branches)
}

/// Gather complete repository for clone
fn gather_complete_repository(
    repo: &Repository,
) -> Result<(
    Vec<crate::core::commit::Commit>,
    Vec<crate::core::store::Blob>,
    Vec<crate::core::store::Tree>,
    std::collections::HashMap<String, String>,
    String,
)> {
    // Fetch all commits, blobs, trees, and branches
    let log = repo.log()?;
    
    let head = log.first()
        .and_then(|l| l.lines().next())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "HEAD".to_string());
    
    let commits = log
        .into_iter()
        .map(|log_line| {
            let parts: Vec<&str> = log_line.lines().collect();
            let id = parts.first().map(|s| s.to_string()).unwrap_or_default();
            crate::core::commit::Commit {
                id,
                tree_hash: String::new(),
                parent: None,
                author: String::new(),
                message: String::new(),
                timestamp: String::new(),
            }
        })
        .collect();
    
    let blobs = Vec::new(); // Placeholder for blob gathering
    let trees = Vec::new(); // Placeholder for tree gathering
    
    // Get all branches
    let all_branches = repo.branches()?;
    let mut branches = std::collections::HashMap::new();
    
    for branch in all_branches {
        branches.insert(branch, head.clone());
    }
    
    // Get default branch
    let default_branch = repo.current_branch()?
        .unwrap_or_else(|| "main".to_string());

    Ok((commits, blobs, trees, branches, default_branch))
}

/// List all branches in repository
async fn list_branches_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing authorization token"}));
        }
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "read") {
        Ok(true) => {}
        _ => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"error": "Permission denied"}));
        }
    }
    drop(auth);

    let repo_path = state.repos_dir.join(&repo_name);
    match Repository::open(&repo_path) {
        Ok(repo) => {
            // Fetch actual branches from repo
            match repo.branches() {
                Ok(branches) => {
                    HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "branches": branches,
                        "message": "Listed branches"
                    }))
                }
                Err(e) => {
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": format!("Failed to list branches: {}", e)}),
                    )
                }
            }
        }
        Err(e) => {
            HttpResponse::NotFound().json(
                serde_json::json!({"error": format!("Repository not found: {}", e)}),
            )
        }
    }
}

/// Get repository information
async fn repo_info_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing authorization token"}));
        }
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "read") {
        Ok(true) => {}
        _ => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"error": "Permission denied"}));
        }
    }
    drop(auth);

    let repo_path = state.repos_dir.join(&repo_name);
    match Repository::open(&repo_path) {
        Ok(_repo) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "name": repo_name,
                "path": repo_path,
                "default_branch": "main",
                "message": "Repository information retrieved"
            }))
        }
        Err(e) => {
            HttpResponse::NotFound().json(
                serde_json::json!({"error": format!("Repository not found: {}", e)}),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token() {
        // Mock request would require more setup
        // This is a placeholder for actual tests
    }
}
