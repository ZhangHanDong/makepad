#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeHostFieldType {
    String,
    Bool,
    Usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeHostFieldSchema {
    pub name: &'static str,
    pub ty: NativeHostFieldType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeHostCommandSchema {
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeHostEventSchema {
    pub name: &'static str,
    pub fields: &'static [NativeHostFieldSchema],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeHostComponentSchema {
    pub name: &'static str,
    pub props: &'static [NativeHostFieldSchema],
    pub prop_updates: &'static [NativeHostFieldSchema],
    pub commands: &'static [NativeHostCommandSchema],
    pub events: &'static [NativeHostEventSchema],
}

pub const NATIVE_TEXT_INPUT_PROPS: &[NativeHostFieldSchema] = &[
    NativeHostFieldSchema {
        name: "text",
        ty: NativeHostFieldType::String,
    },
    NativeHostFieldSchema {
        name: "placeholder",
        ty: NativeHostFieldType::String,
    },
    NativeHostFieldSchema {
        name: "editable",
        ty: NativeHostFieldType::Bool,
    },
];

pub const NATIVE_TEXT_INPUT_PROP_UPDATES: &[NativeHostFieldSchema] = &[
    NativeHostFieldSchema {
        name: "text",
        ty: NativeHostFieldType::String,
    },
    NativeHostFieldSchema {
        name: "placeholder",
        ty: NativeHostFieldType::String,
    },
    NativeHostFieldSchema {
        name: "editable",
        ty: NativeHostFieldType::Bool,
    },
];

pub const NATIVE_TEXT_INPUT_COMMANDS: &[NativeHostCommandSchema] = &[
    NativeHostCommandSchema { name: "focus" },
    NativeHostCommandSchema { name: "blur" },
    NativeHostCommandSchema { name: "select_all" },
    NativeHostCommandSchema { name: "copy" },
    NativeHostCommandSchema { name: "cut" },
    NativeHostCommandSchema { name: "paste" },
];

pub const NATIVE_TEXT_INPUT_CHANGED_FIELDS: &[NativeHostFieldSchema] = &[NativeHostFieldSchema {
    name: "text",
    ty: NativeHostFieldType::String,
}];

pub const NATIVE_TEXT_INPUT_FOCUS_CHANGED_FIELDS: &[NativeHostFieldSchema] =
    &[NativeHostFieldSchema {
        name: "has_focus",
        ty: NativeHostFieldType::Bool,
    }];

pub const NATIVE_TEXT_INPUT_SELECTION_CHANGED_FIELDS: &[NativeHostFieldSchema] = &[
    NativeHostFieldSchema {
        name: "start",
        ty: NativeHostFieldType::Usize,
    },
    NativeHostFieldSchema {
        name: "end",
        ty: NativeHostFieldType::Usize,
    },
];

pub const NATIVE_TEXT_INPUT_EVENTS: &[NativeHostEventSchema] = &[
    NativeHostEventSchema {
        name: "changed",
        fields: NATIVE_TEXT_INPUT_CHANGED_FIELDS,
    },
    NativeHostEventSchema {
        name: "focus_changed",
        fields: NATIVE_TEXT_INPUT_FOCUS_CHANGED_FIELDS,
    },
    NativeHostEventSchema {
        name: "selection_changed",
        fields: NATIVE_TEXT_INPUT_SELECTION_CHANGED_FIELDS,
    },
];

pub const NATIVE_LABEL_PROPS: &[NativeHostFieldSchema] = &[NativeHostFieldSchema {
    name: "text",
    ty: NativeHostFieldType::String,
}];

pub const NATIVE_LABEL_PROP_UPDATES: &[NativeHostFieldSchema] = &[NativeHostFieldSchema {
    name: "text",
    ty: NativeHostFieldType::String,
}];

pub const NATIVE_HOST_SCHEMA: &[NativeHostComponentSchema] = &[
    NativeHostComponentSchema {
        name: "TextInput",
        props: NATIVE_TEXT_INPUT_PROPS,
        prop_updates: NATIVE_TEXT_INPUT_PROP_UPDATES,
        commands: NATIVE_TEXT_INPUT_COMMANDS,
        events: NATIVE_TEXT_INPUT_EVENTS,
    },
    NativeHostComponentSchema {
        name: "Label",
        props: NATIVE_LABEL_PROPS,
        prop_updates: NATIVE_LABEL_PROP_UPDATES,
        commands: &[],
        events: &[],
    },
];

pub fn native_host_component(name: &str) -> Option<&'static NativeHostComponentSchema> {
    NATIVE_HOST_SCHEMA
        .iter()
        .find(|component| component.name == name)
}

pub fn generate_native_host_manifest() -> String {
    let mut out = String::new();
    for component in NATIVE_HOST_SCHEMA {
        out.push_str("component ");
        out.push_str(component.name);
        out.push('\n');
        write_field_list(&mut out, "props", component.props);
        write_field_list(&mut out, "prop_updates", component.prop_updates);
        if !component.commands.is_empty() {
            out.push_str("commands");
            for command in component.commands {
                out.push(' ');
                out.push_str(command.name);
            }
            out.push('\n');
        }
        for event in component.events {
            write_field_list(&mut out, event.name, event.fields);
        }
    }
    out
}

fn write_field_list(out: &mut String, name: &str, fields: &[NativeHostFieldSchema]) {
    if fields.is_empty() {
        return;
    }
    out.push_str(name);
    for field in fields {
        out.push(' ');
        out.push_str(field.name);
        out.push(':');
        out.push_str(match field.ty {
            NativeHostFieldType::String => "String",
            NativeHostFieldType::Bool => "bool",
            NativeHostFieldType::Usize => "usize",
        });
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_host_schema_covers_current_components() {
        let text_input = native_host_component("TextInput").unwrap();
        assert_eq!(text_input.props.len(), 3);
        assert!(text_input
            .commands
            .iter()
            .any(|command| command.name == "focus"));
        assert!(text_input
            .commands
            .iter()
            .any(|command| command.name == "blur"));
        assert!(text_input
            .events
            .iter()
            .any(|event| event.name == "changed"));
        assert!(text_input
            .events
            .iter()
            .any(|event| event.name == "focus_changed"));
        assert!(text_input
            .events
            .iter()
            .any(|event| event.name == "selection_changed"));

        let label = native_host_component("Label").unwrap();
        assert_eq!(label.props.len(), 1);
        assert_eq!(label.props[0].name, "text");
    }

    #[test]
    fn native_host_manifest_codegen_is_stable() {
        let manifest = generate_native_host_manifest();
        assert_eq!(
            manifest,
            "\
component TextInput
props text:String placeholder:String editable:bool
prop_updates text:String placeholder:String editable:bool
commands focus blur select_all copy cut paste
changed text:String
focus_changed has_focus:bool
selection_changed start:usize end:usize
component Label
props text:String
prop_updates text:String
"
        );
    }
}
