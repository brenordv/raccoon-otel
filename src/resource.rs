use std::collections::HashMap;

use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

/// Build an OpenTelemetry [`Resource`] with the service name and optional attributes.
pub(crate) fn build_resource(service_name: &str, attributes: &HashMap<String, String>) -> Resource {
    let mut kvs: Vec<KeyValue> = Vec::with_capacity(attributes.len() + 1);
    kvs.push(KeyValue::new("service.name", service_name.to_owned()));

    for (key, value) in attributes {
        kvs.push(KeyValue::new(key.clone(), value.clone()));
    }

    Resource::builder().with_attributes(kvs).build()
}
