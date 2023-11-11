@tool
extends EditorPlugin

# Replace this value with a PascalCase autoload name, as per the GDScript style guide.
const AUTOLOAD_NAME: String = "Kataru"

# A class member to hold the dock during the plugin life cycle.
var dock


func _enter_tree():
	# Initialization of the plugin goes here.
	# Load the dock scene and instantiate it.
	dock = preload("res://addons/kataru/dock.tscn").instantiate()

	# Add the loaded scene to the docks.
	add_control_to_dock(DOCK_SLOT_LEFT_UR, dock)
	# Note that LEFT_UL means the left of the editor, upper-left dock.

	# The autoload can be a scene or script file.
	add_autoload_singleton(AUTOLOAD_NAME, "res://addons/kataru/kataru.gd")


func _exit_tree():
	# Clean-up of the plugin goes here.
	# Remove the dock.
	remove_control_from_docks(dock)
	# Erase the control from the memory.
	dock.free()
