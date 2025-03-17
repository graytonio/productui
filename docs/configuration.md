# Configuration Documentation

This document provides a detailed explanation of each configuration field in the provided TOML file.

## Fields

### `github_token`
- **Description**: This is the GitHub token used to authenticate API requests to GitHub.
- **Usage**: The token must be a valid GitHub access token that has the appropriate permissions for the operations you want to perform (e.g., accessing repositories, creating issues, etc.).
- **Required**: Yes

### `labels`
- **Description**: This is an array of labels used to filter issues or pull requests.
- **Usage**: Each label in this array will be used to filter issues or pull requests from the specified repositories.
- **Example**: 
  ```toml
  labels = [
      "urgent",
      "bug"
  ]
  ```
- **Required**: Yes

### `repos`
- **Description**: This is an array of GitHub repositories to monitor or interact with.
- **Usage**: Each repository is specified as a two-element array, where the first element is the repository owner (username) and the second element is the repository name.
- **Example**:
  ```toml
  repos = [
      ["graytonio", "productui"],
      ["user2", "another-repo"]
  ]
  ```
- **Required**: Yes

### `filters`
- **Description**: This is an array of event types or filters used to further refine the selection criteria for issues or pull requests.
- **Usage**: These filters are applied when interacting with GitHub's API. Common filters include:
  - `"ReviewRequested"`: Issues that have a review requested.
  - `"Mentions"`: Issues that mention the user or team.
  - `"Labels"`: Issues with specific labels (see `labels` array).
  - `"Assigned"`: Issues assigned to the user.
  - `"Created"`: Issues created by the user.
- **Example**:
  ```toml
  filters = [
      "ReviewRequested",
      "Mentions"
  ]
  ```
- **Required**: Yes

## Usage Notes

1. **GitHub Token**: Make sure that your GitHub token has the necessary permissions for the operations you want to perform (e.g., `repo:status`, `public_repo`).
2. **Labels**: You can define as many labels as needed, but they should be relevant and specific to your workflow.
3. **Repositories**: Ensure that the repositories specified exist and are accessible with the given token.
4. **Filters**: Use these filters to narrow down the issues or pull requests you want to process.

## Example Configuration

Here is an example of a complete configuration:

```toml
[github]
github_token = "your-github-token-here"
labels = [
    "urgent",
    "bug"
]
repos = [
    ["graytonio", "productui"],
    ["user2", "another-repo"]
]
filters = [
    "ReviewRequested",
    "Mentions",
    "Labels",
    "Assigned"
]
```

## Notes

- This configuration is used to customize the behavior of a tool that interacts with GitHub's API.
- The specific fields and their usage depend on the requirements of your project or workflow.
