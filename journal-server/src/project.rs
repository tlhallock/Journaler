use chrono::serde::ts_milliseconds;
use uuid::Uuid;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Project {
	pub project_uuid: Uuid,
	pub name: String,

	#[serde(with = "ts_milliseconds")]
	pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Project {
	// pub fn parse_js(js: serde_json::Value) -> Result<Self, serde_json::Error> {
	// 	println!("The value is: {}", js);
	// 	for (key, value) in js.as_object().unwrap() {
	// 		println!("{}: {}", key, value);
	// 	}
	// 	serde_json::from_value(js)
	// }
}
