# DO NOT EDIT.
# This file was autogenerated by Kataru based on your story.
const Alice = "Alice"
const Bob = "Bob"
const Charlie = "Charlie"

const NAMES: Array[String] = [
	Alice,
	Bob,
	Charlie,
]


# Returns the property for usage in the editor.
static func property(property_name: String) -> Dictionary:
	return {
		"name": property_name,
		"type": TYPE_STRING,
		"usage": PROPERTY_USAGE_DEFAULT,
		"hint": PROPERTY_HINT_ENUM,
		"hint_string": "Alice,Bob,Charlie"
	}
