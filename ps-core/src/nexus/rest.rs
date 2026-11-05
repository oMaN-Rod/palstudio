use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Account {
    pub user_id: u64,
    pub name: String,
    pub is_premium: bool,
    pub is_supporter: bool,
    pub profile_url: Option<String>,
}

#[derive(Deserialize)]
struct Claims {
    user: ClaimedUser,
}

#[derive(Deserialize)]
struct ClaimedUser {
    id: Value,
    username: String,
    #[serde(default)]
    membership_roles: Vec<String>,
    #[serde(default)]
    avatar: Option<String>,
}

/// Reads the account from the claims of a Nexus Mods OAuth access token.
pub fn account_from_claims(claims: Value) -> Result<Account, serde_json::Error> {
    let user = serde_json::from_value::<Claims>(claims)?.user;
    let user_id = match &user.id {
        Value::Number(id) => id.as_u64(),
        Value::String(id) => id.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| serde::de::Error::custom("`user.id` is not a user id"))?;
    let has_role = |role: &str| user.membership_roles.iter().any(|held| held == role);
    Ok(Account {
        user_id,
        is_premium: has_role("premium"),
        is_supporter: has_role("supporter"),
        name: user.username,
        profile_url: user.avatar,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Category {
    pub category_id: u32,
    pub name: String,
    pub parent_category: Option<u32>,
}

#[derive(Deserialize)]
struct CategoryFields {
    category_id: u32,
    name: String,
    #[serde(default)]
    parent_category: Value,
}

pub fn categories_from_game(mut game: Value) -> Result<Vec<Category>, serde_json::Error> {
    let list = game
        .get_mut("categories")
        .map(Value::take)
        .unwrap_or(Value::Array(Vec::new()));
    let fields: Vec<CategoryFields> = serde_json::from_value(list)?;
    Ok(fields
        .into_iter()
        .map(|field| Category {
            category_id: field.category_id,
            name: field.name,
            parent_category: field
                .parent_category
                .as_u64()
                .and_then(|parent| u32::try_from(parent).ok()),
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DownloadLink {
    pub name: String,
    pub short_name: String,
    #[serde(rename = "URI")]
    pub uri: String,
}

pub fn error_message(body: &[u8]) -> Option<String> {
    serde_json::from_slice::<Value>(body)
        .ok()?
        .get("message")?
        .as_str()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn claims_give_the_id_name_and_membership_roles() {
        let premium = account_from_claims(json!({ "sub": "7", "user": {
            "id": 7, "username": "Tester", "email": "t@example.invalid",
            "avatar": "https://example.invalid/u/7.png",
            "membership_roles": ["member", "supporter", "premium"]
        } }))
        .unwrap();
        assert_eq!((premium.user_id, premium.name.as_str()), (7, "Tester"));
        assert!(premium.is_premium && premium.is_supporter);
        assert!(!serde_json::to_string(&premium)
            .unwrap()
            .contains("t@example.invalid"));
        let member =
            account_from_claims(json!({ "user": { "id": "8", "username": "Free" } })).unwrap();
        assert_eq!(member.user_id, 8);
        assert!(!member.is_premium && !member.is_supporter);
        assert!(account_from_claims(json!({ "user": { "id": "x", "username": "X" } })).is_err());
        assert!(account_from_claims(json!({ "sub": "7" })).is_err());
    }

    #[test]
    fn categories_treat_false_as_no_parent() {
        let categories = categories_from_game(json!({ "categories": [
            { "category_id": 1, "name": "Palworld", "parent_category": false },
            { "category_id": 10, "name": "Pals", "parent_category": 1 },
            { "category_id": 11, "name": "Odd" }
        ] }))
        .unwrap();
        assert_eq!(categories[0].parent_category, None);
        assert_eq!(categories[1].parent_category, Some(1));
        assert_eq!(categories[2].parent_category, None);
        assert!(categories_from_game(json!({})).unwrap().is_empty());
    }

    #[test]
    fn download_links_and_error_messages_parse() {
        let links: Vec<DownloadLink> = serde_json::from_value(json!([
            { "name": "Nexus CDN", "short_name": "Nexus CDN", "URI": "https://cf-files.example.invalid/a.zip?md5=x" }
        ]))
        .unwrap();
        assert_eq!(links[0].uri, "https://cf-files.example.invalid/a.zip?md5=x");
        assert_eq!(
            error_message(br#"{"message":"Please provide a valid API Key"}"#).as_deref(),
            Some("Please provide a valid API Key")
        );
        assert_eq!(error_message(b"<html>"), None);
    }
}
