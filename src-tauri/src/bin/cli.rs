//! AirDB CLI - Main entry point for CLI binary
//! 
//! This binary provides the `airdb` CLI tool for managing projects.

use airdb_lib::engine::{
    cli::{Cli, Commands, MigrateAction, KeysAction, AuthAction, SyncAction, OutputFormat},
    config::Config,
    database::Database,
    migrations::MigrationRunner,
    keystore::Keystore,
    api::{ApiState, create_router},
};
use clap::Parser;
use std::path::PathBuf;
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
        println!("📊 AirDB Project Status");
        println!("   Project: {}", config.project.name);
        println!("   Database: {} ({})", config.database.db_type, config.database.path.display());
        println!("   API Port: {}", config.api.port);
        println!("\n   Migrations: {} applied, {} pending", status.applied_count, status.pending_count);
        println!("   Tables: {}", if tables.is_empty() { "(none)".to_string() } else { tables.join(", ") });
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
