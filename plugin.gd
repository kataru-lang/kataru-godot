@tool
extends EditorPlugin

# Replace this value with a PascalCase autoload name, as per the GDScript style guide.
const AUTOLOAD_NAME: String = "Kataru"

var scene


func _enter_tree():
	add_custom_type(
		"Kataru",
		"Node",
		preload("res://addons/kataru/kataru.gd"),
		preload("res://addons/kataru/images/editor.png")
	)
	add_autoload_singleton(AUTOLOAD_NAME, "res://addons/kataru/kataru.tscn")
	# add_control_to_dock(DOCK_SLOT_LEFT_UR, self.get_editor_interface().edit_node(Kataru))


func _exit_tree():
	remove_autoload_singleton(AUTOLOAD_NAME)
	remove_custom_type("Kataru")
