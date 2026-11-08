//! Crate based on the typesafe AI System One API
//! See <https://docs.typesafe.ai/api/system-one>

use serde::{Deserialize, Serialize};
use serde_json::{
    Map,
    Value::{self, Array, Object},
    json,
};

use crate::errors::Result;

use crate::response::TypeSafeResponse;

pub struct TypeSafeClient {
    url: String,
    token: String,
}

impl TypeSafeClient {
    pub fn build(url: &str, token: &str) -> Self {
        Self {
            url: url.to_string(),
            token: token.to_string(),
        }
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Debug, Default)]
pub struct TypeSafeRequestBuilder {
    request: TypeSafeRequest,
}

impl TypeSafeRequestBuilder {
    /// Create an empty TypeSafe request
    pub fn new() -> Self {
        Self::default()
    }

    /// Build into a TypeSafeRequest ready to be sent
    pub fn build(self) -> TypeSafeRequest {
        self.request
    }

    /// The model to use for this request
    pub fn model(mut self, model: &str) -> Self {
        self.request.model = model.into();
        self
    }

    /// The prompt
    pub fn state(mut self, state: &str) -> Self {
        self.request.state = state.into();
        self
    }

    /// A question type for selecting one option from a defined set. The answer includes the selected option, a probability for each option, and confidence.
    pub fn choice(
        mut self,
        name: &str,
        instructions: &str,
        criteria: Vec<TypeSafeCriteria>,
    ) -> Self {
        self.request
            .questions
            .insert(name.into(), Choice::build(instructions, criteria).into());
        self
    }

    /// A Score is a question type for rating content against ordered, descriptive levels. The answer includes a score, a probability for each level, and confidence.
    pub fn score(
        mut self,
        name: &str,
        instructions: &str,
        criteria: Vec<TypeSafeCriteria>,
    ) -> Self {
        self.request
            .questions
            .insert(name.into(), Score::build(instructions, criteria).into());
        self
    }

    /// A Noul question asks Jev to evaluate a yes/no question and return the probability that the answer is yes.
    pub fn noul(
        mut self,
        name: &str,
        instructions: &str,
        criteria: Option<Vec<TypeSafeCriteria>>,
    ) -> Self {
        self.request
            .questions
            .insert(name.into(), Noul::build(instructions, criteria).into());
        self
    }
}

#[derive(Debug, Default, Serialize)]
pub struct TypeSafeRequest {
    // the prompt
    state: String,
    // TODO: switch to enum
    model: String,
    // One request can have multiple question type, we store them as Map and it should be sent as an Object
    questions: Map<String, Value>,
}

impl TypeSafeRequest {
    pub async fn send(&self, typesafe_client: TypeSafeClient) -> Result<TypeSafeResponse> {
        let client = reqwest::Client::new();
        let token = format!("Bearer {}", typesafe_client.token());
        let res: TypeSafeResponse = client
            .post(typesafe_client.url())
            .header("Authorization", token)
            .json(self)
            .send()
            .await?
            .json::<TypeSafeResponse>()
            .await?;
        Ok(res)
    }
}

impl From<TypeSafeRequest> for Value {
    fn from(val: TypeSafeRequest) -> Self {
        json!({
            "state": &val.state,
            "model": &val.model,
            "questions": Object(val.questions),
        })
    }
}

#[macro_export]
macro_rules! criteria {
    // Criteria with key + value
    ($o:expr, $d:expr) => {
        TypeSafeCriteria {
            option: $o,
            description: $d,
        }
    };
    // Criteria with only value, for score
    ($d:expr) => {
        TypeSafeCriteria {
            // Use an empty string
            option: String::default(),
            description: $d,
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TypeSafeCriteria {
    option: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Choice {
    // Always "choice", per the typesafe documentation
    pub r#type: &'static str,
    pub instructions: String,
    /// The answer options, key is the option name, value
    /// is the description of that option
    pub criteria: Vec<TypeSafeCriteria>,
}

impl Choice {
    pub fn build(instructions: &str, criteria: Vec<TypeSafeCriteria>) -> Self {
        Self {
            r#type: "choice",
            instructions: instructions.to_string(),
            criteria,
        }
    }
}

impl From<Choice> for Value {
    /*
    Example:
    "department": {
      "type": "choice",
      "instructions": "Which team should handle this?",
      "criteria": {
        "returns": "Exchanges, wrong or damaged items",
        "shipping": "Delivery status, delays, lost packages",
        "billing": "Charges, invoices, payment problems"
        }
    }
    */
    fn from(val: Choice) -> Self {
        let criterias = {
            let mut map = Map::new();

            for crit in val.criteria {
                map.insert(crit.option, crit.description.into());
            }

            map
        };
        json!({
            "type": val.r#type,
            "instructions": val.instructions,
            "criteria": Object(criterias),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Noul {
    /// Always "noul", per the typesafe documentation
    pub r#type: &'static str,
    pub instructions: String,
    /// The answer options, key is the option name, value
    /// is the description of that option. This is optional
    pub criteria: Vec<TypeSafeCriteria>,
}

impl Noul {
    pub fn build(instructions: &str, criteria: Option<Vec<TypeSafeCriteria>>) -> Self {
        Self {
            r#type: "noul",
            instructions: instructions.into(),
            criteria: criteria.unwrap_or_default(),
        }
    }
}

impl From<Noul> for Value {
    fn from(val: Noul) -> Self {
        if val.criteria.is_empty() {
            // Criterias are optional
            json!({
                "type": val.r#type,
                "instructions": val.instructions
            })
        } else {
            let criterias = {
                let mut map = Map::new();

                for crit in val.criteria {
                    map.insert(crit.option, crit.description.into());
                }

                map
            };

            json!({
                "type": val.r#type,
                "instructions": val.instructions,
                "criteria": Object(criterias),
            })
        }
    }
}

/// A score is a question type for rating content against ordered, descriptive levels.
/// The answer includes a score, a probability for each level, and a confidence
///
/// A score answer is a position along your levels in `score`, which can fall between two levels.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Score {
    /// Always "score", per the typesafe documentation
    pub r#type: &'static str,
    pub instructions: String,
    /// The answer options, key is the option name, value
    /// is the description of that option. This is optional
    pub criteria: Vec<TypeSafeCriteria>,
}

impl Score {
    pub fn build(instructions: &str, criteria: Vec<TypeSafeCriteria>) -> Self {
        // !! Per the documentation, the maximum is 10 !!

        if criteria.len() > 10 {
            // TODO: error
            panic!()
        }

        Self {
            r#type: "score",
            instructions: instructions.into(),
            criteria,
        }
    }
}

impl From<Score> for Value {
    /*
    Example:
    "is_human_escalation": {
      "type": "noul",
      "instructions": "Is the customer asking for a human agent?"
    },
    "is_repeat_contact": {
      "type": "noul",
      "instructions": "Has the customer contacted support about this before?",
      "criteria": {
        "true": "Mentions a prior attempt, ticket, or that they have asked before",
        "false": "No sign of any previous contact"
      }
    }
    */
    fn from(val: Score) -> Self {
        if val.criteria.is_empty() {
            // Criterias are optional
            json!({
                "type": val.r#type,
                "instructions": val.instructions
            })
        } else {
            let criterias = {
                let mut vec: Vec<Value> = Vec::new();

                for crit in val.criteria {
                    vec.push(crit.description.into());
                }

                vec
            };

            json!({
                "type": val.r#type,
                "instructions": val.instructions,
                "criteria": Array(criterias),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn is_choice_parsed_into_correct_json() {
        let example = json!({
          "department": {
            "type": "choice",
            "instructions": "Which team should handle this?",
            "criteria": {
              "returns": "Exchanges, wrong or damaged items",
              "shipping": "Delivery status, delays, lost packages",
              "billing": "Charges, invoices, payment problems"
            }
          }
        });

        // Build our value

        let name = "department";

        let instructions = "Which team should handle this?";

        let criterias = [
            criteria!(
                "returns".to_string(),
                "Exchanges, wrong or damaged items".to_string()
            ),
            criteria!(
                "shipping".to_string(),
                "Delivery status, delays, lost packages".to_string()
            ),
            criteria!(
                "billing".to_string(),
                "Charges, invoices, payment problems".to_string()
            ),
        ];

        let choice = super::Choice::build(instructions, criterias.into());

        let question = {
            let mut map = Map::new();
            map.insert(name.to_string(), choice.into());
            Object(map)
        };

        assert_eq!(question, example)
    }

    #[test]
    fn is_json_parsed_into_question() {}

    #[test]
    fn is_noul_parsed_into_correct_json() {
        let example = json!({
        "is_human_escalation": {
          "type": "noul",
          "instructions": "Is the customer asking for a human agent?"
        },
        });

        // Build our value

        let name = "is_human_escalation";

        let instructions = "Is the customer asking for a human agent?";

        let noul = super::Noul::build(instructions, None);

        let question = {
            let mut map = Map::new();
            map.insert(name.to_string(), noul.into());
            Object(map)
        };

        assert_eq!(question, example)
    }

    #[test]
    fn is_noul_with_optional_parsed_into_correct_json() {
        let example = json!({
        "is_repeat_contact": {
          "type": "noul",
          "instructions": "Has the customer contacted support about this before?",
          "criteria": {
            "true": "Mentions a prior attempt, ticket, or that they have asked before",
            "false": "No sign of any previous contact"
          }
        }
            });

        // Build our value

        let name = "is_repeat_contact";

        let instructions = "Has the customer contacted support about this before?";

        let criterias = [
            criteria!(
                "true".to_string(),
                "Mentions a prior attempt, ticket, or that they have asked before".to_string()
            ),
            criteria!(
                "false".to_string(),
                "No sign of any previous contact".to_string()
            ),
        ];

        let noul = super::Noul::build(instructions, Some(criterias.into()));

        let question = {
            let mut map = Map::new();
            map.insert(name.to_string(), noul.into());
            Object(map)
        };

        assert_eq!(question, example)
    }

    #[test]
    fn is_score_parsed_into_correct_json() {
        let example = json!({
        "bug_severity": {
          "type": "score",
          "instructions": "How severe is the reported issue?",
          "criteria": [
            "Cosmetic; no impact to functionality",
            "Broken or degraded feature, but workaround exists",
            "Blocking issue; no workaround exists"
          ]
        }
        });

        // Build our value

        let name = "bug_severity";

        let instructions = "How severe is the reported issue?";

        let criterias = [
            criteria!("Cosmetic; no impact to functionality".to_string()),
            criteria!("Broken or degraded feature, but workaround exists".to_string()),
            criteria!("Blocking issue; no workaround exists".to_string()),
        ];

        let score = super::Score::build(instructions, criterias.into());

        let question = {
            let mut map = Map::new();
            map.insert(name.to_string(), score.into());
            Object(map)
        };

        assert_eq!(question, example)
    }

    #[test]
    fn builder_into_request_parse_correct_json() {
        let request = TypeSafeRequestBuilder::new()
            .model("jev-latest")
            .state("My running shoes arrived in the wrong size. Can I swap them for a size 10?")
            .choice(
                "department",
                "Which team should handle this?",
                [
                    criteria!(
                        "returns".to_string(),
                        "Exchanges, wrong or damaged items".to_string()
                    ),
                    criteria!(
                        "shipping".to_string(),
                        "Delivery status, delays, lost packages".to_string()
                    ),
                    criteria!(
                        "billing".to_string(),
                        "Charges, invoices, payment problems".to_string()
                    ),
                ]
                .into(),
            )
            .build();

        let json_example = json!({
          "state": "My running shoes arrived in the wrong size. Can I swap them for a size 10?",
          "model": "jev-latest",
          "questions": {
            "department": {
              "type": "choice",
              "instructions": "Which team should handle this?",
              "criteria": {
                "returns": "Exchanges, wrong or damaged items",
                "shipping": "Delivery status, delays, lost packages",
                "billing": "Charges, invoices, payment problems"
              }
            }
          }
        });

        let request_json: Value = request.into();

        assert_eq!(request_json, json_example)
    }
}
