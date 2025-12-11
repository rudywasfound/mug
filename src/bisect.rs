use crate::error::{Error, Result};
use crate::repo::Repository;

/// Bisect session state
#[derive(Debug, Clone)]
pub struct BisectSession {
    pub good_commit: String,
    pub bad_commit: String,
    pub current_commit: String,
    pub tested_commits: Vec<(String, BisectResult)>,
}

/// Result of testing a commit during bisect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BisectResult {
    Good,
    Bad,
    Skip,
}

/// Start a bisect session
pub fn start(repo: &Repository, bad_commit: &str, good_commit: &str) -> Result<BisectSession> {
    let commits = repo.log()?;

    // Validate commits exist
    let bad_exists = commits.iter().any(|c| c.contains(bad_commit));
    let good_exists = commits.iter().any(|c| c.contains(good_commit));

    if !bad_exists {
        return Err(Error::Custom(format!("Bad commit {} not found", bad_commit)));
    }

    if !good_exists {
        return Err(Error::Custom(format!("Good commit {} not found", good_commit)));
    }

    let bad_idx = commits.iter().position(|c| c.contains(bad_commit)).unwrap();
    let good_idx = commits.iter().position(|c| c.contains(good_commit)).unwrap();

    // Find midpoint
    let mid_idx = (bad_idx + good_idx) / 2;
    let current = commits
        .get(mid_idx)
        .map(|c| c.lines().next().unwrap_or("").to_string())
        .unwrap_or_default();

    Ok(BisectSession {
        good_commit: good_commit.to_string(),
        bad_commit: bad_commit.to_string(),
        current_commit: current,
        tested_commits: vec![],
    })
}

/// Mark current commit as good and advance bisect
pub fn mark_good(
    repo: &Repository,
    mut session: BisectSession,
) -> Result<BisectProgress> {
    session
        .tested_commits
        .push((session.current_commit.clone(), BisectResult::Good));

    let commits = repo.log()?;
    let bad_idx = commits
        .iter()
        .position(|c| c.contains(&session.bad_commit))
        .ok_or_else(|| Error::Custom("Bad commit lost".to_string()))?;

    let good_idx = commits.iter().position(|c| c.contains(&session.good_commit));

    match good_idx {
        Some(g) => {
            let remaining = (bad_idx as i32 - g as i32).abs() as usize;

            if remaining <= 1 {
                return Ok(BisectProgress::Found(session.bad_commit.clone()));
            }

            let mid_idx = (bad_idx + g) / 2;
            let next_commit = commits
                .get(mid_idx)
                .map(|c| c.lines().next().unwrap_or("").to_string())
                .unwrap_or_default();

            session.current_commit = next_commit.clone();
            Ok(BisectProgress::Continue {
                session,
                next_commit,
                remaining,
            })
        }
        None => Ok(BisectProgress::Error(
            "Good commit not found".to_string(),
        )),
    }
}

/// Mark current commit as bad and advance bisect
pub fn mark_bad(
    repo: &Repository,
    mut session: BisectSession,
) -> Result<BisectProgress> {
    session
        .tested_commits
        .push((session.current_commit.clone(), BisectResult::Bad));

    session.bad_commit = session.current_commit.clone();

    let commits = repo.log()?;
    let good_idx = commits
        .iter()
        .position(|c| c.contains(&session.good_commit))
        .ok_or_else(|| Error::Custom("Good commit lost".to_string()))?;

    let bad_idx = commits.iter().position(|c| c.contains(&session.bad_commit));

    match bad_idx {
        Some(b) => {
            let remaining = (b as i32 - good_idx as i32).abs() as usize;

            if remaining <= 1 {
                return Ok(BisectProgress::Found(session.bad_commit.clone()));
            }

            let mid_idx = (b + good_idx) / 2;
            let next_commit = commits
                .get(mid_idx)
                .map(|c| c.lines().next().unwrap_or("").to_string())
                .unwrap_or_default();

            session.current_commit = next_commit.clone();
            Ok(BisectProgress::Continue {
                session,
                next_commit,
                remaining,
            })
        }
        None => Ok(BisectProgress::Error("Bad commit not found".to_string())),
    }
}

/// Progress of bisect operation
#[derive(Debug, Clone)]
pub enum BisectProgress {
    Continue {
        session: BisectSession,
        next_commit: String,
        remaining: usize,
    },
    Found(String),
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisect_result_equality() {
        assert_eq!(BisectResult::Good, BisectResult::Good);
        assert_ne!(BisectResult::Good, BisectResult::Bad);
    }

    #[test]
    fn test_bisect_session_creation() {
        let session = BisectSession {
            good_commit: "abc123".to_string(),
            bad_commit: "def456".to_string(),
            current_commit: "mid789".to_string(),
            tested_commits: vec![],
        };

        assert_eq!(session.good_commit, "abc123");
        assert_eq!(session.bad_commit, "def456");
        assert!(session.tested_commits.is_empty());
    }

    #[test]
    fn test_bisect_progress_found() {
        match BisectProgress::Found("abc123".to_string()) {
            BisectProgress::Found(commit) => {
                assert_eq!(commit, "abc123");
            }
            _ => panic!("Expected Found variant"),
        }
    }

    #[test]
    fn test_bisect_progress_error() {
        match BisectProgress::Error("test error".to_string()) {
            BisectProgress::Error(msg) => {
                assert_eq!(msg, "test error");
            }
            _ => panic!("Expected Error variant"),
        }
    }
}
