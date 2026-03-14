use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::error::CiabError;
use serde::{Deserialize, Serialize};

/// Query params for searching the skills.sh registry.
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Search query (min 2 chars for the upstream API).
    pub q: String,
    /// Max results to return (default 20, max 50).
    pub limit: Option<u32>,
}

/// A skill as returned by the skills.sh search API.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillSearchResult {
    /// Full ID (e.g. "owner/repo/skillId").
    pub id: String,
    /// Skill identifier within the repo.
    #[serde(rename = "skillId")]
    pub skill_id: String,
    /// Display name.
    pub name: String,
    /// Total install count.
    pub installs: u64,
    /// Source repo (e.g. "owner/repo").
    pub source: String,
}

/// Response wrapper for the search endpoint.
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub skills: Vec<SkillSearchResult>,
}

/// Skills.sh upstream response format.
#[derive(Debug, Deserialize)]
struct UpstreamSearchResponse {
    #[serde(default)]
    skills: Vec<SkillSearchResult>,
}

/// Query params for fetching skill metadata from GitHub.
#[derive(Debug, Deserialize)]
pub struct SkillMetadataParams {
    /// Skill source in owner/repo format.
    pub source: String,
    /// Optional sub-path within the repo for the skill.
    pub skill_id: Option<String>,
    /// Git branch/tag (default: main).
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

/// Parsed SKILL.md metadata.
#[derive(Debug, Serialize)]
pub struct SkillMetadata {
    pub source: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub raw_content: String,
    /// Available skills in this repo (from tree scan).
    pub available_skills: Vec<RepoSkillEntry>,
}

/// A skill entry discovered in a GitHub repo.
#[derive(Debug, Serialize, Clone)]
pub struct RepoSkillEntry {
    /// Relative path to the SKILL.md.
    pub path: String,
    /// Skill ID (directory name).
    pub skill_id: String,
}

// ─────────────────── Handlers ───────────────────

/// `GET /api/v1/skills/search` — proxy search to skills.sh registry.
pub async fn search_skills(
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, CiabError> {
    let limit = params.limit.unwrap_or(20).min(50);

    if params.q.len() < 2 {
        return Ok(Json(SearchResponse {
            query: params.q,
            skills: vec![],
        }));
    }

    let url = format!(
        "https://skills.sh/api/search?q={}&limit={}",
        urlencoding::encode(&params.q),
        limit
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| CiabError::Internal(format!("skills.sh search failed: {e}")))?;

    if !resp.status().is_success() {
        return Ok(Json(SearchResponse {
            query: params.q,
            skills: vec![],
        }));
    }

    let upstream: UpstreamSearchResponse = resp
        .json()
        .await
        .map_err(|e| CiabError::Internal(format!("skills.sh response parse failed: {e}")))?;

    Ok(Json(SearchResponse {
        query: params.q,
        skills: upstream.skills,
    }))
}

/// `GET /api/v1/skills/trending` — fetch popular skills by querying multiple terms.
pub async fn trending_skills() -> Result<impl IntoResponse, CiabError> {
    let queries = vec![
        "best-practices",
        "skills",
        "react",
        "typescript",
        "python",
        "docker",
        "testing",
        "api",
        "rust",
        "security",
    ];

    let client = reqwest::Client::new();
    let mut handles = Vec::new();

    for q in &queries {
        let url = format!(
            "https://skills.sh/api/search?q={}&limit=10",
            urlencoding::encode(q)
        );
        let c = client.clone();
        handles.push(tokio::spawn(async move {
            let resp = c.get(&url).send().await.ok()?;
            if !resp.status().is_success() {
                return None;
            }
            resp.json::<UpstreamSearchResponse>().await.ok()
        }));
    }

    let mut seen = std::collections::HashSet::new();
    let mut all_skills: Vec<SkillSearchResult> = Vec::new();

    for handle in handles {
        if let Ok(Some(resp)) = handle.await {
            for skill in resp.skills {
                if seen.insert(skill.id.clone()) {
                    all_skills.push(skill);
                }
            }
        }
    }

    // Sort by installs descending
    all_skills.sort_by(|a, b| b.installs.cmp(&a.installs));
    all_skills.truncate(50);

    Ok(Json(SearchResponse {
        query: "trending".to_string(),
        skills: all_skills,
    }))
}

/// `GET /api/v1/skills/metadata` — fetch SKILL.md from GitHub for a given source.
pub async fn skill_metadata(
    Query(params): Query<SkillMetadataParams>,
) -> Result<impl IntoResponse, CiabError> {
    let source = &params.source;
    let git_ref = params.git_ref.as_deref().unwrap_or("main");

    // Parse owner/repo
    let parts: Vec<&str> = source.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(CiabError::WorkspaceValidationError(
            "source must be in owner/repo format".to_string(),
        ));
    }
    let (owner, repo) = (parts[0], parts[1]);

    // Scan the repo tree for SKILL.md files
    let tree_url =
        format!("https://api.github.com/repos/{owner}/{repo}/git/trees/{git_ref}?recursive=1",);

    let client = reqwest::Client::new();
    let tree_resp = client
        .get(&tree_url)
        .header("User-Agent", "ciab-api")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| CiabError::Internal(format!("GitHub tree fetch failed: {e}")))?;

    let mut available_skills: Vec<RepoSkillEntry> = Vec::new();

    if tree_resp.status().is_success() {
        let tree: serde_json::Value = tree_resp
            .json()
            .await
            .map_err(|e| CiabError::Internal(format!("GitHub tree parse failed: {e}")))?;

        if let Some(entries) = tree.get("tree").and_then(|t| t.as_array()) {
            for entry in entries {
                let path_str = match entry.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => continue,
                };
                if path_str.ends_with("/SKILL.md") || path_str == "SKILL.md" {
                    let skill_id = if path_str == "SKILL.md" {
                        repo.to_string()
                    } else {
                        // e.g. "skills/react-best-practices/SKILL.md" → "react-best-practices"
                        let parent = path_str.trim_end_matches("/SKILL.md");
                        parent.rsplit('/').next().unwrap_or(parent).to_string()
                    };
                    available_skills.push(RepoSkillEntry {
                        path: path_str.to_string(),
                        skill_id,
                    });
                }
            }
        }
    }

    // Determine which SKILL.md to fetch
    let skill_md_path = if let Some(ref sid) = params.skill_id {
        available_skills
            .iter()
            .find(|s| s.skill_id == *sid)
            .map(|s| s.path.clone())
            .unwrap_or_else(|| format!("skills/{sid}/SKILL.md"))
    } else if available_skills.len() == 1 {
        available_skills[0].path.clone()
    } else {
        "SKILL.md".to_string()
    };

    // Fetch the SKILL.md content
    let raw_url =
        format!("https://raw.githubusercontent.com/{owner}/{repo}/{git_ref}/{skill_md_path}",);

    let md_resp = client
        .get(&raw_url)
        .header("User-Agent", "ciab-api")
        .send()
        .await
        .map_err(|e| CiabError::Internal(format!("GitHub SKILL.md fetch failed: {e}")))?;

    let (name, description, raw_content) = if md_resp.status().is_success() {
        let content: String = md_resp
            .text()
            .await
            .map_err(|e| CiabError::Internal(format!("SKILL.md read failed: {e}")))?;

        let (name, desc) = parse_frontmatter(&content);
        (name, desc, content)
    } else {
        (None, None, String::new())
    };

    Ok(Json(SkillMetadata {
        source: source.clone(),
        name,
        description,
        raw_content,
        available_skills,
    }))
}

/// Parse YAML frontmatter from a SKILL.md file.
/// Extracts `name` and `description` fields.
fn parse_frontmatter(content: &str) -> (Option<String>, Option<String>) {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        return (None, None);
    }

    let after_first = &trimmed[3..];
    if let Some(end) = after_first.find("---") {
        let frontmatter = &after_first[..end];

        let mut name = None;
        let mut description = None;

        for line in frontmatter.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("name:") {
                name = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
            } else if let Some(val) = line.strip_prefix("description:") {
                description = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }

        (name, description)
    } else {
        (None, None)
    }
}
