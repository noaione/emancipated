use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    height: u32,
}

/// Error response schema for GraphQL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponseError {
    pub errors: Vec<GraphQLError>,
}

impl std::fmt::Display for GraphQLResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.errors.is_empty() {
            writeln!(f, "No errors found")?;
        } else if self.errors.len() == 1 {
            writeln!(f, "{}", self.errors[0])?;
        } else {
            // print with numbering
            for (i, error) in self.errors.iter().enumerate() {
                writeln!(f, "[{}] {}", i + 1, error)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    pub locations: Vec<GraphQLErrorLocation>,
    pub path: Vec<serde_json::Value>,
}

impl std::fmt::Display for GraphQLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Error: {msg} (at {line}, {column}[..] in {path})
        write!(f, "Error: {}", self.message)?;
        if !self.locations.is_empty() {
            write!(
                f,
                " (at {})",
                self.locations
                    .iter()
                    .map(|loc| format!("{}:{}", loc.line, loc.column))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
        }
        if !self.path.is_empty() {
            // stringify path and write it
            let path_str = self
                .path
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(".");
            write!(f, " in {}", path_str)?;
        }
        // newline
        writeln!(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLErrorLocation {
    pub line: u32,
    pub column: u32,
}
