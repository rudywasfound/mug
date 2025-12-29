use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use uuid::Uuid;

use crate::core::database::MugDb;
use crate::core::error::Result;

/// Represents a resumable operation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub op_type: OperationType,
    pub status: OperationStatus,
    pub created_at: String,
    pub started_at: String,
    pub last_updated: String,
    pub state: OperationState,
    pub progress: OperationProgress,
}

/// Type of operation that can be resumed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Long-running pack operation
    Pack,
    /// Clone or fetch operation
    Clone,
    /// Fetch operation
    Fetch,
    /// Push operation
    Push,
    /// Rebase operation
    Rebase,
    /// Merge operation
    Merge,
    /// Custom/unknown operation
    Custom(String),
}

impl OperationType {
    pub fn as_str(&self) -> &str {
        match self {
            OperationType::Pack => "pack",
            OperationType::Clone => "clone",
            OperationType::Fetch => "fetch",
            OperationType::Push => "push",
            OperationType::Rebase => "rebase",
            OperationType::Merge => "merge",
            OperationType::Custom(s) => s.as_str(),
        }
    }
}

/// Current status of an operation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationStatus {
    /// Currently running
    Running,
    /// Paused/interrupted
    Paused,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed,
}

impl OperationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            OperationStatus::Running => "running",
            OperationStatus::Paused => "paused",
            OperationStatus::Completed => "completed",
            OperationStatus::Failed => "failed",
        }
    }
}

/// Detailed state of the operation for resumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationState {
    /// Serialized checkpoint data
    pub checkpoint: String,
    /// Current step/phase
    pub current_step: String,
    /// Total steps if known
    pub total_steps: Option<usize>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Custom metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProgress {
    /// Items processed
    pub processed: u64,
    /// Total items to process
    pub total: Option<u64>,
    /// Bytes transferred
    pub bytes_processed: u64,
    /// Total bytes to transfer
    pub total_bytes: Option<u64>,
}

impl OperationProgress {
    pub fn percentage(&self) -> Option<f64> {
        self.total.map(|t| {
            if t == 0 {
                0.0
            } else {
                (self.processed as f64 / t as f64) * 100.0
            }
        })
    }

    pub fn bytes_percentage(&self) -> Option<f64> {
        self.total_bytes.map(|t| {
            if t == 0 {
                0.0
            } else {
                (self.bytes_processed as f64 / t as f64) * 100.0
            }
        })
    }
}

/// Manager for operations that can be resumed
pub struct OperationManager {
    db: MugDb,
}

impl OperationManager {
    pub fn new(db: MugDb) -> Self {
        OperationManager { db }
    }

    /// Create a new operation
    pub fn create(
        &self,
        op_type: OperationType,
        checkpoint: String,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<Operation> {
        let id = format!("op-{}", Uuid::new_v4());
        let now = Local::now().to_rfc3339();

        let operation = Operation {
            id: id.clone(),
            op_type,
            status: OperationStatus::Running,
            created_at: now.clone(),
            started_at: now.clone(),
            last_updated: now.clone(),
            state: OperationState {
                checkpoint,
                current_step: "initialized".to_string(),
                total_steps: None,
                error_message: None,
                metadata,
            },
            progress: OperationProgress {
                processed: 0,
                total: None,
                bytes_processed: 0,
                total_bytes: None,
            },
        };

        let serialized = serde_json::to_vec(&operation)?;
        self.db.set("operations", &id, serialized)?;

        Ok(operation)
    }

    /// Get an operation by ID
    pub fn get(&self, op_id: &str) -> Result<Option<Operation>> {
        match self.db.get("operations", op_id)? {
            Some(data) => {
                let operation: Operation = serde_json::from_slice(&data)?;
                Ok(Some(operation))
            }
            None => Ok(None),
        }
    }

    /// Update operation status
    pub fn update_status(&self, op_id: &str, status: OperationStatus) -> Result<()> {
        if let Some(mut op) = self.get(op_id)? {
            op.status = status;
            op.last_updated = Local::now().to_rfc3339();
            let serialized = serde_json::to_vec(&op)?;
            self.db.set("operations", op_id, serialized)?;
            Ok(())
        } else {
            Err(crate::core::error::Error::Custom(format!(
                "Operation {} not found",
                op_id
            )))
        }
    }

    /// Update operation progress
    pub fn update_progress(
        &self,
        op_id: &str,
        processed: u64,
        total: Option<u64>,
        bytes_processed: u64,
        total_bytes: Option<u64>,
    ) -> Result<()> {
        if let Some(mut op) = self.get(op_id)? {
            op.progress.processed = processed;
            op.progress.total = total;
            op.progress.bytes_processed = bytes_processed;
            op.progress.total_bytes = total_bytes;
            op.last_updated = Local::now().to_rfc3339();
            let serialized = serde_json::to_vec(&op)?;
            self.db.set("operations", op_id, serialized)?;
            Ok(())
        } else {
            Err(crate::core::error::Error::Custom(format!(
                "Operation {} not found",
                op_id
            )))
        }
    }

    /// Update operation checkpoint and step
    pub fn update_checkpoint(
        &self,
        op_id: &str,
        checkpoint: String,
        current_step: String,
        total_steps: Option<usize>,
    ) -> Result<()> {
        if let Some(mut op) = self.get(op_id)? {
            op.state.checkpoint = checkpoint;
            op.state.current_step = current_step;
            op.state.total_steps = total_steps;
            op.last_updated = Local::now().to_rfc3339();
            let serialized = serde_json::to_vec(&op)?;
            self.db.set("operations", op_id, serialized)?;
            Ok(())
        } else {
            Err(crate::core::error::Error::Custom(format!(
                "Operation {} not found",
                op_id
            )))
        }
    }

    /// Mark operation as completed
    pub fn complete(&self, op_id: &str) -> Result<()> {
        self.update_status(op_id, OperationStatus::Completed)
    }

    /// Mark operation as failed with error message
    pub fn fail(&self, op_id: &str, error: &str) -> Result<()> {
        if let Some(mut op) = self.get(op_id)? {
            op.status = OperationStatus::Failed;
            op.state.error_message = Some(error.to_string());
            op.last_updated = Local::now().to_rfc3339();
            let serialized = serde_json::to_vec(&op)?;
            self.db.set("operations", op_id, serialized)?;
            Ok(())
        } else {
            Err(crate::core::error::Error::Custom(format!(
                "Operation {} not found",
                op_id
            )))
        }
    }

    /// List all operations, optionally filtered by status
    pub fn list(&self, status_filter: Option<OperationStatus>) -> Result<Vec<Operation>> {
        let entries = self.db.scan("operations", "")?;
        let mut operations = Vec::new();

        for (_, value) in entries {
            if let Ok(op) = serde_json::from_slice::<Operation>(&value) {
                if let Some(filter) = status_filter {
                    if op.status == filter {
                        operations.push(op);
                    }
                } else {
                    operations.push(op);
                }
            }
        }

        // Sort by timestamp (newest first)
        operations.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
        Ok(operations)
    }

    /// Get the most recent pausable operation of a given type
    pub fn get_latest_pausable(&self, op_type: &str) -> Result<Option<Operation>> {
        let mut operations = self.list(Some(OperationStatus::Paused))?;
        operations.retain(|op| op.op_type.as_str() == op_type);
        Ok(operations.first().cloned())
    }

    /// Get running operation
    pub fn get_running(&self, op_type: &str) -> Result<Option<Operation>> {
        let mut operations = self.list(Some(OperationStatus::Running))?;
        operations.retain(|op| op.op_type.as_str() == op_type);
        Ok(operations.first().cloned())
    }

    /// Delete an operation
    pub fn delete(&self, op_id: &str) -> Result<()> {
        self.db.delete("operations", op_id)?;
        Ok(())
    }

    /// Clean up old completed/failed operations (older than days_old)
    pub fn cleanup_old(&self, days_old: i64) -> Result<usize> {
        let all_operations = self.list(None)?;
        let cutoff = Local::now() - chrono::Duration::days(days_old);
        let mut deleted = 0;

        for op in all_operations {
            if let Ok(last_updated) = DateTime::parse_from_rfc3339(&op.last_updated) {
                let dt: DateTime<Local> = last_updated.with_timezone(&Local);
                if dt < cutoff && (op.status == OperationStatus::Completed || op.status == OperationStatus::Failed) {
                    self.delete(&op.id)?;
                    deleted += 1;
                }
            }
        }

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_progress_percentage() {
        let progress = OperationProgress {
            processed: 50,
            total: Some(100),
            bytes_processed: 0,
            total_bytes: None,
        };
        assert_eq!(progress.percentage(), Some(50.0));
    }

    #[test]
    fn test_operation_progress_zero_total() {
        let progress = OperationProgress {
            processed: 0,
            total: Some(0),
            bytes_processed: 0,
            total_bytes: None,
        };
        assert_eq!(progress.percentage(), Some(0.0));
    }

    #[test]
    fn test_operation_type_as_str() {
        assert_eq!(OperationType::Pack.as_str(), "pack");
        assert_eq!(OperationType::Clone.as_str(), "clone");
        assert_eq!(OperationType::Custom("test".to_string()).as_str(), "test");
    }

    #[test]
    fn test_operation_status_as_str() {
        assert_eq!(OperationStatus::Running.as_str(), "running");
        assert_eq!(OperationStatus::Paused.as_str(), "paused");
        assert_eq!(OperationStatus::Completed.as_str(), "completed");
        assert_eq!(OperationStatus::Failed.as_str(), "failed");
    }
}
