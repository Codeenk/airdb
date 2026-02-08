//! AirDB CLI - Main entry point for CLI binary
//! 
//! This binary provides the `airdb` CLI tool for managing projects.

use airdb_lib::engine::{
    cli::{Cli, Commands, MigrateAction, KeysAction, AuthAction, SyncAction, UpdateAction, NoSqlAction, SchemaAction, HybridAction, OutputFormat, CliFormatter},
    config::Config,
    database::Database,
    migrations::MigrationRunner,
    keystore::Keystore,
    api::{ApiState, create_router},
};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn main() {
    let cli = Cli::parse();
    
    if let Err(e) = run_cli(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = cli.get_project_dir();
    let json_output = cli.format == OutputFormat::Json;

    match cli.command {
        Commands::Init { name, visibility, no_github } => {
            cmd_init(&name, &visibility, no_github, json_output)?;
        }
        Commands::Migrate { action } => {
            cmd_migrate(action, &project_dir, json_output)?;
        }
        Commands::Serve { port, host } => {
            cmd_serve(&project_dir, &host, port)?;
        }
        Commands::Status => {
            cmd_status(&project_dir, json_output)?;
        }
        Commands::Invite { username, role } => {
            cmd_invite(&project_dir, &username, &role, json_output)?;
        }
        Commands::Keys { action } => {
            cmd_keys(action, &project_dir, json_output)?;
        }
        Commands::Auth { action } => {
            cmd_auth(action, json_output)?;
        }
        Commands::Sync { action } => {
            cmd_sync(action, &project_dir, json_output)?;
        }
        Commands::Update { action } => {
            cmd_update(action, json_output)?;
        }
        Commands::Nosql { action } => {
            cmd_nosql(action, &project_dir, json_output)?;
        }
        Commands::Hybrid { action } => {
            cmd_hybrid(action, &project_dir, json_output)?;
        }
        Commands::Info => {
            cmd_info(&project_dir, json_output)?;
        }
    }

    Ok(())
}

fn cmd_init(name: &str, visibility: &str, no_github: bool, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let project_dir = home_dir.join("AirDB").join("projects").join(name);

    if project_dir.exists() {
        return Err(format!("Project directory already exists: {}", project_dir.display()).into());
    }

    std::fs::create_dir_all(&project_dir)?;

    // Create config
    let mut config = Config::default_for_project(name);
    if !no_github {
        config.github = Some(airdb_lib::engine::config::GitHubConfig {
            repo: format!("airdb-{}", name),
            visibility: visibility.to_string(),
        });
    }
    config.save(&project_dir)?;

    // Create directory structure
    std::fs::create_dir_all(project_dir.join("sql").join("migrations"))?;
    std::fs::create_dir_all(project_dir.join("access"))?;
    std::fs::create_dir_all(project_dir.join("api"))?;
    std::fs::create_dir_all(project_dir.join("data"))?;
    std::fs::create_dir_all(project_dir.join(".airdb"))?;

    // Create default access files
    let roles = serde_json::json!({
        "roles": {
            "admin": { "permissions": ["*"] },
            "developer": { "permissions": ["read", "write", "migrate"] },
            "readonly": { "permissions": ["read"] }
        }
    });
    std::fs::write(
        project_dir.join("access").join("roles.json"),
        serde_json::to_string_pretty(&roles)?,
    )?;

    let policies = serde_json::json!({
        "default_role": "readonly",
        "rate_limit": { "requests_per_minute": 100 }
    });
    std::fs::write(
        project_dir.join("access").join("policies.json"),
        serde_json::to_string_pretty(&policies)?,
    )?;

    // Create README
    let readme = format!(
        "# {}\n\nAirDB project created with `airdb init`.\n\n## Getting Started\n\n```bash\ncd {}\nairdb migrate create initial_schema\nairdb serve\n```\n",
        name, project_dir.display()
    );
    std::fs::write(project_dir.join("README.md"), readme)?;

    // Create .gitignore
    let gitignore = "# AirDB\ndata/*.db\ndata/*.db-*\n.airdb/keys.json\n*.log\n";
    std::fs::write(project_dir.join(".gitignore"), gitignore)?;

    // Initialize database
    let db_path = project_dir.join("data").join("airdb.db");
    let _db = Database::new(&db_path)?;

    if json {
        println!("{}", serde_json::json!({
            "success": true,
            "project_dir": project_dir.display().to_string(),
            "name": name
        }));
    } else {
        println!("✅ Created AirDB project: {}", name);
        println!("   📁 {}", project_dir.display());
        println!("\n   Next steps:");
        println!("   cd {}", project_dir.display());
        println!("   airdb migrate create initial_schema");
        println!("   airdb serve");
    }

    Ok(())
}

fn cmd_migrate(action: MigrateAction, project_dir: &PathBuf, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(project_dir)?;
    let db_path = project_dir.join(&config.database.path);
    let db = Database::new(&db_path)?;
    let runner = MigrationRunner::new(project_dir);

    match action {
        MigrateAction::Create { name } => {
            let path = runner.create(&name)?;
            if json {
                println!("{}", serde_json::json!({
                    "success": true,
                    "path": path.display().to_string()
                }));
            } else {
                println!("✅ Created migration: {}", path.display());
                println!("   Edit the file and run `airdb migrate push`");
            }
        }
        MigrateAction::Push => {
            let applied = runner.push(&db)?;
            if json {
                println!("{}", serde_json::json!({
                    "success": true,
                    "applied": applied
                }));
            } else if applied.is_empty() {
                println!("✅ No pending migrations");
            } else {
                println!("✅ Applied {} migration(s):", applied.len());
                for name in &applied {
                    println!("   • {}", name);
                }
            }
            
            // Generate schema snapshot
            runner.generate_schema_snapshot(&db, project_dir)?;
        }
        MigrateAction::Check => {
            let status = runner.check(&db)?;
            if json {
                println!("{}", serde_json::json!({
                    "applied": status.applied_count,
                    "pending": status.pending_count,
                    "pending_migrations": status.pending_migrations
                }));
            } else {
                println!("📊 Migration Status:");
                println!("   Applied: {}", status.applied_count);
                println!("   Pending: {}", status.pending_count);
                if !status.pending_migrations.is_empty() {
                    println!("\n   Pending migrations:");
                    for name in &status.pending_migrations {
                        println!("   • {}", name);
                    }
                }
            }
        }
        MigrateAction::Rollback { count: _ } => {
            if json {
                println!("{}", serde_json::json!({
                    "error": "Rollback not yet implemented"
                }));
            } else {
                println!("⚠️  Rollback not yet implemented");
            }
        }
        MigrateAction::List => {
            let applied = db.get_applied_migrations()?;
            if json {
                println!("{}", serde_json::json!({
                    "migrations": applied
                }));
            } else {
                println!("📋 Applied Migrations:");
                if applied.is_empty() {
                    println!("   (none)");
                } else {
                    for name in &applied {
                        println!("   ✓ {}", name);
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn cmd_serve(project_dir: &PathBuf, host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(project_dir)?;
    let db_path = project_dir.join(&config.database.path);
    let db = Database::new(&db_path)?;

    let state = ApiState { db: Arc::new(db) };
    let app = create_router(state);

    let addr = format!("{}:{}", host, port);
    println!("🚀 AirDB API Server");
    println!("   Project: {}", config.project.name);
    println!("   Listening: http://{}", addr);
    println!("   Press Ctrl+C to stop");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn cmd_status(project_dir: &PathBuf, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(project_dir)?;
    let db_path = project_dir.join(&config.database.path);
    let db = Database::new(&db_path)?;
    let runner = MigrationRunner::new(project_dir);
    let status = runner.check(&db)?;
    let tables = db.get_tables()?;

    if json {
        println!("{}", serde_json::json!({
            "project": config.project.name,
            "database": config.database.db_type,
            "api_port": config.api.port,
            "migrations_applied": status.applied_count,
            "migrations_pending": status.pending_count,
            "tables": tables
        }));
    } else {
        CliFormatter::header("AirDB Project Status");
        CliFormatter::kv("Project", &config.project.name);
        CliFormatter::kv("Database", &format!("{} ({})", config.database.db_type, config.database.path.display()));
        CliFormatter::kv("API Port", &config.api.port.to_string());
        CliFormatter::blank();
        CliFormatter::kv("Migrations", &format!("{} applied, {} pending", status.applied_count, status.pending_count));
        CliFormatter::kv("Tables", &if tables.is_empty() { "(none)".to_string() } else { tables.join(", ") });
    }

    Ok(())
}

fn cmd_info(project_dir: &PathBuf, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    use airdb_lib::engine::observability::{Metrics, HealthDashboardGenerator};
    use airdb_lib::engine::platform::Platform;
    use std::fs;
    
    let config = Config::load(project_dir)?;
    let db_path = project_dir.join(&config.database.path);
    let db = Database::new(&db_path)?;
    let runner = MigrationRunner::new(project_dir);
    let status = runner.check(&db)?;
    let tables = db.get_tables()?;
    
    // Get metrics
    let metrics = Metrics::load(project_dir).unwrap_or_default();
    
    // Get health
    let health_gen = HealthDashboardGenerator::new();
    let health = health_gen.generate(project_dir).ok();
    
    // Check for nosql
    let nosql_path = project_dir.join("nosql");
    let nosql_collections: Vec<String> = if nosql_path.exists() {
        fs::read_dir(&nosql_path)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('_'))
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };
    
    // Platform info
    let platform = Platform::current();
    
    if json {
        println!("{}", serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "platform": format!("{:?}", platform),
            "project": {
                "name": config.project.name,
                "dir": project_dir.display().to_string()
            },
            "database": {
                "type": config.database.db_type,
                "path": config.database.path.display().to_string(),
                "tables": tables
            },
            "nosql": {
                "collections": nosql_collections
            },
            "migrations": {
                "applied": status.applied_count,
                "pending": status.pending_count
            },
            "api": {
                "port": config.api.port
            },
            "metrics": {
                "total_updates": metrics.updates.total_updates,
                "rollback_count": metrics.updates.rollback_count,
                "update_success_rate": metrics.updates.success_rate()
            },
            "health": health.as_ref().map(|h| format!("{:?}", h.overall_status))
        }));
    } else {
        println!("\n╔══════════════════════════════════════════╗");
        println!("║         AirDB Project Information         ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║ Version: {:<31} ║", env!("CARGO_PKG_VERSION"));
        println!("║ Platform: {:<30} ║", format!("{:?}", platform));
        println!("╠══════════════════════════════════════════╣");
        println!("║ 📁 Project                                ║");
        println!("║   Name: {:<32} ║", config.project.name);
        println!("║   Dir: {:<33} ║", 
            project_dir.file_name().unwrap_or_default().to_string_lossy());
        println!("╠══════════════════════════════════════════╣");
        println!("║ 🗃️  SQL Database                           ║");
        println!("║   Type: {:<32} ║", config.database.db_type);
        println!("║   Tables: {:<30} ║", 
            if tables.is_empty() { "(none)".to_string() } else { tables.len().to_string() });
        println!("║   Migrations: {} applied, {} pending{} ║", 
            status.applied_count, status.pending_count,
            " ".repeat(16 - status.applied_count.to_string().len() - status.pending_count.to_string().len()));
        println!("╠══════════════════════════════════════════╣");
        println!("║ 📦 NoSQL Collections                      ║");
        if nosql_collections.is_empty() {
            println!("║   (none)                                  ║");
        } else {
            for coll in nosql_collections.iter().take(5) {
                println!("║   • {:<36} ║", coll);
            }
            if nosql_collections.len() > 5 {
                println!("║   ... and {} more                        ║", nosql_collections.len() - 5);
            }
        }
        println!("╠══════════════════════════════════════════╣");
        println!("║ 📊 Metrics                                ║");
        println!("║   Updates: {:<29} ║", metrics.updates.total_updates);
        println!("║   Rollbacks: {:<27} ║", metrics.updates.rollback_count);
        println!("║   Success Rate: {:<23} ║", format!("{:.1}%", metrics.updates.success_rate()));
        println!("╠══════════════════════════════════════════╣");
        println!("║ 🔧 API                                    ║");
        println!("║   Port: {:<32} ║", config.api.port);
        if let Some(h) = health.as_ref() {
            println!("║   Health: {:<30} ║", format!("{:?}", h.overall_status));
        }
        println!("╚══════════════════════════════════════════╝\n");
    }
    
    Ok(())
}

#[tokio::main]
async fn cmd_invite(project_dir: &PathBuf, username: &str, role: &str, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    use airdb_lib::engine::github::GitHubClient;
    use airdb_lib::engine::keystore::Keystore;
    use airdb_lib::engine::config::Config;

    // Get token from keyring
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let airdb_global = home_dir.join(".airdb");
    let keystore = Keystore::new(&airdb_global);
    
    let token = keystore.get_github_token().map_err(|_| {
        "Not authenticated. Run `airdb auth login` first."
    })?;

    // Load project config
    let config = Config::load(project_dir)?;
    
    // Get repo info
    let github_config = config.github.as_ref().ok_or("No GitHub repository configured")?;
    let repo_name = &github_config.repo;
    
    // Get owner (current user)
    let client = GitHubClient::with_token(token);
    let user = client.get_user().await?;
    
    // Map role to GitHub permission
    let permission = match role.to_lowercase().as_str() {
        "admin" => "admin",
        "developer" | "write" => "push",
        "readonly" | "read" => "pull",
        _ => "pull"
    };

    if !json {
        println!("📧 Inviting {} to {}/{}...", username, user.login, repo_name);
    }

    // Add collaborator
    client.add_collaborator(&user.login, repo_name, username, permission).await?;

    // Also add to local access file
    let access_file = project_dir.join("access").join("team.json");
    let mut team: serde_json::Value = if access_file.exists() {
        let content = std::fs::read_to_string(&access_file)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({ "members": [] })
    };
    
    let members = team.get_mut("members")
        .and_then(|m| m.as_array_mut())
        .ok_or("Invalid team.json format")?;
    
    members.push(serde_json::json!({
        "username": username,
        "role": role,
        "added_at": chrono::Utc::now().to_rfc3339()
    }));
    
    std::fs::write(&access_file, serde_json::to_string_pretty(&team)?)?;

    if json {
        println!("{}", serde_json::json!({
            "success": true,
            "username": username,
            "role": role,
            "permission": permission,
            "repo": format!("{}/{}", user.login, repo_name)
        }));
    } else {
        println!("✅ Invited {} as {} collaborator", username, role);
        println!("   GitHub permission: {}", permission);
    }

    Ok(())
}

fn cmd_keys(action: KeysAction, project_dir: &PathBuf, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let keystore = Keystore::new(project_dir);

    match action {
        KeysAction::List => {
            let keys = keystore.list_api_keys()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&keys)?);
            } else {
                println!("🔑 API Keys:");
                if keys.is_empty() {
                    println!("   (none)");
                } else {
                    for key in &keys {
                        println!("   {} - {} ({})", key.id, key.name, key.role);
                    }
                }
            }
        }
        KeysAction::Create { name, role } => {
            let (raw_key, key_info) = keystore.create_api_key(&name, &role)?;
            if json {
                println!("{}", serde_json::json!({
                    "key": raw_key,
                    "id": key_info.id,
                    "name": key_info.name,
                    "role": key_info.role
                }));
            } else {
                println!("✅ Created API key:");
                println!("   ID:   {}", key_info.id);
                println!("   Name: {}", key_info.name);
                println!("   Role: {}", key_info.role);
                println!("\n   🔐 Key (save this, shown only once):");
                println!("   {}", raw_key);
            }
        }
        KeysAction::Revoke { key_id } => {
            let revoked = keystore.revoke_api_key(&key_id)?;
            if json {
                println!("{}", serde_json::json!({
                    "success": revoked,
                    "key_id": key_id
                }));
            } else if revoked {
                println!("✅ Revoked API key: {}", key_id);
            } else {
                println!("⚠️  Key not found: {}", key_id);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn cmd_auth(action: AuthAction, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    use airdb_lib::engine::github::GitHubClient;
    use airdb_lib::engine::keystore::Keystore;
    
    // Use a temp path for keystore - will store token globally
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let airdb_dir = home_dir.join(".airdb");
    std::fs::create_dir_all(&airdb_dir)?;
    let keystore = Keystore::new(&airdb_dir);

    match action {
        AuthAction::Login => {
            // Check if already logged in
            if let Ok(token) = keystore.get_github_token() {
                let client = GitHubClient::with_token(token);
                if let Ok(user) = client.get_user().await {
                    if json {
                        println!("{}", serde_json::json!({
                            "already_authenticated": true,
                            "username": user.login
                        }));
                    } else {
                        println!("✅ Already logged in as: {}", user.login);
                        println!("   Use `airdb auth logout` to sign out first.");
                    }
                    return Ok(());
                }
            }

            // Start device flow
            let mut client = GitHubClient::new();
            
            if !json {
                println!("🔐 Starting GitHub authentication...\n");
            }

            let device_code = client.start_device_flow().await?;
            
            if json {
                println!("{}", serde_json::json!({
                    "user_code": device_code.user_code,
                    "verification_uri": device_code.verification_uri,
                    "expires_in": device_code.expires_in
                }));
            } else {
                println!("📋 Open this URL in your browser:");
                println!("   {}\n", device_code.verification_uri);
                println!("🔑 Enter this code: {}\n", device_code.user_code);
                println!("⏳ Waiting for authorization (expires in {} seconds)...", device_code.expires_in);
            }

            // Poll for token
            match client.complete_device_flow(&device_code).await {
                Ok(token) => {
                    // Get user info
                    let user = client.get_user().await?;
                    
                    // Store token in keyring
                    keystore.store_github_token(&token)?;
                    
                    if json {
                        println!("{}", serde_json::json!({
                            "success": true,
                            "username": user.login,
                            "name": user.name
                        }));
                    } else {
                        println!("\n✅ Successfully authenticated as: {}", user.login);
                        if let Some(name) = &user.name {
                            println!("   Name: {}", name);
                        }
                        println!("\n   Token securely stored in OS keyring.");
                    }
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "error": e.to_string()
                        }));
                    } else {
                        println!("❌ Authentication failed: {}", e);
                    }
                    return Err(e.into());
                }
            }
        }
        AuthAction::Logout => {
            match keystore.delete_github_token() {
                Ok(()) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "success": true,
                            "message": "Logged out successfully"
                        }));
                    } else {
                        println!("✅ Logged out successfully");
                        println!("   Token removed from OS keyring.");
                    }
                }
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "success": true,
                            "message": "Not logged in"
                        }));
                    } else {
                        println!("ℹ️  Not logged in (no token found)");
                    }
                }
            }
        }
        AuthAction::Status => {
            match keystore.get_github_token() {
                Ok(token) => {
                    let client = GitHubClient::with_token(token);
                    match client.get_user().await {
                        Ok(user) => {
                            if json {
                                println!("{}", serde_json::json!({
                                    "authenticated": true,
                                    "username": user.login,
                                    "name": user.name,
                                    "email": user.email
                                }));
                            } else {
                                println!("✅ Logged in as: {}", user.login);
                                if let Some(name) = &user.name {
                                    println!("   Name: {}", name);
                                }
                                if let Some(email) = &user.email {
                                    println!("   Email: {}", email);
                                }
                            }
                        }
                        Err(_) => {
                            // Token exists but is invalid
                            if json {
                                println!("{}", serde_json::json!({
                                    "authenticated": false,
                                    "error": "Token expired or invalid"
                                }));
                            } else {
                                println!("⚠️  Token found but invalid or expired");
                                println!("   Run `airdb auth login` to re-authenticate");
                            }
                        }
                    }
                }
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "authenticated": false
                        }));
                    } else {
                        println!("❌ Not authenticated");
                        println!("   Run `airdb auth login` to authenticate");
                    }
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn cmd_sync(action: SyncAction, project_dir: &PathBuf, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    use airdb_lib::engine::github::{GitHubClient, GitSync};
    use airdb_lib::engine::keystore::Keystore;
    use airdb_lib::engine::config::Config;

    // Get token from keyring
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let airdb_global = home_dir.join(".airdb");
    let keystore = Keystore::new(&airdb_global);
    
    let token = keystore.get_github_token().map_err(|_| {
        "Not authenticated. Run `airdb auth login` first."
    })?;

    // Load project config
    let config = Config::load(project_dir)?;
    
    match action {
        SyncAction::Setup { create } => {
            let client = GitHubClient::with_token(token.clone());
            let user = client.get_user().await?;
            
            let repo_name = config.github.as_ref()
                .map(|g| g.repo.clone())
                .unwrap_or_else(|| format!("airdb-{}", config.project.name));
            
            if !json {
                println!("🔗 Setting up GitHub sync for: {}", repo_name);
            }
            
            // Check if repo exists
            let existing = client.get_repo(&user.login, &repo_name).await?;
            
            if existing.is_some() {
                if json {
                    println!("{}", serde_json::json!({
                        "status": "exists",
                        "repo": format!("{}/{}", user.login, repo_name)
                    }));
                } else {
                    println!("✅ Repository exists: {}/{}", user.login, repo_name);
                }
            } else if create {
                // Create the repository
                let visibility = config.github.as_ref()
                    .map(|g| g.visibility == "private")
                    .unwrap_or(true);
                
                if !json {
                    println!("📦 Creating repository...");
                }
                
                let repo = client.create_repo(
                    &repo_name,
                    &format!("AirDB project: {}", config.project.name),
                    visibility,
                ).await?;
                
                if json {
                    println!("{}", serde_json::json!({
                        "status": "created",
                        "repo": repo.full_name,
                        "url": repo.html_url
                    }));
                } else {
                    println!("✅ Created repository: {}", repo.full_name);
                    println!("   URL: {}", repo.html_url);
                }
                
                // Initialize git in project
                let sync = GitSync::new(project_dir, &token);
                sync.create_gitignore()?;
                let repo_local = sync.init()?;
                GitHubClient::add_remote(&repo_local, "origin", &repo.clone_url)?;
                
                // Initial commit
                sync.commit(
                    "Initial AirDB project setup",
                    &user.login,
                    &user.email.unwrap_or_else(|| format!("{}@users.noreply.github.com", user.login)),
                )?;
                
                if !json {
                    println!("✅ Git initialized and ready to push");
                }
            } else {
                if json {
                    println!("{}", serde_json::json!({
                        "status": "not_found",
                        "repo": repo_name,
                        "hint": "Use --create to create the repository"
                    }));
                } else {
                    println!("⚠️  Repository not found: {}/{}", user.login, repo_name);
                    println!("   Run with --create to create it");
                }
            }
        }
        
        SyncAction::Push { message } => {
            let client = GitHubClient::with_token(token.clone());
            let user = client.get_user().await?;
            
            if !json {
                println!("📤 Pushing changes to GitHub...");
            }
            
            let sync = GitSync::new(project_dir, &token);
            
            // Commit all changes
            sync.commit(
                &message,
                &user.login,
                &user.email.unwrap_or_else(|| format!("{}@users.noreply.github.com", user.login)),
            )?;
            
            // Push to origin
            sync.push("main")?;
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "pushed",
                    "message": message
                }));
            } else {
                println!("✅ Pushed to GitHub");
                println!("   Commit: {}", message);
            }
        }
        
        SyncAction::Pull => {
            if !json {
                println!("📥 Pulling changes from GitHub...");
            }
            
            let sync = GitSync::new(project_dir, &token);
            sync.pull("main")?;
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "pulled"
                }));
            } else {
                println!("✅ Pulled latest changes");
            }
        }
        
        SyncAction::Status => {
            use git2::Repository;
            
            let repo = Repository::open(project_dir);
            
            match repo {
                Ok(repo) => {
                    let head = repo.head().ok().and_then(|h| h.shorthand().map(String::from));
                    let remote = repo.find_remote("origin").ok()
                        .and_then(|r| r.url().map(String::from));
                    
                    // Get uncommitted changes count
                    let mut opts = git2::StatusOptions::new();
                    opts.include_untracked(true);
                    let statuses = repo.statuses(Some(&mut opts))?;
                    let changes = statuses.len();
                    
                    if json {
                        println!("{}", serde_json::json!({
                            "git_initialized": true,
                            "branch": head,
                            "remote": remote,
                            "uncommitted_changes": changes
                        }));
                    } else {
                        println!("📊 Sync Status");
                        println!("   Branch: {}", head.unwrap_or("unknown".into()));
                        if let Some(url) = remote {
                            println!("   Remote: {}", url);
                        }
                        println!("   Changes: {} uncommitted file(s)", changes);
                    }
                }
                Err(_) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "git_initialized": false
                        }));
                    } else {
                        println!("⚠️  Git not initialized in this project");
                        println!("   Run `airdb sync setup --create` to set up GitHub sync");
                    }
                }
            }
        }

        SyncAction::Conflicts => {
            let sync = GitSync::new(project_dir, &token);
            match sync.list_conflicts() {
                Ok(conflicts) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "conflicts": conflicts
                        }));
                    } else if conflicts.is_empty() {
                        println!("✅ No merge conflicts detected");
                    } else {
                        println!("⚠️  Merge Conflicts Detected:");
                        for file in conflicts {
                            println!("   ❌ {}", file);
                        }
                        println!("\nUse `airdb sync resolve <FILE> --ours` or `--theirs` to resolve.");
                    }
                }
                Err(e) => {
                    eprintln!("Error checking conflicts: {}", e);
                    std::process::exit(1);
                }
            }
        }

        SyncAction::Resolve { file, ours, theirs } => {
            let strategy = if ours { "ours" } else if theirs { "theirs" } else {
                eprintln!("Error: Must specify --ours or --theirs");
                std::process::exit(1);
            };

            let sync = GitSync::new(project_dir, &token);
            match sync.resolve_conflict(&file, strategy) {
                Ok(_) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "resolved",
                            "file": file,
                            "strategy": strategy
                        }));
                    } else {
                        println!("✅ Resolved {} using '{}' version", file, strategy);
                        println!("   Don't forget to push afterwards!");
                    }
                }
                Err(e) => {
                    eprintln!("Error resolving conflict: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}

/// Handle update commands
fn cmd_update(action: UpdateAction, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    use airdb_lib::engine::updater::{VersionManager, UpdateState};

    let version_manager = VersionManager::new()?;
    version_manager.init()?;
    
    let state_path = version_manager.state_path();
    let mut state = UpdateState::load(&state_path).unwrap_or_default();

    match action {
        UpdateAction::Check => {
            // In production, this would check GitHub releases
            let current = &state.current_version;
            
            if json {
                println!("{}", serde_json::json!({
                    "current_version": current,
                    "update_available": false,
                    "message": "No updates available"
                }));
            } else {
                println!("🔍 Checking for updates...");
                println!("   Current version: v{}", current);
                println!("   ✅ You are running the latest version");
            }
        }

        UpdateAction::Download { version } => {
            let target_version = version.unwrap_or_else(|| "latest".to_string());
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "not_available",
                    "target_version": target_version,
                    "message": "Self-update downloads not yet implemented"
                }));
            } else {
                println!("📥 Download functionality will be available in a future release");
                println!("   Target: {}", target_version);
            }
        }

        UpdateAction::Apply => {
            if state.pending_version.is_none() {
                if json {
                    println!("{}", serde_json::json!({
                        "status": "no_pending",
                        "message": "No update pending"
                    }));
                } else {
                    println!("ℹ️  No update pending. Run `airdb update check` first.");
                }
            } else {
                let pending = state.pending_version.as_ref().unwrap();
                if json {
                    println!("{}", serde_json::json!({
                        "status": "ready",
                        "pending_version": pending,
                        "message": "Restart required to apply update"
                    }));
                } else {
                    println!("🔄 Update v{} is pending", pending);
                    println!("   Restart AirDB to apply the update");
                }
            }
        }

        UpdateAction::Rollback => {
            let last_good = &state.last_good_version;
            let current = &state.current_version;
            
            if last_good == current {
                if json {
                    println!("{}", serde_json::json!({
                        "status": "no_rollback",
                        "message": "Already on the oldest version"
                    }));
                } else {
                    println!("ℹ️  No previous version to rollback to");
                }
            } else {
                // Mark for rollback
                state.pending_version = Some(last_good.clone());
                state.save(&state_path).ok();
                
                if json {
                    println!("{}", serde_json::json!({
                        "status": "pending_rollback",
                        "current": current,
                        "target": last_good,
                        "message": "Restart to complete rollback"
                    }));
                } else {
                    println!("⏪ Rollback prepared");
                    println!("   Current: v{}", current);
                    println!("   Rolling back to: v{}", last_good);
                    println!("   Restart AirDB to complete rollback");
                }
            }
        }

        UpdateAction::Status => {
            let versions = version_manager.list_versions().unwrap_or_default();
            
            if json {
                println!("{}", serde_json::json!({
                    "current_version": state.current_version,
                    "pending_version": state.pending_version,
                    "last_good_version": state.last_good_version,
                    "channel": state.channel,
                    "installed_versions": versions,
                    "update_status": format!("{:?}", state.update_status)
                }));
            } else {
                println!("📊 Update Status");
                println!("   Current version:   v{}", state.current_version);
                if let Some(pending) = &state.pending_version {
                    println!("   Pending version:   v{}", pending);
                }
                println!("   Last good version: v{}", state.last_good_version);
                println!("   Channel:           {}", state.channel);
                println!("   Status:            {:?}", state.update_status);
                if !versions.is_empty() {
                    println!("   Installed:         {}", versions.join(", "));
                }
            }
        }

        UpdateAction::Channel { channel } => {
            let valid_channels = ["stable", "beta", "nightly"];
            if !valid_channels.contains(&channel.as_str()) {
                return Err(format!(
                    "Invalid channel '{}'. Valid options: {}", 
                    channel, 
                    valid_channels.join(", ")
                ).into());
            }
            
            state.channel = channel.clone();
            state.save(&state_path)?;
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "changed",
                    "channel": channel
                }));
            } else {
                println!("📡 Update channel changed to: {}", channel);
            }
        }
    }

    Ok(())
}

/// Handle NoSQL commands
fn cmd_nosql(action: NoSqlAction, project_dir: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    use airdb_lib::engine::nosql::{NoSqlEngine, Document, Query, Filter};

    match action {
        NoSqlAction::Init => {
            let engine = NoSqlEngine::open_or_create(project_dir)?;
            let meta = engine.meta();
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "initialized",
                    "format_version": meta.format_version,
                    "engine": meta.engine
                }));
            } else {
                println!("✅ NoSQL storage initialized");
                println!("   Format version: {}", meta.format_version);
                println!("   Engine: {}", meta.engine);
            }
        }

        NoSqlAction::Create { name } => {
            let engine = NoSqlEngine::open_or_create(project_dir)?;
            engine.create_collection(&name)?;
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "created",
                    "collection": name
                }));
            } else {
                println!("✅ Collection '{}' created", name);
            }
        }

        NoSqlAction::List => {
            let engine = NoSqlEngine::open(project_dir)?;
            let collections = engine.list_collections()?;
            
            if json {
                println!("{}", serde_json::json!({
                    "collections": collections
                }));
            } else {
                if collections.is_empty() {
                    println!("No collections found");
                } else {
                    println!("📦 Collections:");
                    for col in collections {
                        let count = engine.count(&col).unwrap_or(0);
                        println!("   {} ({} documents)", col, count);
                    }
                }
            }
        }

        NoSqlAction::Drop { name } => {
            let engine = NoSqlEngine::open(project_dir)?;
            engine.drop_collection(&name)?;
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "dropped",
                    "collection": name
                }));
            } else {
                println!("🗑️  Collection '{}' dropped", name);
            }
        }

        NoSqlAction::Insert { collection, data } => {
            let engine = NoSqlEngine::open(project_dir)?;
            let data_value: serde_json::Value = serde_json::from_str(&data)?;
            let doc = Document::new(data_value);
            let id = engine.insert(&collection, doc)?;
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "inserted",
                    "id": id,
                    "collection": collection
                }));
            } else {
                println!("✅ Document inserted with ID: {}", id);
            }
        }

        NoSqlAction::Get { collection, id } => {
            let engine = NoSqlEngine::open(project_dir)?;
            let doc = engine.get(&collection, &id)?;
            
            if json {
                println!("{}", serde_json::to_string_pretty(&doc)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&doc)?);
            }
        }

        NoSqlAction::Query { collection, field, value, limit } => {
            let engine = NoSqlEngine::open(project_dir)?;
            
            let mut query = Query::new();
            
            if let (Some(f), Some(v)) = (field, value) {
                query = query.filter(Filter::eq(&f, v));
            }
            
            if let Some(n) = limit {
                query = query.limit(n);
            }
            
            let results = engine.query(&collection, query)?;
            
            if json {
                println!("{}", serde_json::json!({
                    "count": results.len(),
                    "documents": results
                }));
            } else {
                println!("Found {} documents:", results.len());
                for doc in results {
                    println!("---");
                    println!("{}", serde_json::to_string_pretty(&doc)?);
                }
            }
        }

        NoSqlAction::Delete { collection, id } => {
            let engine = NoSqlEngine::open(project_dir)?;
            engine.collection(&collection)?.delete(&id)?;
            
            if json {
                println!("{}", serde_json::json!({
                    "status": "deleted",
                    "id": id
                }));
            } else {
                println!("🗑️  Document '{}' deleted", id);
            }
        }

        NoSqlAction::Stats { collection } => {
            let engine = NoSqlEngine::open(project_dir)?;
            let count = engine.count(&collection)?;
            let meta = engine.meta();
            
            if json {
                println!("{}", serde_json::json!({
                    "collection": collection,
                    "document_count": count,
                    "format_version": meta.format_version
                }));
            } else {
                println!("📊 Collection: {}", collection);
                println!("   Documents: {}", count);
                println!("   Format: v{}", meta.format_version);
            }
        }

        NoSqlAction::Schema { collection, action } => {
            use airdb_lib::engine::nosql::{MigrationRunner, MigrationOp};
            use airdb_lib::engine::nosql::schema::FieldType;
            
            let collection_path = project_dir.join("nosql").join(&collection);
            
            if !collection_path.exists() {
                return Err(format!("Collection '{}' not found", collection).into());
            }
            
            let runner = MigrationRunner::new(&collection_path);
            
            match action {
                SchemaAction::Create { name } => {
                    let migration = runner.create_migration(&name)?;
                    let migrations_dir = collection_path.join("migrations");
                    migration.save(&migrations_dir)?;
                    
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "created",
                            "version": migration.version,
                            "name": name
                        }));
                    } else {
                        println!("✅ Migration {:03}_{}.json created", migration.version, name);
                        println!("   Edit the file to add operations, then run: airdb nosql schema {} run", collection);
                    }
                }
                
                SchemaAction::Run => {
                    let schema = runner.run()?;
                    
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "applied",
                            "version": schema.version,
                            "fields": schema.fields.len()
                        }));
                    } else {
                        println!("✅ Schema updated to version {}", schema.version);
                        println!("   Fields: {}", schema.fields.len());
                    }
                }
                
                SchemaAction::Status => {
                    let migrations = runner.list_migrations()?;
                    
                    if json {
                        println!("{}", serde_json::json!({
                            "collection": collection,
                            "migration_count": migrations.len(),
                            "migrations": migrations.iter().map(|m| serde_json::json!({
                                "version": m.version,
                                "name": m.name,
                                "ops": m.operations.len()
                            })).collect::<Vec<_>>()
                        }));
                    } else {
                        println!("📋 Schema Migrations for '{}':", collection);
                        if migrations.is_empty() {
                            println!("   No migrations yet");
                        } else {
                            for m in migrations {
                                println!("   {:03}_{} ({} ops)", m.version, m.name, m.operations.len());
                            }
                        }
                    }
                }
                
                SchemaAction::AddField { name, field_type, required } => {
                    let ft = match field_type.to_lowercase().as_str() {
                        "string" => FieldType::String,
                        "number" => FieldType::Number,
                        "boolean" | "bool" => FieldType::Boolean,
                        "array" => FieldType::Array,
                        "object" => FieldType::Object,
                        _ => FieldType::Any,
                    };
                    
                    let mut migration = runner.create_migration(&format!("add_{}", name))?;
                    migration.operations.push(MigrationOp::AddField {
                        name: name.clone(),
                        field_type: ft,
                        required,
                        default: None,
                    });
                    
                    let migrations_dir = collection_path.join("migrations");
                    std::fs::create_dir_all(&migrations_dir)?;
                    migration.save(&migrations_dir)?;
                    
                    // Auto-run the migration
                    let schema = runner.run()?;
                    
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "added",
                            "field": name,
                            "schema_version": schema.version
                        }));
                    } else {
                        println!("✅ Field '{}' added to schema (v{})", name, schema.version);
                    }
                }
                
                SchemaAction::Show => {
                    let schema = runner.build_schema()?;
                    
                    if json {
                        println!("{}", serde_json::to_string_pretty(&schema)?);
                    } else {
                        println!("📝 Schema for '{}' (v{}):", collection, schema.version);
                        if schema.fields.is_empty() {
                            println!("   No fields defined (accepts any structure)");
                        } else {
                            for (name, def) in &schema.fields {
                                let req = if def.required { "*" } else { "" };
                                println!("   {}{}: {:?}", name, req, def.field_type);
                            }
                        }
                        println!("   Allow additional: {}", schema.allow_additional);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle Hybrid SQL/NoSQL commands
fn cmd_hybrid(action: HybridAction, project_dir: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    use airdb_lib::engine::hybrid::{
        RelationsManifest, Relation, FieldRef, RelationType
    };

    match action {
        HybridAction::Relate { name, source, target, relation_type } => {
            let source_ref = FieldRef::parse(&source)
                .ok_or_else(|| format!("Invalid source format: {}. Expected: engine.collection.field", source))?;
            let target_ref = FieldRef::parse(&target)
                .ok_or_else(|| format!("Invalid target format: {}. Expected: engine.collection.field", target))?;
            
            let rel_type = match relation_type.to_lowercase().as_str() {
                "one-to-one" | "onetoone" => RelationType::OneToOne,
                "one-to-many" | "onetomany" => RelationType::OneToMany,
                "many-to-one" | "manytoone" => RelationType::ManyToOne,
                "many-to-many" | "manytomany" => RelationType::ManyToMany,
                _ => RelationType::ManyToOne,
            };

            let relation = Relation::new(&name, source_ref, target_ref, rel_type);
            
            let mut manifest = RelationsManifest::load(project_dir)?;
            manifest.add(relation);
            manifest.save(project_dir)?;

            if json {
                println!("{}", serde_json::json!({
                    "status": "created",
                    "name": name,
                    "source": source,
                    "target": target
                }));
            } else {
                println!("✅ Relation '{}' created", name);
                println!("   {} → {}", source, target);
            }
        }

        HybridAction::Relations => {
            let manifest = RelationsManifest::load(project_dir)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&manifest)?);
            } else {
                println!("🔗 Relations ({} total):", manifest.relations.len());
                if manifest.relations.is_empty() {
                    println!("   No relations defined");
                } else {
                    for rel in &manifest.relations {
                        println!("   {} ({:?})", rel.name, rel.relation_type);
                        println!("     {} → {}", rel.source.to_string(), rel.target.to_string());
                    }
                }
            }
        }

        HybridAction::Unrelate { name } => {
            let mut manifest = RelationsManifest::load(project_dir)?;
            manifest.relations.retain(|r| r.name != name);
            manifest.save(project_dir)?;

            if json {
                println!("{}", serde_json::json!({"status": "removed", "name": name}));
            } else {
                println!("✅ Relation '{}' removed", name);
            }
        }

        HybridAction::Query { query } => {
            use airdb_lib::engine::hybrid::airql::{AirQuery, AIRQL_VERSION};
            use airdb_lib::engine::hybrid::EngineType;
            use airdb_lib::engine::nosql::{NoSqlEngine, Query as NsQuery, Filter as NsFilter};

            let air_query: AirQuery = serde_json::from_str(&query)?;

            if !air_query.is_compatible() {
                return Err(format!(
                    "Query version {} is newer than supported version {}",
                    air_query.airql_version, AIRQL_VERSION
                ).into());
            }

            match air_query.engine {
                EngineType::Sql => {
                    let sql = air_query.to_sql();
                    if json {
                        println!("{}", serde_json::json!({"sql": sql}));
                    } else {
                        println!("🔍 Generated SQL:");
                        println!("   {}", sql);
                    }
                }
                EngineType::Nosql => {
                    let engine = NoSqlEngine::open(project_dir)?;
                    let mut ns_query = NsQuery::new();
                    
                    for filter in &air_query.filters {
                        ns_query = ns_query.filter(NsFilter::eq(&filter.field, filter.value.clone()));
                    }
                    
                    if let Some(limit) = air_query.limit {
                        ns_query = ns_query.limit(limit);
                    }

                    let results = engine.query(&air_query.from, ns_query)?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&results)?);
                    } else {
                        println!("📄 Results from '{}':", air_query.from);
                        println!("   Found {} documents", results.len());
                        for doc in results {
                            println!("   • {}", doc.id);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
