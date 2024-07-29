

use crate::events;

use chrono::Utc;
use uuid::Uuid;
use journal_lib::service;



pub fn build_trace(trace_builder: &service::TraceBuilder) -> events::Trace {
		events::Trace {
			trace_uuid: Uuid::new_v4(),
			trace_template_uuid: trace_builder.trace_template.trace_template_uuid,
			created_at: Utc::now(),
			name: trace_builder.name.clone(),
			completion: None,
			origin_trace_uuids: Vec::new(), // TODO
	}
}


	pub fn build_event(event_builder: &service::EventBuilder) -> events::Event {
		events::Event {
			event_uuid: Uuid::new_v4(),
			event_template_uuid: event_builder.event_template.event_template_uuid,
			trace_uuid: match &event_builder.selected_trace {
				service::TraceSelection::None => panic!("Event must have a trace"),
				service::TraceSelection::Selected(trace) => trace.trace_uuid,
			},
			fields: event_builder.fields.iter().flat_map(|f| build_field_suggestion(f)).collect(),
			tags: event_builder.tags.clone(),
			began_at: event_builder.began_at,
			created_at: Utc::now(),
		}
	}

pub fn build_field_value_suggestion(field_value_suggestion: &service::FieldValueSuggestion) -> Option<events::FieldValue> {
	match field_value_suggestion {
		service::FieldValueSuggestion::Number(suggestion) => suggestion
			.value
			.map(|value| events::FieldValue::Number(value)),
		// could remove this clone...
		service::FieldValueSuggestion::Text(suggestion) => suggestion
			.value
			.as_ref()
			.map(|value| events::FieldValue::Text(value.clone())),
			service::FieldValueSuggestion::Bool(suggestion) => suggestion
			.value
			.map(|value| events::FieldValue::Bool(value)),
			service::FieldValueSuggestion::Enumerated(suggestion) => suggestion
			.selected
			.clone()
			.map(|value| events::FieldValue::Enumerated(value)),
	}
}

pub fn build_field_suggestion(field_suggestion: &service::FieldSuggestion) -> Option<events::Field> {
	build_field_value_suggestion(&field_suggestion.value).map(|value| events::Field {
		name: field_suggestion.name.clone(),
		label: field_suggestion.label.clone(),
		value: Some(value), // Do we always want the keys anyway?
	})
}

