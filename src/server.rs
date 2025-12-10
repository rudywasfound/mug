use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, middleware};
use crate::protocol::{PushResponse, PullResponse, FetchResponse, CloneResponse};
use crate::auth::ServerAuth;
use crate::error::Result;
use crate::repo::Repository;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

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
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing authorization token"})),
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "write") {
        Ok(true) => {},
        _ => return HttpResponse::Forbidden().json(serde_json::json!({"error": "Permission denied"})),
    }
    drop(auth);

    // Get or create repository
    let repo_path = state.repos_dir.join(&repo_name);
    let _repo = match Repository::open(&repo_path) {
        Ok(r) => r,
        Err(_) => {
            // Try to initialize if doesn't exist
            match Repository::init(&repo_path) {
                Ok(r) => r,
                Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Failed to initialize repo: {}", e)})),
            }
        }
    };

    // TODO: Process push
    // - Receive commits, blobs, trees
    // - Store in object store
    // - Update branch reference

    HttpResponse::Ok().json(PushResponse {
        success: true,
        message: "Push successful".to_string(),
        head: Some("main".to_string()),
    })
}

/// Pull endpoint: GET /repo/{name}/pull
async fn pull_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing authorization token"})),
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "read") {
        Ok(true) => {},
        _ => return HttpResponse::Forbidden().json(serde_json::json!({"error": "Permission denied"})),
    }
    drop(auth);

    let repo_path = state.repos_dir.join(&repo_name);
    let _repo = match Repository::open(&repo_path) {
        Ok(r) => r,
        Err(e) => return HttpResponse::NotFound().json(serde_json::json!({"error": format!("Repository not found: {}", e)})),
    };

    // TODO: Gather commits, blobs, trees for branch
    // For now, return empty response
    HttpResponse::Ok().json(PullResponse {
        success: true,
        commits: Vec::new(),
        blobs: Vec::new(),
        trees: Vec::new(),
        head: "main".to_string(),
        message: "No changes".to_string(),
    })
}

/// Fetch endpoint: GET /repo/{name}/fetch
async fn fetch_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing authorization token"})),
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "read") {
        Ok(true) => {},
        _ => return HttpResponse::Forbidden().json(serde_json::json!({"error": "Permission denied"})),
    }
    drop(auth);

    let repo_path = state.repos_dir.join(&repo_name);
    let _repo = match Repository::open(&repo_path) {
        Ok(r) => r,
        Err(e) => return HttpResponse::NotFound().json(serde_json::json!({"error": format!("Repository not found: {}", e)})),
    };

    // TODO: Gather branches and their heads
    HttpResponse::Ok().json(FetchResponse {
        success: true,
        branches: Default::default(),
        message: "OK".to_string(),
    })
}

/// Clone endpoint: GET /repo/{name}/clone
async fn clone_handler(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let repo_name = path.into_inner();

    // Extract and validate token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing authorization token"})),
    };

    // Verify permission
    let auth = state.auth.lock().unwrap();
    match auth.verify(&token, &repo_name, "read") {
        Ok(true) => {},
        _ => return HttpResponse::Forbidden().json(serde_json::json!({"error": "Permission denied"})),
    }
    drop(auth);

    let repo_path = state.repos_dir.join(&repo_name);
    let _repo = match Repository::open(&repo_path) {
        Ok(r) => r,
        Err(e) => return HttpResponse::NotFound().json(serde_json::json!({"error": format!("Repository not found: {}", e)})),
    };

    // TODO: Gather all commits, blobs, trees, and branches
    HttpResponse::Ok().json(CloneResponse {
        commits: Vec::new(),
        blobs: Vec::new(),
        trees: Vec::new(),
        branches: Default::default(),
        default_branch: "main".to_string(),
    })
}

/// Health check
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

/// Start HTTP server
pub async fn run_server(repos_dir: PathBuf, host: &str, port: u16) -> Result<()> {
    let auth = Arc::new(Mutex::new(ServerAuth::new()));
    
    let state = web::Data::new(ServerState {
        repos_dir,
        auth,
    });

    println!("Starting MUG HTTP server on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .route("/health", web::get().to(health))
            .route("/repo/{name}/push", web::post().to(push_handler))
            .route("/repo/{name}/pull", web::get().to(pull_handler))
            .route("/repo/{name}/fetch", web::get().to(fetch_handler))
            .route("/repo/{name}/clone", web::get().to(clone_handler))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;

    Ok(())
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
