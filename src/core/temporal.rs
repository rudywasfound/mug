/// Temporal branching - branches that can fork/merge at any point in history
use crate::core::database::MugDb;
use crate::core::error::Result;
use serde::{Deserialize, Serialize};

/// A temporal branch tracks fork and merge points explicitly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalBranch {
    /// Branch name
    pub name: String,
    /// Current HEAD commit
    pub head: String,
    /// Point in history where this branch forked from its parent
    pub fork_point: Option<String>,
    /// Parent branch (if this is a temporal child)
    pub parent_branch: Option<String>,
    /// Merge points: list of commits where this branch merged in changes
    pub merge_points: Vec<(String, String)>, // (source_commit, merge_commit)
    /// Creation timestamp
    pub created_at: String,
}

pub struct TemporalBranchManager {
    db: MugDb,
}

impl TemporalBranchManager {
    pub fn new(db: MugDb) -> Self {
        TemporalBranchManager { db }
    }

    /// Create a temporal branch at a specific commit
    pub fn create_temporal_branch(
        &self,
        name: String,
        head: String,
        fork_point: Option<String>,
    ) -> Result<()> {
        let branch = TemporalBranch {
            name: name.clone(),
            head,
            fork_point,
            parent_branch: None,
            merge_points: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let serialized = serde_json::to_vec(&branch)?;
        self.db.set("TEMPORAL_BRANCHES", &name, serialized)?;
        Ok(())
    }

    /// Get a temporal branch
    pub fn get_temporal_branch(&self, name: &str) -> Result<Option<TemporalBranch>> {
        match self.db.get("TEMPORAL_BRANCHES", name)? {
            Some(data) => Ok(Some(serde_json::from_slice(&data)?)),
            None => Ok(None),
        }
    }

    /// Merge another branch into this one at the current point in history
    /// This creates a merge point record without requiring linear history
    pub fn merge_temporal_branch(
        &self,
        target_branch: &str,
        source_branch: &str,
        source_commit: &str,
        merge_commit: &str,
    ) -> Result<()> {
        let mut branch = self
            .get_temporal_branch(target_branch)?
            .ok_or(crate::core::error::Error::Custom(
                format!("Branch {} not found", target_branch),
            ))?;

        // Record this merge point
        branch.merge_points.push((
            source_commit.to_string(),
            merge_commit.to_string(),
        ));

        // Update HEAD to the merge commit
        branch.head = merge_commit.to_string();

        let serialized = serde_json::to_vec(&branch)?;
        self.db.set("TEMPORAL_BRANCHES", target_branch, serialized)?;
        Ok(())
    }

    /// Get the history of a temporal branch, including merge points
    pub fn get_temporal_history(&self, branch_name: &str) -> Result<TemporalHistory> {
        let branch = self
            .get_temporal_branch(branch_name)?
            .ok_or(crate::core::error::Error::Custom(
                format!("Branch {} not found", branch_name),
            ))?;

        Ok(TemporalHistory {
            branch_name: branch_name.to_string(),
            head: branch.head,
            fork_point: branch.fork_point,
            merge_points: branch.merge_points,
        })
    }

    /// List all temporal branches
    pub fn list_temporal_branches(&self) -> Result<Vec<TemporalBranch>> {
        let entries = self.db.scan("TEMPORAL_BRANCHES", "")?;
        let mut branches = Vec::new();
        for (_name, data) in entries {
            if let Ok(branch) = serde_json::from_slice::<TemporalBranch>(&data) {
                branches.push(branch);
            }
        }
        Ok(branches)
    }
}

/// Timeline view of a temporal branch
#[derive(Debug, Clone)]
pub struct TemporalHistory {
    pub branch_name: String,
    pub head: String,
    pub fork_point: Option<String>,
    pub merge_points: Vec<(String, String)>,
}

impl TemporalHistory {
    /// Visualize the temporal branch as a DAG
    pub fn visualize(&self) -> String {
        let mut output = format!("Temporal Branch: {}\n", self.branch_name);
        output.push_str("═════════════════════\n");

        if let Some(fork) = &self.fork_point {
            output.push_str(&format!("Fork at: {}\n", &fork[..8]));
        }

        for (i, (source, merge)) in self.merge_points.iter().enumerate() {
            output.push_str(&format!(
                "Merge {}: {} ← {} → {}\n",
                i + 1,
                &source[..8],
                self.branch_name,
                &merge[..8]
            ));
        }

        output.push_str(&format!("HEAD: {}\n", &self.head[..8]));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_branch_creation() {
        // Would need test DB setup
        // Just testing the struct for now
        let branch = TemporalBranch {
            name: "feature".to_string(),
            head: "abc123".to_string(),
            fork_point: Some("def456".to_string()),
            parent_branch: None,
            merge_points: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        assert_eq!(branch.name, "feature");
        assert_eq!(branch.fork_point, Some("def456".to_string()));
    }
}
