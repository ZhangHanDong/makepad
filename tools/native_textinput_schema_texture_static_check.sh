#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

lib_rs="platform/src/lib.rs"
schema_rs="platform/src/native_host_schema.rs"
texture_rs="platform/src/native_texture_layer.rs"
cx_api_rs="platform/src/cx_api.rs"
roadmap="docs/native-textinput-macos-poc.roadmap.md"
audit="docs/native-textinput-macos-poc.audit.md"

require_pattern() {
    local file="$1"
    local pattern="$2"
    local description="$3"
    if ! rg -q -- "$pattern" "$file"; then
        echo "missing ${description}: ${pattern} in ${file}" >&2
        exit 1
    fi
}

require_pattern "$lib_rs" "pub mod native_host_schema;" "schema module export"
require_pattern "$lib_rs" "pub mod native_texture_layer;" "texture layer module export"
require_pattern "$lib_rs" "NativeTextureLayerBridge" "texture bridge re-export"

require_pattern "$schema_rs" "pub enum NativeHostFieldType" "schema field type enum"
require_pattern "$schema_rs" "pub struct NativeHostComponentSchema" "component schema struct"
require_pattern "$schema_rs" "pub const NATIVE_TEXT_INPUT_PROPS" "TextInput props schema"
require_pattern "$schema_rs" "name: \"text\"" "text field schema"
require_pattern "$schema_rs" "name: \"placeholder\"" "placeholder field schema"
require_pattern "$schema_rs" "name: \"editable\"" "editable field schema"
require_pattern "$schema_rs" "pub const NATIVE_TEXT_INPUT_COMMANDS" "TextInput command schema"
require_pattern "$schema_rs" "name: \"focus\"" "focus command schema"
require_pattern "$schema_rs" "name: \"blur\"" "blur command schema"
require_pattern "$schema_rs" "name: \"select_all\"" "select all command schema"
require_pattern "$schema_rs" "name: \"copy\"" "copy command schema"
require_pattern "$schema_rs" "name: \"cut\"" "cut command schema"
require_pattern "$schema_rs" "name: \"paste\"" "paste command schema"
require_pattern "$schema_rs" "name: \"changed\"" "changed event schema"
require_pattern "$schema_rs" "name: \"focus_changed\"" "focus event schema"
require_pattern "$schema_rs" "name: \"selection_changed\"" "selection event schema"
require_pattern "$schema_rs" "name: \"start\"" "selection start field"
require_pattern "$schema_rs" "name: \"end\"" "selection end field"
require_pattern "$schema_rs" "NativeHostFieldType::Usize" "usize schema type"
require_pattern "$schema_rs" "name: \"Label\"" "Label component schema"
require_pattern "$schema_rs" "pub fn native_host_component" "schema lookup helper"
require_pattern "$schema_rs" "pub fn generate_native_host_manifest\\(\\) -> String" "manifest generator"
require_pattern "$schema_rs" "commands focus blur select_all copy cut paste" "manifest command golden"
require_pattern "$schema_rs" "selection_changed start:usize end:usize" "manifest selection golden"
require_pattern "$schema_rs" "fn native_host_schema_covers_current_components\\(\\)" "schema coverage test"
require_pattern "$schema_rs" "fn native_host_manifest_codegen_is_stable\\(\\)" "manifest stability test"

require_pattern "$cx_api_rs" "fn native_host_schema_matches_typed_kinds\\(\\)" "schema typed kind parity test"
require_pattern "$cx_api_rs" "fn native_host_schema_matches_typed_props\\(\\)" "schema typed props parity test"
require_pattern "$cx_api_rs" "fn native_host_schema_matches_typed_prop_updates\\(\\)" "schema typed prop updates parity test"
require_pattern "$cx_api_rs" "fn native_host_schema_matches_typed_commands_and_events\\(\\)" "schema command/event parity test"

require_pattern "$texture_rs" "pub struct NativeTextureLayer" "texture layer struct"
require_pattern "$texture_rs" "pub struct NativeTextureLayerFrame" "texture layer frame struct"
require_pattern "$texture_rs" "pub trait NativeTextureLayerBridge" "texture bridge trait"
require_pattern "$texture_rs" "fn alloc_native_texture_layer\\(&mut self, host_id: LiveId\\) -> NativeTextureLayer" "texture allocation API"
require_pattern "$texture_rs" "TextureFormat::VideoExternal" "external texture allocation"
require_pattern "$texture_rs" "generation: 0" "initial generation"
require_pattern "$texture_rs" "saturating_add\\(1\\)" "frame generation increment"
require_pattern "$texture_rs" "fn native_texture_layer_allocates_external_texture\\(\\)" "external texture test"
require_pattern "$texture_rs" "fn native_texture_layer_allocates_distinct_texture_per_host\\(\\)" "distinct texture test"
require_pattern "$texture_rs" "fn native_texture_layer_marks_frame_generations\\(\\)" "generation test"

require_pattern "$roadmap" "Seed complete, generator replacement not implemented|typed enum 替换为生成产物" "P6 boundary"
require_pattern "$roadmap" "尚未把任何 native widget 渲染进 texture" "P7 boundary"
require_pattern "$audit" "P6 \\| schema / codegen seed" "P6 audit row"
require_pattern "$audit" "P7 \\| native texture layer seed" "P7 audit row"

echo "NativeHost schema and texture layer static contract check passed."
