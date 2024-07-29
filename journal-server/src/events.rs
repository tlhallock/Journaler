
use journal_lib::views;
use chrono::serde::ts_milliseconds;
use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;


#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct NumberField {
	pub name: String,
	pub value: Option<f64>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TextField {
	pub name: String,
	pub value: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct BoolField {
	pub name: String,
	pub value: Option<bool>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct EnumeratedField {
	pub name: String,
	pub value: Option<views::EnumerationOption>,
}


#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub enum FieldValue {
	Number(f64),
	Text(String),
	Bool(bool),
	Enumerated(views::EnumerationOption),
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Field {
	pub name: String,
	pub label: String,
	pub value: Option<FieldValue>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Trace {
	pub trace_uuid: Uuid,
	pub trace_template_uuid: Uuid,
	pub origin_trace_uuids: Vec<Uuid>,
	#[serde(with = "ts_milliseconds")]
	pub created_at: DateTime<Utc>,
	pub name: String,
	pub completion: Option<views::TraceCompletion>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Event {
	pub event_uuid: Uuid,
	pub event_template_uuid: Uuid,
	pub trace_uuid: Uuid,
	pub fields: Vec<Field>,
	pub tags: Vec<String>,

	#[serde(with = "ts_milliseconds")]
	pub began_at: DateTime<Utc>,
	#[serde(with = "ts_milliseconds")]
	pub created_at: DateTime<Utc>,
}

impl Trace {
	pub fn to_item(&self, template_name: Option<String>) -> views::TraceItemView {
		views::TraceItemView {
			trace_uuid: self.trace_uuid,
			name: self.name.clone(),
			template_name,
		}
	}

	// TODO: ugly to have to pass the service here
	// pub fn to_view(&self, service: &fs_service::FileSystemEventsService) -> views::TraceView {
	// 	views::TraceView {
	// 		trace_uuid: self.trace_uuid,
	// 		name: self.name.clone(),
	// 		template: None,   // TODO
	// 		tags: vec![],     // TODO
	// 		completion: None, // TODO

	// 		last_event: service.get_last_event_for_trace(self.trace_uuid),
	// 		suggested_event_templates: service.get_suggested_event_templates_for_trace(self.trace_uuid),

	// pub last_event: Option<EventItemView>,
	// pub suggested_event_templates: Vec<EventTemplateItemView>,
	// pub other_event_templates: Vec<EventTemplateItemView>,
	// 	}
	// }
}


impl Field {
	pub fn to_view(&self) -> views::FieldView {
		views::FieldView {
			name: self.name.clone(),
			label: self.label.clone(),
			value: self.value.as_ref().map(
				|value| match value {
					FieldValue::Number(n) => views::FieldValueView::Number(views::NumberValueView { value: Some(*n) }),
					FieldValue::Text(s) => views::FieldValueView::Text(views::TextValueView { value: Some(s.to_string()) }),
					FieldValue::Bool(b) => views::FieldValueView::Bool(views::BoolValueView { value: Some(*b) }),
					FieldValue::Enumerated(e) => views::FieldValueView::Enumerated(
						views::EnumerationOptionView { label: e.label.clone() },
					),
				},
			),
		}
	}

}