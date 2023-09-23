@tool
extends Node

var character: String


func _get_property_list():
	return [Kataru.Character.property("character")]


# Called when the node enters the scene tree for the first time.
func _ready():
	Kataru.dialogue.connect(_on_dialogue)
	Kataru.command.connect(_on_command)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass


func _on_dialogue(char_name: String, text: String, _attributes: Array):
	# print("Kataru name: ", Kataru.char_name(self.character))
	if self.character == char_name:
		print(char_name, ": ", text)


func _on_command(cmd_name: String, params: Dictionary):
	if cmd_name.begins_with(self.name + "."):
		print(cmd_name, params)
