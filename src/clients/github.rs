use octocrab::Octocrab;

pub fn get_github_client(token: &Option<String>) -> octocrab::Result<Octocrab> {
    let mut client = Octocrab::builder();

    match token {
        Some(token) => client = client.personal_token(token.clone()),
        _ => (),
    }
    
    client.build()
}