use crate::args::ServerArgs;

use super::Repository;

pub mod delete_messages;
pub mod download;
pub mod get_messages;

pub async fn render_table(
    repo: &dyn Repository,
    query: SearchParams,
    args: &ServerArgs,
) -> anyhow::Result<actix_web::web::Html> {
    let username = query.username.clone().filter(|s| !s.is_empty());
    let messages = repo
        .get_messages(username, query.offset, query.limit)
        .await?;

    let mut tera = tera::Tera::default();
    tera.add_raw_template("index.html", TEMPLATE)?;

    let mut context = tera::Context::new();
    context.insert("messages", &messages);
    context.insert("last_query", &query);
    context.insert("docs_enabled", &!args.web.disable_docs);
    let result = tera.render("index.html", &context)?;

    Ok(actix_web::web::Html::new(result))
}

use std::num::NonZeroUsize;

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::IntoParams)]
pub struct SearchParams {
    pub username: Option<String>,
    // `utoipa` doesn't handle non zero types yet
    #[param(default = 20, value_type = usize, minimum = 1)]
    #[serde(default = "get_default_limit")]
    pub limit: NonZeroUsize,
    #[param(default = 0)]
    #[serde(default)]
    pub offset: usize,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            username: None,
            limit: get_default_limit(),
            offset: 0,
        }
    }
}

fn get_default_limit() -> NonZeroUsize {
    NonZeroUsize::new(20).unwrap()
}

const TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Messages</title>
    <style>
        table {
            width: 100%;
            border-collapse: collapse;
        }

        th, td {
            border: 1px solid #dddddd;
            padding: 8px;
            text-align: left;
        }

        th {
            background-color: #f2f2f2;
        }
        
        .hash {
            font-family: monospace;
            max-width: 100px; 
            overflow: hidden;
            text-overflow: ellipsis;
        }
    </style>
</head>
<body>
    <h1>Messages ({{ messages | length }})</h1>
    {% if docs_enabled %}
        <a href="/_docs/redoc">See API documentation</a>
    {% endif %}
    <form action="/" method="get">
        <label for="username">Username:</label>
        <input
            type="text"
            id="username"
            name="username"
            {% if last_query.username %} value="{{ last_query.username }}" {% endif %}
        >

        <label for="limit">Limit:</label>
        <input type="number" id="limit" name="limit" min="1" value="{{ last_query.limit }}">

        <label for="offset">Offset:</label>
        <input type="number" id="offset" name="offset" min="0" value="{{ last_query.offset }}">

        <button type="submit">Search</button>
    </form>
    <table>
        <thead>
            <tr>
                <th>Timestamp</th>
                <th>User</th>
                <th>IP</th>
                <th>Message</th>
                <th>Filename</th>
                <th>Filesize</th>
                <th>Mime</th>
                <th>SHA256</th>
                <th>Filelink</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody>
            {% for message in messages %}
            <tr>
                <td>{{ message.0.timestamp }}</td>
                <td>{{ message.0.user_nickname }}</td>
                <td>{{ message.0.user_ip }}</td>
                <td>
                    {% if message.1 %}
                        {{ message.1.text }}
                    {% endif %}
                </td>
                <td>
                    {% if message.2 %}
                        {{ message.2.filename }}
                    {% endif %}
                </td>
                <td>
                    {% if message.2 %}
                        {{ message.2.length | filesizeformat }}
                    {% endif %}
                </td>
                <td>
                    {% if message.2 %}
                        {{ message.2.mime }}
                    {% endif %}
                </td>
                {% if message.2 %}
                    <td class="hash" title="{{ message.2.hash }}">
                        {{ message.2.hash }}
                    </td>
                {% else %}
                    <td></td>
                {% endif %}
                <td>
                    {% if message.2 %}
                        <a href="/download/{{ message.0.public_id }}" target="_blank">Download</a>
                    {% endif %}
                </td>
                <td>
                    <form action="delete" method="post">
                        <input type="hidden" name="id" value="{{ message.0.public_id }}">
                        <button type="submit">Delete</button>
                    </form>
                </td>
            </tr>
            {% endfor %}
        </tbody>
    </table>
</body>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_defaults() {
        let params = serde_json::from_str::<SearchParams>("{}").unwrap();

        assert_eq!(params.username, None);
        assert_eq!(params.limit.get(), get_default_limit().get());
        assert_eq!(params.limit.get(), 20);
        assert_eq!(params.offset, 0);
    }
}
