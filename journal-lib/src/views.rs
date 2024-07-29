use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;
use chrono::serde::ts_milliseconds;


#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TraceCompletion {
	#[serde(with = "ts_milliseconds")]
	pub completed_at: DateTime<Utc>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct NumberValueView {
	pub value: Option<f64>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TextValueView {
	pub value: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct BoolValueView {
	pub value: Option<bool>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct EnumerationOptionView {
	pub label: String,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub enum FieldValueView {
	Number(NumberValueView),
	Text(TextValueView),
	Bool(BoolValueView),
	Enumerated(EnumerationOptionView),
}

// This could change in the future, to hold recent values, etc. 
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct FieldView {
	pub name: String,
	pub label: String,

	// TODO: option of option?
	pub value: Option<FieldValueView>,
}


#[derive(Clone)]
pub struct ProjectItemView {}

#[derive(Clone)]
pub struct ProjectView {}

#[derive(Clone)]
pub struct EventTemplateItemView {
	pub event_template_uuid: Uuid,
	pub name: String,
	pub created_at: DateTime<Utc>,
	pub last_used: DateTime<Utc>,
	// TODO
	// project_name
	// trace template
}

#[derive(Clone)]
pub struct EventTemplateView {
	// project name
	// trace template
}

#[derive(Clone)]
pub struct TraceTemplateItemView {
	pub trace_template_uuid: Uuid,
	pub name: String,
	pub created_at: DateTime<Utc>,
	pub last_used: Option<DateTime<Utc>>,
	// pub default_state: String,
}

#[derive(Clone)]
pub struct TraceTemplateView {
	pub trace_template_uuid: Uuid,
	pub name: String,
	// TODO
	// event templates
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraceItemView {
	pub trace_uuid: Uuid,
	pub name: String,
	pub template_name: Option<String>,
	// TODO:
	// last used
}

#[derive(Clone)]
pub struct TraceView {
	pub trace_uuid: Uuid,
	pub name: String,
	pub trace_template: Option<TraceTemplateItemView>,
	pub tags: Vec<String>,
	pub completion: Option<TraceCompletion>,

	pub last_event: Option<EventItemView>,
	pub suggested_event_templates: Vec<EventTemplateItemView>,
	pub other_event_templates: Vec<EventTemplateItemView>,
	// TODO
	// in ui: option to complete
	// last used
}

impl Into<TraceItemView> for &TraceView {
	fn into(self) -> TraceItemView {
		TraceItemView {
			trace_uuid: self.trace_uuid,
			name: self.name.clone(),
			template_name: self.trace_template.as_ref().map(|t| t.name.clone()),
		}
	}
}

#[derive(Clone)]
pub struct EventItemView {
	pub event_uuid: Uuid,
	pub event_template: Option<EventTemplateItemView>,
	pub created_at: DateTime<Utc>,
	pub trace_name: Option<String>,
	// created at
	// trace name
	// event template name
}

#[derive(Clone)]
pub struct EventView {
	// rename to event_uuid
	pub event_uuid: Uuid,
	pub event_template: Option<EventTemplateItemView>,
	pub trace: Option<TraceItemView>,
	pub fields: Vec<FieldView>,
	pub tags: Vec<String>,

	pub began_at: DateTime<Utc>,
	pub created_at: DateTime<Utc>,
	// TODO
	// trace template
	// project
}

/*

fn list_event_templates(&self, project_uuid: Option<Uuid>) -> Vec<views::EventTemplateItemView>;
fn view_event_template(&self, template_uuid: Uuid) -> Option<views::EventTemplate>;

fn list_trace_templates(&self, project_uuid: Option<Uuid>) -> Vec<views::TraceTemplateItemView>;
fn view_trace_templates(&self, trace_uuid: Uuid) -> Option<views::TraceTemplateView>;

fn list_traces(&self, project_uuid: Option<Uuid>) -> Vec<views::TraceItemView>;
fn view_trace(&self, trace_uuid: Uuid) -> Option<views::TraceView>;

fn create_trace(&self, trace_template_uuid: Uuid, name: Option<String>) -> TraceBuilder;
fn save_trace(&self, trace_builder: &TraceBuilder);

fn list_events(&self, project_uuid: Option<Uuid>) -> Vec<views::EventItemView>;
fn view_event(&self, event_uuid: Uuid) -> Option<views::EventView>;

fn create_event(&self, template_uuid: Uuid) -> EventBuilder;
fn save_event(&self, event_builder: &EventBuilder);

*/

impl TraceView {
	// pub fn get_current_state(&self) -> String {
	// 	if let Some(event) = &self.last_event {
	// 		return format!(
	// 			"Last event: {}",
	// 			event.event_template.as_ref().unwrap().name
	// 		);
	// 	}
	// 	if let Some(trace_template) = &self.trace_template {
	// 		return trace_template.default_state.clone();
	// 	}
	// 	"Missing Trace template".into()
	// }
}


#[derive(PartialEq, Eq, Clone, serde::Deserialize, serde::Serialize)]
pub struct EnumerationOption {
	pub name: String,
	pub label: String,
}
