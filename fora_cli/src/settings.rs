use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct EnvironmentVariable {
    name: String,
    value: String,
}

impl FromStr for EnvironmentVariable {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Support name=value
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid environment variable format: '{}'. Expected format is 'NAME=VALUE'",
                s
            ));
        }
        Ok(EnvironmentVariable {
            name: parts[0].to_string(),
            value: parts[1].to_string(),
        })
    }
}
