//! AirDB CLI Module
//! Command-line interface for AirDB operations

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "airdb")]
#[command(author = "AirDB Team")]
#[command(version)]
#[command(about = "Local-first, GitHub-backed database platform", long_about = None)]
pub struct Cli {
    /// Project directory (defaults to current directory)
    #[arg(short, long, global = true)]
    pub project: Option<PathBuf>,

    /// Output format (json for scripting)
    #[arg(short, long, global = true, default_value = "text")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new AirDB project
    Init {
        /// Project name
        #[arg(short, long)]
        name: String,

        /// GitHub repository visibility
        #[arg(short, long, default_value = "private")]
        visibility: String,

        /// Skip GitHub integration
        #[arg(long)]
        no_github: bool,
    },

    /// Migration commands
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },

    /// Start local API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "54321")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Show project status
    Status,

    /// Invite a collaborator
    Invite {
        /// GitHub username to invite
        username: String,

        /// Role to assign
        #[arg(short, long, default_value = "developer")]
        role: String,
    },

    /// API key management
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },

    /// GitHub authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// GitHub repository sync
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },

    /// Self-update management
    Update {
        #[command(subcommand)]
        action: UpdateAction,
    },

    /// NoSQL database operations
    Nosql {
        #[command(subcommand)]
        action: NoSqlAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum MigrateAction {
    /// Create a new migration
    Create {
        /// Migration name
        name: String,
    },

    /// Run pending migrations and push to GitHub
    Push,

    /// Check migration status
    Check,

    /// Rollback last N migrations
    Rollback {
        /// Number of migrations to rollback
        #[arg(default_value = "1")]
        count: usize,
    },

    /// List migration history
    List,
}

#[derive(Subcommand, Debug)]
pub enum KeysAction {
    /// List all API keys
    List,

    /// Create a new API key
    Create {
        /// Key name/description
        #[arg(short, long)]
        name: String,

        /// Role for the key
        #[arg(short, long, default_value = "readonly")]
        role: String,
    },

    /// Revoke an API key
    Revoke {
        /// Key ID to revoke
        key_id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthAction {
    /// Login with GitHub
    Login,

    /// Logout from GitHub
    Logout,

    /// Show current auth status
    Status,
}

#[derive(Subcommand, Debug)]
pub enum SyncAction {
    /// Initialize GitHub repo for this project
    Setup {
        /// Create new repo if it doesn't exist
        #[arg(long)]
        create: bool,
    },

    /// Push local changes to GitHub
    Push {
        /// Commit message
        #[arg(short, long, default_value = "Update schema")]
        message: String,
    },

    /// Pull changes from GitHub  
    Pull,

    /// Show sync status
    Status,

    /// List conflicted files
    Conflicts,

    /// Resolve conflicts
    Resolve {
        /// File to resolve
        file: String,

        /// Use local version
        #[arg(long, group = "strategy")]
        ours: bool,

        /// Use remote version
        #[arg(long, group = "strategy")]
        theirs: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum UpdateAction {
    /// Check for available updates
    Check,

    /// Download and prepare an update
    Download {
        /// Specific version to download
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Apply a pending update (requires restart)
    Apply,

    /// Rollback to the previous version
    Rollback,

    /// Show current update status
    Status,

    /// Set update channel
    Channel {
        /// Channel name (stable, beta, nightly)
        channel: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum NoSqlAction {
    /// Initialize NoSQL storage for this project
    Init,

    /// Create a new collection
    Create {
        /// Collection name
        name: String,
    },

    /// List all collections
    List,

    /// Drop a collection
    Drop {
        /// Collection name
        name: String,
    },

    /// Insert a document
    Insert {
        /// Collection name
        collection: String,
        
        /// JSON data
        data: String,
    },

    /// Get a document by ID
    Get {
        /// Collection name
        collection: String,
        
        /// Document ID
        id: String,
    },

    /// Query documents
    Query {
        /// Collection name
        collection: String,
        
        /// Filter field
        #[arg(short, long)]
        field: Option<String>,
        
        /// Filter value (equality)
        #[arg(short = 'v', long)]
        value: Option<String>,
        
        /// Limit results
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Delete a document
    Delete {
        /// Collection name
        collection: String,
        
        /// Document ID
        id: String,
    },

    /// Show collection stats
    Stats {
        /// Collection name
        collection: String,
    },

    /// Schema migration management
    Schema {
        /// Collection name
        collection: String,
        
        #[command(subcommand)]
        action: SchemaAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum SchemaAction {
    /// Create a new migration
    Create {
        /// Migration name (e.g., add_email_field)
        name: String,
    },

    /// Run pending migrations
    Run,

    /// Show migration status
    Status,

    /// Add a field via migration
    AddField {
        /// Field name
        name: String,
        
        /// Field type (string, number, boolean, array, object, any)
        #[arg(short = 't', long, default_value = "string")]
        field_type: String,
        
        /// Make field required
        #[arg(short, long)]
        required: bool,
    },

    /// Show current schema
    Show,
}

impl Cli {
    pub fn get_project_dir(&self) -> PathBuf {
        self.project
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}
