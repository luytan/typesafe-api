use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TypeSafeResponse {
    /// Model that answered
    pub model: String,
    /// One answer per question, keyed by the question key from the request
    pub answers: HashMap<String, Answer>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    Choice(ChoiceResponse),
    Noul(NoulResponse),
    Score(ScoreResponse),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoulResponse {
    /// Probability that the statement is true, from 0 to 1
    pub noul: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChoiceResponse {
    /// The option with the highest probability
    pub choice: String,
    /// Confidence in the selection, from 0 to 1
    pub confidence: f64,
    /// Probability of each option, keyed by option name
    pub probabilities: HashMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoreResponse {
    /// Average of the options
    pub score: f64,
    /// Confidence in the score, from 0 to 1
    pub confidence: f64,
    /// Each level number mapped back to its description
    pub legend: HashMap<String, Value>,
    /// Probability of each level, keyed like `legend`
    pub probabilities: HashMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn response_deserializes_into_typed_answers() {
        let example = json!({
            "model": "jev-1.14.0",
            "answers": {
                "is_spam": {
                    "type": "noul",
                    "noul": 0.98
                },
                "department": {
                    "type": "choice",
                    "choice": "billing",
                    "probabilities": { "returns": 0.05, "shipping": 0.1, "billing": 0.85 },
                    "confidence": 0.9
                },
                "urgency": {
                    "type": "score",
                    "score": 2.0,
                    "legend": { "0": "Can wait", "1": "This week", "2": "Today" },
                    "probabilities": { "0": 0.0, "1": 0.0, "2": 1.0 },
                    "confidence": 1.0
                },
            },
            "usage": { "input_tokens": 120, "output_tokens": 12 }
        });

        let response: TypeSafeResponse = serde_json::from_value(example).unwrap();

        assert_eq!(response.model, "jev-1.14.0");
        assert_eq!(response.usage.input_tokens, 120);
        assert_eq!(response.usage.output_tokens, 12);

        match &response.answers["is_spam"] {
            Answer::Noul(noul) => assert_eq!(noul.noul, 0.98),
            other => panic!("expected noul answer, got {other:?}"),
        }

        match &response.answers["department"] {
            Answer::Choice(choice) => {
                assert_eq!(choice.choice, "billing");
                assert_eq!(choice.probabilities["shipping"], 0.1);
                assert_eq!(choice.confidence, 0.9);
            }
            other => panic!("expected choice answer, got {other:?}"),
        }

        match &response.answers["urgency"] {
            Answer::Score(score) => {
                assert_eq!(score.score, 2.0);
                assert_eq!(score.legend["2"], json!("Today"));
            }
            other => panic!("expected score answer, got {other:?}"),
        }
    }
}
