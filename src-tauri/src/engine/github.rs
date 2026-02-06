//! AirDB GitHub Integration Module
//! OAuth device flow, repository management, and Git operations

use git2::{Cred, RemoteCallbacks, Repository, Signature};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_API_URL: &str = "https://api.github.com";

// AirDB OAuth App Client ID
const DEFAULT_CLIENT_ID: &str = "Ov23liIxu7QChssFXhpr";

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("OAuth error: {0}")]
    OAuthError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Not authenticated")]
    NotAuthenticated,
    #[error("Authorization pending - user has not completed login")]
    AuthorizationPending,
    #[error("Authorization expired")]
    AuthorizationExpired,
    #[error("Slow down - polling too fast")]
    SlowDown,
    #[error("Merge conflicts detected in: {0}")]
    Conflict(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateRepoRequest {
    name: String,
    description: String,
    private: bool,
    auto_init: bool,
}

#[derive(Debug, Serialize)]
struct AddCollaboratorRequest {
    permission: String,
}

/// Device code response from GitHub
#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Token response from GitHub
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

pub struct GitHubClient {
    token: Option<String>,
    client_id: String,
    http_client: reqwest::Client,
}

impl GitHubClient {
    pub fn new() -> Self {
        Self {
            token: None,
            client_id: DEFAULT_CLIENT_ID.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_token(token: String) -> Self {
        Self {
            token: Some(token),
            client_id: DEFAULT_CLIENT_ID.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_client_id(client_id: String) -> Self {
        Self {
            token: None,
            client_id,
            http_client: reqwest::Client::new(),
        }
    }

    /// Start the device code OAuth flow
    /// Returns device code info that user needs to complete in browser
    pub async fn start_device_flow(&self) -> Result<DeviceCodeResponse, GitHubError> {
        let response = self.http_client
            .post(GITHUB_DEVICE_CODE_URL)
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("scope", "repo read:org read:user"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GitHubError::OAuthError(format!(
                "Failed to start device flow: {}",
                error_text
            )));
        }

        let device_code: DeviceCodeResponse = response.json().await?;
        Ok(device_code)
    }

    /// Poll for the access token after user completes device flow
    pub async fn poll_for_token(&mut self, device_code: &str, _interval: u64) -> Result<String, GitHubError> {
        let response = self.http_client
            .post(GITHUB_TOKEN_URL)
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?;

        let token_response: TokenResponse = response.json().await?;

        if let Some(error) = token_response.error {
            return match error.as_str() {
                "authorization_pending" => Err(GitHubError::AuthorizationPending),
                "slow_down" => Err(GitHubError::SlowDown),
                "expired_token" => Err(GitHubError::AuthorizationExpired),
                "access_denied" => Err(GitHubError::OAuthError("User denied access".to_string())),
                _ => Err(GitHubError::OAuthError(
                    token_response.error_description.unwrap_or(error)
                )),
            };
        }

        if let Some(access_token) = token_response.access_token {
            self.token = Some(access_token.clone());
            return Ok(access_token);
        }

        Err(GitHubError::OAuthError("No access token in response".to_string()))
    }

    /// Complete device flow with polling
    pub async fn complete_device_flow(&mut self, device_code: &DeviceCodeResponse) -> Result<String, GitHubError> {
        let interval = Duration::from_secs(device_code.interval.max(5));
        let max_attempts = (device_code.expires_in / device_code.interval) as usize;

        for _ in 0..max_attempts {
            tokio::time::sleep(interval).await;
            
            match self.poll_for_token(&device_code.device_code, device_code.interval).await {
                Ok(token) => return Ok(token),
                Err(GitHubError::AuthorizationPending) => continue,
                Err(GitHubError::SlowDown) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(GitHubError::AuthorizationExpired)
    }

    pub fn create_oauth_client(
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<BasicClient, GitHubError> {
        let auth_url = AuthUrl::new(GITHUB_AUTH_URL.to_string())
            .map_err(|e| GitHubError::OAuthError(e.to_string()))?;
        let token_url = TokenUrl::new(GITHUB_TOKEN_URL.to_string())
            .map_err(|e| GitHubError::OAuthError(e.to_string()))?;
        let redirect_url = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| GitHubError::OAuthError(e.to_string()))?;

        let client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url);

        Ok(client)
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    fn get_token(&self) -> Result<&str, GitHubError> {
        self.token.as_deref().ok_or(GitHubError::NotAuthenticated)
    }

    pub async fn get_user(&self) -> Result<GitHubUser, GitHubError> {
        let token = self.get_token()?;
        
        let response = self.http_client
            .get(format!("{}/user", GITHUB_API_URL))
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(USER_AGENT, "AirDB")
            .header(ACCEPT, "application/vnd.github+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError(format!(
                "Failed to get user: {}",
                response.status()
            )));
        }

        let user: GitHubUser = response.json().await?;
        Ok(user)
    }

    pub async fn get_repo(&self, owner: &str, name: &str) -> Result<Option<GitHubRepo>, GitHubError> {
        let token = self.get_token()?;

        let response = self.http_client
            .get(format!("{}/repos/{}/{}", GITHUB_API_URL, owner, name))
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(USER_AGENT, "AirDB")
            .header(ACCEPT, "application/vnd.github+json")
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(GitHubError::ApiError(format!(
                "Failed to get repo: {}",
                response.status()
            )));
        }

        let repo: GitHubRepo = response.json().await?;
        Ok(Some(repo))
    }

    pub async fn create_repo(
        &self,
        name: &str,
        description: &str,
        private: bool,
    ) -> Result<GitHubRepo, GitHubError> {
        let token = self.get_token()?;

        let request = CreateRepoRequest {
            name: name.to_string(),
            description: description.to_string(),
            private,
            auto_init: true,
        };

        let response = self.http_client
            .post(format!("{}/user/repos", GITHUB_API_URL))
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(USER_AGENT, "AirDB")
            .header(ACCEPT, "application/vnd.github+json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError(format!(
                "Failed to create repo: {}",
                error_text
            )));
        }

        let repo: GitHubRepo = response.json().await?;
        Ok(repo)
    }

    pub async fn add_collaborator(
        &self,
        owner: &str,
        repo: &str,
        username: &str,
        permission: &str,
    ) -> Result<(), GitHubError> {
        let token = self.get_token()?;

        let request = AddCollaboratorRequest {
            permission: permission.to_string(),
        };

        let response = self.http_client
            .put(format!(
                "{}/repos/{}/{}/collaborators/{}",
                GITHUB_API_URL, owner, repo, username
            ))
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .header(USER_AGENT, "AirDB")
            .header(ACCEPT, "application/vnd.github+json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError(format!(
                "Failed to add collaborator: {}",
                error_text
            )));
        }

        Ok(())
    }

    pub fn clone_repo(url: &str, path: &Path, token: &str) -> Result<Repository, GitHubError> {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("oauth2", token)
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        let repo = builder.clone(url, path)?;
        Ok(repo)
    }

    pub fn init_repo(path: &Path) -> Result<Repository, GitHubError> {
        let repo = Repository::init(path)?;
        Ok(repo)
    }

    pub fn add_remote(repo: &Repository, name: &str, url: &str) -> Result<(), GitHubError> {
        repo.remote(name, url)?;
        Ok(())
    }

    pub fn commit_all(
        repo_path: &Path,
        message: &str,
        author_name: &str,
        author_email: &str,
    ) -> Result<git2::Oid, GitHubError> {
        let repo = Repository::open(repo_path)?;
        
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let signature = Signature::now(author_name, author_email)?;

        // Check if there's a parent commit
        let parent_commit = match repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None,
        };

        let commit_id = if let Some(parent) = parent_commit {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent],
            )?
        } else {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[],
            )?
        };

        Ok(commit_id)
    }

    pub fn push(repo_path: &Path, token: &str, branch: &str) -> Result<(), GitHubError> {
        let repo = Repository::open(repo_path)?;
        
        let mut remote = repo.find_remote("origin")?;
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("oauth2", token)
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        remote.push(&[&refspec], Some(&mut push_options))?;

        Ok(())
    }

    pub fn pull(repo_path: &Path, token: &str, branch: &str) -> Result<(), GitHubError> {
        let repo = Repository::open(repo_path)?;
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("oauth2", token)
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[branch], Some(&mut fetch_options), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        
        let (analysis, _preference) = repo.merge_analysis(&[&fetch_commit])?;
        
        if analysis.is_fast_forward() {
            let refname = format!("refs/heads/{}", branch);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        } else if analysis.is_normal() {
            let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
            let our_commit = repo.find_commit(head_commit.id())?;
            let their_commit = repo.find_commit(fetch_commit.id())?;
            
            let mut index = repo.merge_commits(&our_commit, &their_commit, None)?;
            
            if index.has_conflicts() {
                // List conflicted files
                let conflicts = index.conflicts()?;
                let mut files = Vec::new();
                for conflict in conflicts {
                    if let Ok(c) = conflict {
                        if let Some(entry) = c.our {
                            files.push(String::from_utf8_lossy(&entry.path).to_string());
                        } else if let Some(entry) = c.their {
                            files.push(String::from_utf8_lossy(&entry.path).to_string());
                        }
                    }
                }
                repo.checkout_index(Some(&mut index), Some(git2::build::CheckoutBuilder::default().allow_conflicts(true)))?;
                return Err(GitHubError::Conflict(files.join(", ")));
            } else {
                // Auto-merge successful, create commit
                let signature = repo.signature()?;
                let tree_id = index.write_tree_to(&repo)?;
                let tree = repo.find_tree(tree_id)?;
                
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    "Merge remote-tracking branch 'origin/main'",
                    &tree,
                    &[&repo.head()?.peel_to_commit()?, &repo.find_commit(fetch_commit.id())?],
                )?;
                repo.checkout_head(None)?;
            }
        }

        Ok(())
    }

    pub fn commit_and_push(
        repo_path: &Path,
        message: &str,
        token: &str,
    ) -> Result<(), GitHubError> {
        let repo = Repository::open(repo_path)?;
        
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let head = repo.head()?;
        let parent_commit = head.peel_to_commit()?;

        let signature = repo.signature()?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        // Push to remote
        let mut remote = repo.find_remote("origin")?;
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("oauth2", token)
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options))?;

        Ok(())
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Sync operations for AirDB projects
pub struct GitSync {
    project_dir: std::path::PathBuf,
    token: String,
}

impl GitSync {
    pub fn new(project_dir: &Path, token: &str) -> Self {
        Self {
            project_dir: project_dir.to_path_buf(),
            token: token.to_string(),
        }
    }

    /// Initialize a new Git repository for the project
    pub fn init(&self) -> Result<Repository, GitHubError> {
        GitHubClient::init_repo(&self.project_dir)
    }

    /// Create .gitignore with AirDB defaults
    pub fn create_gitignore(&self) -> Result<(), std::io::Error> {
        let gitignore = r#"# AirDB
data/*.db
data/*.db-*
.airdb/keys.json
*.log
.DS_Store
"#;
        std::fs::write(self.project_dir.join(".gitignore"), gitignore)?;
        Ok(())
    }

    /// Commit all changes with a message
    pub fn commit(&self, message: &str, author_name: &str, author_email: &str) -> Result<git2::Oid, GitHubError> {
        GitHubClient::commit_all(&self.project_dir, message, author_name, author_email)
    }

    /// Push to remote origin
    pub fn push(&self, branch: &str) -> Result<(), GitHubError> {
        GitHubClient::push(&self.project_dir, &self.token, branch)
    }

    /// Pull from remote origin
    pub fn pull(&self, branch: &str) -> Result<(), GitHubError> {
        GitHubClient::pull(&self.project_dir, &self.token, branch)
    }

    /// List all files currently in conflict
    pub fn list_conflicts(&self) -> Result<Vec<String>, GitHubError> {
        let repo = Repository::open(&self.project_dir)?;
        let index = repo.index()?;
        
        let conflicts = index.conflicts()?;
        let mut files = Vec::new();
        
        for conflict in conflicts {
            if let Ok(c) = conflict {
                if let Some(entry) = c.our {
                    files.push(String::from_utf8_lossy(&entry.path).to_string());
                } else if let Some(entry) = c.their {
                    files.push(String::from_utf8_lossy(&entry.path).to_string());
                }
            }
        }
        
        // Deduplicate
        files.sort();
        files.dedup();
        
        Ok(files)
    }

    /// Resolve a conflict by choosing a strategy
    pub fn resolve_conflict(&self, file_path: &str, strategy: &str) -> Result<(), GitHubError> {
        let repo = Repository::open(&self.project_dir)?;
        let mut index = repo.index()?;
        
        // Find the conflict entry
        let path = Path::new(file_path);
        
        // Stage 2 (ours) or Stage 3 (theirs)
        let stage = match strategy {
            "ours" => 2,
            "theirs" => 3,
            _ => return Err(GitHubError::GitError(git2::Error::from_str("Invalid resolution strategy"))),
        };

        if let Some(entry) = index.get_path(path, stage) {
            let oid = entry.id;
            let mode = entry.mode;
            
            // Remove conflict entries
            index.remove_path(path)?;
            
            // Add the chosen version
            let new_entry = git2::IndexEntry {
                path: path.as_os_str().as_encoded_bytes().to_vec(),
                id: oid,
                mode,
                ..entry
            };
            index.add(&new_entry)?;
            index.write()?;
            
            // Checkout the file to working directory
            repo.checkout_index(Some(&mut index), Some(git2::build::CheckoutBuilder::default().path(file_path).force()))?;
            
            Ok(())
        } else {
            Err(GitHubError::GitError(git2::Error::from_str("Conflict entry not found for specified strategy")))
        }
    }
}

