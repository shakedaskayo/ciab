use ciab_core::error::{CiabError, CiabResult};
use ciab_core::types::workspace::{Workspace, WorkspaceFilters, WorkspaceSpec};
use uuid::Uuid;

use crate::Database;

impl Database {
    pub async fn insert_workspace(&self, workspace: &Workspace) -> CiabResult<()> {
        let id = workspace.id.to_string();
        let name = &workspace.name;
        let description = workspace.description.as_deref();
        let spec_json = serde_json::to_string(&workspace.spec)?;
        let labels_json = serde_json::to_string(&workspace.labels)?;
        let created_at = workspace.created_at.to_rfc3339();
        let updated_at = workspace.updated_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO workspaces (id, name, description, spec_json, labels_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(&spec_json)
        .bind(&labels_json)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_workspace(&self, id: &Uuid) -> CiabResult<Option<Workspace>> {
        let id_str = id.to_string();

        let row: Option<(String, String, Option<String>, String, String, String, String)> =
            sqlx::query_as(
                "SELECT id, name, description, spec_json, labels_json, created_at, updated_at FROM workspaces WHERE id = ?",
            )
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        match row {
            Some((id, name, description, spec_json, labels_json, created_at, updated_at)) => {
                let spec: WorkspaceSpec = serde_json::from_str(&spec_json)?;
                let labels = serde_json::from_str(&labels_json)?;
                let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| CiabError::Database(e.to_string()))?
                    .with_timezone(&chrono::Utc);
                let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
                    .map_err(|e| CiabError::Database(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                Ok(Some(Workspace {
                    id: Uuid::parse_str(&id).map_err(|e| CiabError::Database(e.to_string()))?,
                    name,
                    description,
                    spec,
                    labels,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_workspaces(&self, filters: &WorkspaceFilters) -> CiabResult<Vec<Workspace>> {
        let mut query =
            String::from("SELECT id, name, description, spec_json, labels_json, created_at, updated_at FROM workspaces WHERE 1=1");
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref name) = filters.name {
            query.push_str(" AND name LIKE ?");
            bind_values.push(format!("%{}%", name));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                String,
                String,
            ),
        >(&query);
        for val in &bind_values {
            q = q.bind(val);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for (id, name, description, spec_json, labels_json, created_at, updated_at) in rows {
            let spec: WorkspaceSpec = serde_json::from_str(&spec_json)?;
            let labels: std::collections::HashMap<String, String> =
                serde_json::from_str(&labels_json)?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| CiabError::Database(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| CiabError::Database(e.to_string()))?
                .with_timezone(&chrono::Utc);

            let workspace = Workspace {
                id: Uuid::parse_str(&id).map_err(|e| CiabError::Database(e.to_string()))?,
                name,
                description,
                spec,
                labels: labels.clone(),
                created_at,
                updated_at,
            };

            // Apply label filters in-memory
            if !filters.labels.is_empty() {
                let matches = filters
                    .labels
                    .iter()
                    .all(|(k, v)| labels.get(k).map_or(false, |lv| lv == v));
                if !matches {
                    continue;
                }
            }

            results.push(workspace);
        }

        Ok(results)
    }

    pub async fn update_workspace(&self, id: &Uuid, workspace: &Workspace) -> CiabResult<()> {
        let id_str = id.to_string();
        let spec_json = serde_json::to_string(&workspace.spec)?;
        let labels_json = serde_json::to_string(&workspace.labels)?;
        let updated_at = workspace.updated_at.to_rfc3339();

        sqlx::query(
            "UPDATE workspaces SET name = ?, description = ?, spec_json = ?, labels_json = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&workspace.name)
        .bind(workspace.description.as_deref())
        .bind(&spec_json)
        .bind(&labels_json)
        .bind(&updated_at)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_workspace(&self, id: &Uuid) -> CiabResult<()> {
        let id_str = id.to_string();

        sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn link_sandbox_to_workspace(
        &self,
        workspace_id: &Uuid,
        sandbox_id: &Uuid,
    ) -> CiabResult<()> {
        let workspace_id_str = workspace_id.to_string();
        let sandbox_id_str = sandbox_id.to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO workspace_sandboxes (workspace_id, sandbox_id, created_at) VALUES (?, ?, ?)",
        )
        .bind(&workspace_id_str)
        .bind(&sandbox_id_str)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn unlink_sandbox_from_workspace(
        &self,
        workspace_id: &Uuid,
        sandbox_id: &Uuid,
    ) -> CiabResult<()> {
        let workspace_id_str = workspace_id.to_string();
        let sandbox_id_str = sandbox_id.to_string();

        sqlx::query("DELETE FROM workspace_sandboxes WHERE workspace_id = ? AND sandbox_id = ?")
            .bind(&workspace_id_str)
            .bind(&sandbox_id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn list_workspace_sandboxes(&self, workspace_id: &Uuid) -> CiabResult<Vec<String>> {
        let workspace_id_str = workspace_id.to_string();

        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT sandbox_id FROM workspace_sandboxes WHERE workspace_id = ?")
                .bind(&workspace_id_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    // --- Template Sources ---

    pub async fn insert_template_source(
        &self,
        source: &ciab_core::types::workspace::TemplateSource,
    ) -> CiabResult<()> {
        let id = source.id.to_string();
        let last_synced = source.last_synced_at.map(|t| t.to_rfc3339());
        let created_at = source.created_at.to_rfc3339();
        let updated_at = source.updated_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO template_sources (id, name, url, branch, templates_path, last_synced_at, template_count, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&source.name)
        .bind(&source.url)
        .bind(&source.branch)
        .bind(&source.templates_path)
        .bind(&last_synced)
        .bind(source.template_count as i64)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_template_source(
        &self,
        id: &Uuid,
    ) -> CiabResult<Option<ciab_core::types::workspace::TemplateSource>> {
        let id_str = id.to_string();

        let row: Option<(String, String, String, String, String, Option<String>, i64, String, String)> =
            sqlx::query_as(
                "SELECT id, name, url, branch, templates_path, last_synced_at, template_count, created_at, updated_at
                 FROM template_sources WHERE id = ?",
            )
            .bind(&id_str)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        match row {
            Some((
                id,
                name,
                url,
                branch,
                templates_path,
                last_synced_at,
                template_count,
                created_at,
                updated_at,
            )) => {
                let last_synced = last_synced_at
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|d| d.with_timezone(&chrono::Utc))
                    })
                    .transpose()
                    .map_err(|e| CiabError::Database(e.to_string()))?;
                let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| CiabError::Database(e.to_string()))?
                    .with_timezone(&chrono::Utc);
                let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
                    .map_err(|e| CiabError::Database(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                Ok(Some(ciab_core::types::workspace::TemplateSource {
                    id: Uuid::parse_str(&id).map_err(|e| CiabError::Database(e.to_string()))?,
                    name,
                    url,
                    branch,
                    templates_path,
                    last_synced_at: last_synced,
                    template_count: template_count as u32,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_template_sources(
        &self,
    ) -> CiabResult<Vec<ciab_core::types::workspace::TemplateSource>> {
        let rows: Vec<(String, String, String, String, String, Option<String>, i64, String, String)> =
            sqlx::query_as(
                "SELECT id, name, url, branch, templates_path, last_synced_at, template_count, created_at, updated_at
                 FROM template_sources ORDER BY created_at DESC",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for (
            id,
            name,
            url,
            branch,
            templates_path,
            last_synced_at,
            template_count,
            created_at,
            updated_at,
        ) in rows
        {
            let last_synced = last_synced_at
                .map(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&chrono::Utc))
                })
                .transpose()
                .map_err(|e| CiabError::Database(e.to_string()))?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| CiabError::Database(e.to_string()))?
                .with_timezone(&chrono::Utc);
            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| CiabError::Database(e.to_string()))?
                .with_timezone(&chrono::Utc);

            results.push(ciab_core::types::workspace::TemplateSource {
                id: Uuid::parse_str(&id).map_err(|e| CiabError::Database(e.to_string()))?,
                name,
                url,
                branch,
                templates_path,
                last_synced_at: last_synced,
                template_count: template_count as u32,
                created_at,
                updated_at,
            });
        }

        Ok(results)
    }

    pub async fn update_template_source(
        &self,
        id: &Uuid,
        source: &ciab_core::types::workspace::TemplateSource,
    ) -> CiabResult<()> {
        let id_str = id.to_string();
        let last_synced = source.last_synced_at.map(|t| t.to_rfc3339());
        let updated_at = source.updated_at.to_rfc3339();

        sqlx::query(
            "UPDATE template_sources SET name = ?, url = ?, branch = ?, templates_path = ?, last_synced_at = ?, template_count = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&source.name)
        .bind(&source.url)
        .bind(&source.branch)
        .bind(&source.templates_path)
        .bind(&last_synced)
        .bind(source.template_count as i64)
        .bind(&updated_at)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_template_source(&self, id: &Uuid) -> CiabResult<()> {
        let id_str = id.to_string();

        // Delete workspaces imported from this source
        let pattern = format!("%\"ciab/source_id\":\"{}\"%", id_str);
        sqlx::query("DELETE FROM workspaces WHERE labels_json LIKE ?")
            .bind(&pattern)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        sqlx::query("DELETE FROM template_sources WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| CiabError::Database(e.to_string()))?;

        Ok(())
    }
}
