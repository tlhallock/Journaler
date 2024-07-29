
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;

use journal_lib::service;
use journal_lib::views;


#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct NumberTemplate {
	pub default_value: Option<f64>, // TODO: rename to template value?
}

/*
impl Into<NumberField> for NumberFieldTemplate {
	fn into(self) -> Field {
		Field::Number(NumberField {
			name: self.name,
			value: self.default_value,
		})
	}
}
*/

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TextTemplate {
	pub default_value: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct BoolTemplate {
	pub default_value: Option<bool>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct EnumeratedTemplate {
	pub default_value: Option<views::EnumerationOption>,
	pub options: Vec<views::EnumerationOption>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub enum FieldValueTemplate {
	Number(NumberTemplate),
	Text(TextTemplate),
	Bool(BoolTemplate),
	Enumerated(EnumeratedTemplate),
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct FieldTemplate {
	pub name: String,
	pub label: String,
	pub value: FieldValueTemplate,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct EventTemplate {
	// Could just be stored as the key to the map
	pub event_template_uuid: Uuid,
	pub trace_template_uuid: Uuid,
	pub name: String,
	pub fields: Vec<FieldTemplate>,
	pub default_tags: Vec<String>,
	pub created_at: DateTime<Utc>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TraceFlowEntry {
	pub from: Uuid,
	pub to: Vec<Uuid>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TraceTemplate {
	pub trace_template_uuid: Uuid,
	pub project_uuid: Uuid,
	pub name: String,
	pub created_at: DateTime<Utc>,
	pub flow: Vec<TraceFlowEntry>,
}

impl FieldValueTemplate {
	fn get_initial(&self) -> service::FieldValueSuggestion {
		match self {
			// TODO move these into their own classes
			FieldValueTemplate::Number(template) => {
				service::FieldValueSuggestion::Number(service::NumberSuggestion {
					value: template.default_value,
					last_values: Vec::new(), // TODO
					default_value: template.default_value,
				})
			}
			FieldValueTemplate::Text(template) => {
				service::FieldValueSuggestion::Text(service::TextSuggestion {
					value: template.default_value.clone(),
					last_values: Vec::new(), // TODO
					default_value: template.default_value.clone(),
				})
			}
			FieldValueTemplate::Bool(template) => {
				service::FieldValueSuggestion::Bool(service::BoolSuggestion {
					value: template.default_value,
					last_values: Vec::new(), // TODO
					default_value: template.default_value,
				})
			}
			FieldValueTemplate::Enumerated(template) => {
				service::FieldValueSuggestion::Enumerated(service::EnumeratedSuggestion {
					selected: template.default_value.clone(),
					last_values: Vec::new(), // TODO
					options: template.options.clone(),
				})
			}
		}
	}
}

impl FieldTemplate {
	fn get_initial(&self) -> service::FieldSuggestion {
		service::FieldSuggestion {
			name: self.name.clone(),
			label: self.label.clone(),
			value: self.value.get_initial(),
		}
	}
}

impl EventTemplate {
	fn get_initial_fields(&self) -> Vec<service::FieldSuggestion> {
		self.fields
			.iter()
			.map(|field| field.get_initial())
			.collect()
	}

	fn get_default_tags(&self) -> Vec<String> {
		self.default_tags.clone()
	}

	pub fn create_builder(&self, trace: &views::TraceView) -> service::EventBuilder {
		service::EventBuilder {
			uuid: Uuid::new_v4(),
			event_template: service::RenameMe_EventTemplate {
				event_template_uuid: self.event_template_uuid,
				name: self.name.clone(),
			},
			fields: self.get_initial_fields(),
			tags: self.get_default_tags(),
			traces: Vec::new(),
			began_at: Utc::now(),
			selected_trace: service::TraceSelection::Selected(trace.into()),
		}
	}

	pub fn to_item(&self, last_used: Option<DateTime<Utc>>) -> views::EventTemplateItemView {
		views::EventTemplateItemView {
			event_template_uuid: self.event_template_uuid,
			name: self.name.clone(),
			created_at: self.created_at.clone(),
			last_used: last_used.unwrap_or(self.created_at.clone()),
		}
	}
}

impl TraceTemplate {
	pub fn to_item(&self, last_used: Option<DateTime<Utc>>) -> views::TraceTemplateItemView {
		views::TraceTemplateItemView {
			trace_template_uuid: self.trace_template_uuid,
			name: self.name.clone(),
			created_at: self.created_at.clone(),
			last_used,
		}
	}

	pub fn create_builder(&self) -> service::TraceBuilder {
		service::TraceBuilder {
			trace_uuid: Uuid::new_v4(),
			name: "".into(), // TODO
			trace_template: service::RenameMe_TraceTemplate {
				trace_template_uuid: self.trace_template_uuid,
				name: self.name.clone(),
			},
			tags: Vec::new(),
			began_at: Utc::now(),
			origin_traces: HashSet::new(),
			selected_trace: service::TraceSelection::None,
		}
	}

	fn get_current_event_template(&self, last_event_template_uuid: Option<Uuid>) -> Option<Uuid> {
		if let Some(uuid) = last_event_template_uuid {
			Some(uuid)
		} else {
			if let Some(entry) = self.flow.first() {
				Some(entry.from)
			} else {
				None
			}
		}
	}

	pub fn get_suggested_event_templates(
		&self,
		last_event_template_uuid: Option<Uuid>,
	) -> HashSet<Uuid> {
		self.get_current_event_template(last_event_template_uuid)
			.map_or(HashSet::new(), |current_event_template_uuid| {
				self.flow
					.iter()
					.filter(|entry| entry.from == current_event_template_uuid)
					.flat_map(|entry| entry.to.iter())
					.cloned()
					.collect()
			})
	}
}
