use comfy_table::{Cell, Table};
use serde_json::Value;

#[derive(Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

pub fn print_json(value: &Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    );
}

pub fn print_value(value: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json => print_json(value),
        OutputFormat::Yaml => {
            // YAML output is approximated via pretty-printed JSON.
            println!("# (YAML output approximated as JSON)");
            print_json(value);
        }
        OutputFormat::Table => print_json(value),
    }
}

pub fn print_sandboxes(sandboxes: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json | OutputFormat::Yaml => print_value(sandboxes, format),
        OutputFormat::Table => {
            let items = match sandboxes.as_array() {
                Some(arr) => arr,
                None => {
                    print_json(sandboxes);
                    return;
                }
            };
            let mut table = Table::new();
            table.set_header(vec!["ID", "Name", "State", "Provider", "Created"]);
            for s in items {
                table.add_row(vec![
                    Cell::new(short_id(s.get("id"))),
                    Cell::new(s.get("name").and_then(|v| v.as_str()).unwrap_or("-")),
                    Cell::new(s.get("state").and_then(|v| v.as_str()).unwrap_or("?")),
                    Cell::new(
                        s.get("agent_provider")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?"),
                    ),
                    Cell::new(s.get("created_at").and_then(|v| v.as_str()).unwrap_or("-")),
                ]);
            }
            println!("{table}");
        }
    }
}

pub fn print_sandbox(sandbox: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json | OutputFormat::Yaml => print_value(sandbox, format),
        OutputFormat::Table => {
            println!("Sandbox Details:");
            println!("  ID:        {}", val_str(sandbox.get("id")));
            println!(
                "  Name:      {}",
                sandbox.get("name").and_then(|v| v.as_str()).unwrap_or("-")
            );
            println!("  State:     {}", val_str(sandbox.get("state")));
            println!("  Provider:  {}", val_str(sandbox.get("agent_provider")));
            println!("  Created:   {}", val_str(sandbox.get("created_at")));
            println!("  Updated:   {}", val_str(sandbox.get("updated_at")));
            if let Some(url) = sandbox.get("endpoint_url").and_then(|v| v.as_str()) {
                println!("  Endpoint:  {}", url);
            }
            if let Some(stats) = sandbox.get("resource_stats") {
                if !stats.is_null() {
                    println!("  Resources: {}", stats);
                }
            }
            if let Some(labels) = sandbox.get("labels") {
                if labels.is_object() && !labels.as_object().unwrap().is_empty() {
                    println!("  Labels:    {}", labels);
                }
            }
        }
    }
}

pub fn print_sessions(sessions: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json | OutputFormat::Yaml => print_value(sessions, format),
        OutputFormat::Table => {
            let items = match sessions.as_array() {
                Some(arr) => arr,
                None => {
                    print_json(sessions);
                    return;
                }
            };
            let mut table = Table::new();
            table.set_header(vec!["ID", "Sandbox", "State", "Created"]);
            for s in items {
                table.add_row(vec![
                    Cell::new(short_id(s.get("id"))),
                    Cell::new(short_id(s.get("sandbox_id"))),
                    Cell::new(s.get("state").and_then(|v| v.as_str()).unwrap_or("?")),
                    Cell::new(s.get("created_at").and_then(|v| v.as_str()).unwrap_or("-")),
                ]);
            }
            println!("{table}");
        }
    }
}

pub fn print_credentials(creds: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json | OutputFormat::Yaml => print_value(creds, format),
        OutputFormat::Table => {
            let items = match creds.as_array() {
                Some(arr) => arr,
                None => {
                    print_json(creds);
                    return;
                }
            };
            let mut table = Table::new();
            table.set_header(vec!["ID", "Name", "Type", "Created", "Expires"]);
            for c in items {
                table.add_row(vec![
                    Cell::new(short_id(c.get("id"))),
                    Cell::new(c.get("name").and_then(|v| v.as_str()).unwrap_or("-")),
                    Cell::new(
                        c.get("credential_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?"),
                    ),
                    Cell::new(c.get("created_at").and_then(|v| v.as_str()).unwrap_or("-")),
                    Cell::new(c.get("expires_at").and_then(|v| v.as_str()).unwrap_or("-")),
                ]);
            }
            println!("{table}");
        }
    }
}

pub fn print_workspaces(workspaces: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json | OutputFormat::Yaml => print_value(workspaces, format),
        OutputFormat::Table => {
            let items = match workspaces.as_array() {
                Some(arr) => arr,
                None => {
                    print_json(workspaces);
                    return;
                }
            };
            let mut table = Table::new();
            table.set_header(vec!["ID", "Name", "Description", "Created"]);
            for w in items {
                table.add_row(vec![
                    Cell::new(short_id(w.get("id"))),
                    Cell::new(w.get("name").and_then(|v| v.as_str()).unwrap_or("-")),
                    Cell::new(w.get("description").and_then(|v| v.as_str()).unwrap_or("-")),
                    Cell::new(w.get("created_at").and_then(|v| v.as_str()).unwrap_or("-")),
                ]);
            }
            println!("{table}");
        }
    }
}

pub fn print_workspace(workspace: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json | OutputFormat::Yaml => print_value(workspace, format),
        OutputFormat::Table => {
            println!("Workspace Details:");
            println!("  ID:          {}", val_str(workspace.get("id")));
            println!(
                "  Name:        {}",
                workspace
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-")
            );
            println!(
                "  Description: {}",
                workspace
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-")
            );
            println!("  Created:     {}", val_str(workspace.get("created_at")));
            println!("  Updated:     {}", val_str(workspace.get("updated_at")));
            if let Some(spec) = workspace.get("spec") {
                if let Some(repos) = spec.get("repositories").and_then(|v| v.as_array()) {
                    if !repos.is_empty() {
                        println!("  Repositories: {}", repos.len());
                    }
                }
                if let Some(skills) = spec.get("skills").and_then(|v| v.as_array()) {
                    if !skills.is_empty() {
                        println!("  Skills:       {}", skills.len());
                    }
                }
                if let Some(agent) = spec.get("agent") {
                    if let Some(provider) = agent.get("provider").and_then(|v| v.as_str()) {
                        println!("  Provider:     {}", provider);
                    }
                }
            }
        }
    }
}

fn val_str(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => "-".to_string(),
    }
}

fn short_id(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => {
            if s.len() > 8 {
                s[..8].to_string()
            } else {
                s.clone()
            }
        }
        Some(v) => v.to_string(),
        None => "-".to_string(),
    }
}
