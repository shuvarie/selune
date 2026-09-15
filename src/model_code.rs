use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelCodeError {
    #[error("invalid format")]
    InvalidFormat,
    #[error("invalid character: {0}")]
    InvalidCharacter(char),
}

#[derive(Debug)]
pub struct ModelCode {
    pub org_id: String,
    pub model_id: String,
    pub variant: Option<String>,
}

impl ModelCode {
    pub fn new(org_id: &str, model_id: &str, variant: Option<&str>) -> Self {
        Self {
            org_id: org_id.to_string(),
            model_id: model_id.to_string(),
            variant: variant.map(|s| s.to_string()),
        }
    }
}

impl FromStr for ModelCode {
    type Err = ModelCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut org_id = String::new();
        let mut model_id = String::new();
        let mut variant: Option<String> = None;

        let mut part = 0;
        let mut buf = String::new();

        for ch in s.chars() {
            match ch {
                '/' if part == 0 => {
                    part = 1;
                    org_id.push_str(&buf);
                    buf.clear();
                }
                ':' if part == 1 => {
                    part = 2;
                    model_id.push_str(&buf);
                    buf.clear();
                }
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => {
                    buf.push(ch);
                }
                other => return Err(ModelCodeError::InvalidCharacter(other)),
            }
        }

        if buf.len() > 0 {
            match part {
                0 => org_id = buf,
                1 => model_id = buf,
                2 => variant = Some(buf),
                _ => unreachable!(),
            }
        }

        if org_id.len() == 0 || model_id.len() == 0 {
            return Err(ModelCodeError::InvalidFormat);
        }

        Ok(Self { org_id, model_id, variant })
    }
}

impl Display for ModelCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.org_id, self.model_id)?;
        if let Some(ref variant) = self.variant {
            write!(f, ":{variant}")?;
        }
        Ok(())
    }
}
