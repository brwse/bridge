use schemars::Schema;

pub fn remove_excess(schema: &mut Schema) {
    let object = schema.as_object_mut().unwrap();
    object.remove("$schema");
    object.remove("title");
}
