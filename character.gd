extends Node


# Called when the node enters the scene tree for the first time.
func _ready():
	Kataru.dialogue.connect(_on_dialogue)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass


func _on_dialogue(char_name: String, text: String, _attributes: Array):
	if self.name == char_name:
		print(char_name, ": ", text)
