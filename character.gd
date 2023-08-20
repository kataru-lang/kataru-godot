extends Node


# Called when the node enters the scene tree for the first time.
func _ready():
	Kataru.dialogue.connect(_on_dialogue)
	Kataru.command.connect(_on_command)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass


func _on_dialogue(char_name: String, text: String, _attributes: Array):
	if self.name == char_name:
		print(char_name, ": ", text)

func _on_command(cmd_name: String, params: Dictionary):
	if String.begins_with(self.name + "."):
		print(cmd_name, params)
