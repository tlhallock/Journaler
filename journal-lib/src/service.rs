
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;

use crate::errors;
use crate::views;


#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct NumberSuggestion {
	pub value: Option<f64>,
	pub last_values: Vec<f64>,      // TODO: Could be a set of recent values?
	pub default_value: Option<f64>, // TODO: rename to template value?
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TextSuggestion {
	pub value: Option<String>,
	pub last_values: Vec<String>,
	pub default_value: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct BoolSuggestion {
	pub value: Option<bool>,
	pub last_values: Vec<bool>,
	pub default_value: Option<bool>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct EnumeratedSuggestion {
	pub selected: Option<views::EnumerationOption>,
	pub last_values: Vec<views::EnumerationOption>,
	pub options: Vec<views::EnumerationOption>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub enum FieldValueSuggestion {
	Number(NumberSuggestion),
	Text(TextSuggestion),
	Bool(BoolSuggestion),
	Enumerated(EnumeratedSuggestion),
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct FieldSuggestion {
	pub name: String,
	pub label: String,
	pub value: FieldValueSuggestion,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TraceSelection {
	None,
	Selected(views::TraceItemView),
}

impl TraceSelection {
	pub fn label(&self) -> String {
		match self {
			TraceSelection::None => "None".to_string(),
			TraceSelection::Selected(trace) => trace.name.clone(),
		}
	}
}

pub struct RenameMe_EventTemplate {
	pub event_template_uuid: Uuid,
	pub name: String,
}

pub struct EventBuilder {
	pub uuid: Uuid,
	pub event_template: RenameMe_EventTemplate,
	// pub event_template: definition::EventTemplate,
	pub fields: Vec<FieldSuggestion>,
	pub tags: Vec<String>,
	pub traces: Vec<Uuid>,
	pub began_at: DateTime<Utc>,
	pub selected_trace: TraceSelection,
}

pub struct RenameMe_TraceTemplate {
	pub trace_template_uuid: Uuid,
	pub name: String,
}

pub struct TraceBuilder {
	pub trace_uuid: Uuid,
	pub name: String,
	pub trace_template: RenameMe_TraceTemplate,
	pub tags: Vec<String>,
	pub began_at: DateTime<Utc>,
	pub origin_traces: HashSet<views::TraceItemView>,
	pub selected_trace: TraceSelection,
}


pub trait EventsService {
	fn list_event_templates(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::EventTemplateItemView>>;

	fn create_event(
		&self,
		trace: &views::TraceView,
		template_uuid: Uuid,
	) -> Result<EventBuilder, errors::EventTemplateNotFound>;
	
	fn create_trace(
		&self,
		trace_template_uuid: Uuid,
	) -> Result<TraceBuilder, errors::TraceTemplateNotFound>;

	fn complete_trace(&mut self, trace_uuid: Uuid);

	fn save_event(&mut self, event_builder: &EventBuilder);

	fn save_trace(&mut self, trace_builder: &TraceBuilder);

	fn view_event(&self, event_uuid: Uuid) -> Option<views::EventView>;

	fn view_trace(&self, trace_uuid: Uuid) -> Option<views::TraceView>;

	fn list_events(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::EventItemView> + '_>;

	fn list_traces(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::TraceItemView> + '_>;

	fn list_trace_templates(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::TraceTemplateItemView>>;


	fn load_from_disk(&mut self) -> Result<(), std::io::Error>;
	fn save_to_disk(&self) -> Result<(), std::io::Error>;
	fn import_definition(
		&mut self,
		project_name: String,
		definition_path: &std::path::PathBuf
	) -> Result<(), errors::ParsingError>;

	/*
	fn import_all_projects(&self, json: String);
	fn export_all_projects(&self) -> String;

	fn list_projects(&self) -> Box<dyn Iterator<Item = views::ProjectItemView>>;
	fn view_project(&self) -> Box<dyn Iterator<Item = views::ProjectView>>;

	// fn create_project(&self, name: String) -> views::ProjectView;

	fn list_event_templates(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::EventTemplateItemView>>;
	fn view_event_template(&self, template_uuid: Uuid) -> Option<views::EventTemplateView>;

	fn list_trace_templates(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::TraceTemplateItemView>>;
	fn view_trace_templates(&self, trace_uuid: Uuid) -> Option<views::TraceTemplateView>;

	fn list_traces(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::TraceItemView>>;
	fn view_trace(&self, trace_uuid: Uuid) -> Option<views::TraceView>;

	fn create_trace(&self, trace_template_uuid: Uuid, name: Option<String>) -> TraceBuilder;
	fn save_trace(&self, trace_builder: &TraceBuilder);

	fn list_events(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::EventItemView>>;
	fn view_event(&self, event_uuid: Uuid) -> Option<views::EventView>;

	fn create_event(
		&self,
		template_uuid: Uuid,
	) -> Result<EventBuilder, errors::EventTemplateNotFound>;
	fn save_event(&self, event_builder: &EventBuilder);

	// pub fn create_event(&self, template_uuid: Uuid) -> EventBuilder;
	// pub fn save_event(&mut self, event_builder: &EventBuilder);
	// pub fn view_event_from_uuid(&self, event_uuid: Uuid) -> Option<EventView>;
	// pub fn view_event(&self, event: &Event) -> EventView;
	// pub fn list_event_schemas(&self) -> Vec<EventTemplateEntry>;
	// pub fn list_events(&self) -> Vec<EventView>;

	*/
}


impl TraceBuilder {
	pub fn label(&self) -> String {
		self.name.clone()
	}
}