use crate::core::auth::ServerAuth;
use crate::core::error::Result;
use crate::remote::protocol::{CloneResponse, FetchResponse, PullResponse, PushResponse};
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
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;

    Ok(())
}

/// Gather all objects for a specific branch
fn gather_branch_objects(
    _repo: &Repository,
    branch: &str,
    _current_head: &Option<String>,
) -> Result<(Vec<crate::core::commit::Commit>, Vec<crate::core::store::Blob>, Vec<crate::core::store::Tree>, String)> {
    // Get commits for branch
    let commits = Vec::new(); // TODO: fetch commits from branch
    let blobs = Vec::new();   // TODO: gather blobs from branch
    let trees = Vec::new();   // TODO: gather trees from branch
    let head = format!("refs/heads/{}", branch);

    Ok((commits, blobs, trees, head))
}

/// Gather all branches and their heads
fn gather_all_branches(
    _repo: &Repository,
    _specific_branch: Option<&str>,
) -> Result<std::collections::HashMap<String, String>> {
    // TODO: fetch all branches from repository
    Ok(std::collections::HashMap::new())
}

/// Gather complete repository for clone
fn gather_complete_repository(
    _repo: &Repository,
) -> Result<(
    Vec<crate::core::commit::Commit>,
    Vec<crate::core::store::Blob>,
    Vec<crate::core::store::Tree>,
    std::collections::HashMap<String, String>,
    String,
)> {
    // TODO: fetch all commits, blobs, trees, and branches
    Ok((Vec::new(), Vec::new(), Vec::new(), std::collections::HashMap::new(), "main".to_string()))
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
        Ok(_repo) => {
            // TODO: fetch actual branches from repo
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "branches": [],
                "message": "Listed branches"
            }))
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
